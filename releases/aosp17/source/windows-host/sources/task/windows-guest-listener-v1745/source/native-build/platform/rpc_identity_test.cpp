#include "DesktopRpcIdentity.h"
#include <binder/Binder.h>
#include <binder/Parcel.h>
#include <binder/RpcServer.h>
#include <binder/RpcSession.h>
#include <cassert>
#include <cstdio>
#include <cstdlib>
#include <spawn.h>
#include <sys/wait.h>
#include <unistd.h>
#include <thread>
#include "RpcWireFormat.h"
#include <sys/un.h>
#include <sys/socket.h>
#include <cstring>
#include <vector>
#include <chrono>
extern char **environ;
static int spawn_child(pid_t* pid,char* executable,char** args,posix_spawnattr_t* attrs){
 posix_spawn_file_actions_t actions;assert(posix_spawn_file_actions_init(&actions)==0);
 assert(posix_spawn_file_actions_adddup2(&actions,STDOUT_FILENO,STDOUT_FILENO)==0);
 assert(posix_spawn_file_actions_adddup2(&actions,STDERR_FILENO,STDERR_FILENO)==0);
 assert(posix_spawn_file_actions_addclosefrom_np(&actions,3)==0);
 int status=posix_spawn(pid,executable,&actions,attrs,args,environ);
 posix_spawn_file_actions_destroy(&actions);return status;
}
using namespace android;
class Root:public BBinder {
 status_t onTransact(uint32_t code,const Parcel&,Parcel* reply,uint32_t) override {
  if(code!=1)return BAD_VALUE;
  auto peer=desktop_rpc::current();if(!peer)return PERMISSION_DENIED;
  return reply->writeInt32(peer->process);
 }
};
static void send_all(int fd,const void* p,size_t n){auto b=static_cast<const char*>(p);while(n){ssize_t k=send(fd,b,n,0);assert(k>0);b+=k;n-=k;}}
static void recv_all(int fd,void* p,size_t n){auto b=static_cast<char*>(p);while(n){ssize_t k=recv(fd,b,n,0);assert(k>0);b+=k;n-=k;}}
static int connect_raw(const char* path){int fd=socket(AF_UNIX,SOCK_STREAM,0);assert(fd>=0);timeval limit{3,0};assert(setsockopt(fd,SOL_SOCKET,SO_RCVTIMEO,&limit,sizeof(limit))==0);sockaddr_un a{};a.sun_family=AF_UNIX;assert(std::strlen(path)<sizeof(a.sun_path));std::strcpy(a.sun_path,path);assert(connect(fd,reinterpret_cast<sockaddr*>(&a),sizeof(a))==0);return fd;}
static void hello(int fd,const std::vector<uint8_t>& id){RpcConnectionHeader h{};h.version=1;h.sessionIdSize=id.size();send_all(fd,&h,sizeof(h));if(!id.empty())send_all(fd,id.data(),id.size());else {RpcNewSessionResponse response{};recv_all(fd,&response,sizeof(response));assert(response.version==1);}}
static std::vector<uint8_t> session_id(int fd){RpcOutgoingConnectionInit init{};std::memcpy(init.msg,"cci",4);send_all(fd,&init,sizeof(init));RpcWireHeader h{};h.command=RPC_COMMAND_TRANSACT;h.bodySize=sizeof(RpcWireTransaction);RpcWireTransaction t{};t.code=RPC_SPECIAL_TRANSACT_GET_SESSION_ID;send_all(fd,&h,sizeof(h));send_all(fd,&t,sizeof(t));recv_all(fd,&h,sizeof(h));assert(h.command==RPC_COMMAND_REPLY&&h.bodySize>=sizeof(RpcWireReply)&&h.bodySize<1024);std::vector<uint8_t>b(h.bodySize);recv_all(fd,b.data(),b.size());RpcWireReply reply{};std::memcpy(&reply,b.data(),sizeof(reply));assert(reply.status==OK&&reply.parcelDataSize>=4);uint32_t size;std::memcpy(&size,b.data()+sizeof(reply),4);assert(size>0&&size<=128&&sizeof(reply)+4+size<=b.size());return std::vector<uint8_t>(b.begin()+sizeof(reply)+4,b.begin()+sizeof(reply)+4+size);}
int main(int argc,char**argv){
 std::thread([]{std::this_thread::sleep_for(std::chrono::seconds(20));std::_Exit(124);}).detach();
 if(argc==4&&std::string(argv[1])=="attach"){
  std::string hex=argv[3];assert(hex.size()%2==0);std::vector<uint8_t> id;for(size_t i=0;i<hex.size();i+=2)id.push_back(std::stoul(hex.substr(i,2),nullptr,16));
  int fd=connect_raw(argv[2]);hello(fd,id);char byte;assert(recv(fd,&byte,1,0)==0);close(fd);std::puts("cross_process_existing_session=DENIED");return 0;
 }
 if(argc==3){
  auto s=RpcSession::make();assert(s->setupUnixDomainClient(argv[2])==OK);auto root=s->getRootObject();assert(root);
  if(std::string(argv[1])=="denied") {Parcel d,q;d.markForRpc(s);assert(root->transact(1,d,&q,0)==PERMISSION_DENIED);assert(s->shutdownAndWait(true));return 0;}
  int32_t peer=0;for(int j=0;j<16;j++){Parcel data,reply;data.markForRpc(s);assert(root->transact(1,data,&reply,0)==OK);peer=reply.readInt32();assert(peer==getpid());}
  std::printf("actual_client_pid=%d authenticated_pid=%d\n",getpid(),peer);std::fflush(stdout);assert(s->shutdownAndWait(true));return 0;
 }
 char directory[]="/tmp/bscp-rpc-auth-XXXXXX";assert(mkdtemp(directory));std::string path=std::string(directory)+"/rpc.sock";
 auto server=RpcServer::make();assert(server->setDesktopAuthentication(getuid(),getgid())==OK);server->setRootObject(sp<Root>::make());server->setMaxThreads(2);assert(server->setupUnixDomainServer(path.c_str())==OK);server->start();
 pid_t children[2];
 for(int i=0;i<2;i++){
  posix_spawnattr_t attrs;assert(posix_spawnattr_init(&attrs)==0);
  char mode[]="client";char*args[]={argv[0],mode,path.data(),nullptr};pid_t pid;assert(spawn_child(&pid,argv[0],args,&attrs)==0);posix_spawnattr_destroy(&attrs);
  children[i]=pid;
 }
 for(pid_t pid:children){int status;assert(waitpid(pid,&status,0)==pid);assert(WIFEXITED(status)&&WEXITSTATUS(status)==0);std::printf("client_pid=%d actual_identity_16_requests=PASS exit=0\n",pid);
 }
 assert(server->setDesktopAuthentication(getuid(),getgid())==INVALID_OPERATION);
 int original=connect_raw(path.c_str());hello(original,{});auto id=session_id(original);
 int same=connect_raw(path.c_str());hello(same,id);assert(session_id(same)==id);
 std::string hex;const char*digits="0123456789abcdef";for(auto c:id){hex+=digits[c>>4];hex+=digits[c&15];}
 {posix_spawnattr_t attrs;assert(posix_spawnattr_init(&attrs)==0);char mode[]="attach";char*args[]={argv[0],mode,path.data(),hex.data(),nullptr};pid_t pid;assert(spawn_child(&pid,argv[0],args,&attrs)==0);posix_spawnattr_destroy(&attrs);int status;assert(waitpid(pid,&status,0)==pid);assert(WIFEXITED(status)&&WEXITSTATUS(status)==0);}
 close(same);close(original);
 auto rejected=RpcServer::make();assert(rejected->setDesktopAuthentication(0,getgid())==PERMISSION_DENIED);
 assert(server->shutdown());assert(!desktop_rpc::current());
 auto unauth=RpcServer::make();unauth->setRootObject(sp<Root>::make());std::string denied=std::string(directory)+"/denied.sock";assert(unauth->setupUnixDomainServer(denied.c_str())==OK);unauth->start();
 posix_spawnattr_t attrs;assert(posix_spawnattr_init(&attrs)==0);char mode[]="denied";char*args[]={argv[0],mode,denied.data(),nullptr};pid_t pid;assert(spawn_child(&pid,argv[0],args,&attrs)==0);posix_spawnattr_destroy(&attrs);int status;assert(waitpid(pid,&status,0)==pid);assert(WIFEXITED(status)&&WEXITSTATUS(status)==0);assert(unauth->shutdown());std::puts("server_without_identity=DENIED");std::printf("two_independent_clients=PASS shutdown=PASS no_identity_outside_dispatch=PASS path=%s\n",path.c_str());
}
