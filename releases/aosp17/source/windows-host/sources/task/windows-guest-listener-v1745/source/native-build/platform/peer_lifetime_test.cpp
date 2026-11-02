
#include "DesktopRpcIdentity.h"
#include <cassert>
#include <cerrno>
#include <cstdio>
#include <cstring>
#include <dirent.h>
#include <fcntl.h>
#include <poll.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <unistd.h>
#include <thread>
#include <chrono>
using namespace android::desktop_rpc;
static int fds(){int n=0;DIR* d=opendir("/proc/self/fd");assert(d);while(auto* e=readdir(d))if(e->d_name[0]!='.')n++;closedir(d);return n;}
struct Child{pid_t pid;int control;int socket;};
static Child child(int listener,const sockaddr_un& address){
 int p[2];assert(pipe2(p,O_CLOEXEC)==0);pid_t pid=fork();assert(pid>=0);
 if(pid==0){close(p[1]);close(listener);int s=socket(AF_UNIX,SOCK_STREAM|SOCK_CLOEXEC,0);if(s<0 || connect(s,reinterpret_cast<const sockaddr*>(&address),sizeof(address)))_exit(2);
  pollfd wait{p[0],POLLIN|POLLHUP,0};int rc=poll(&wait,1,10000);char c=0;if(rc>0)read(p[0],&c,1);close(s);close(p[0]);_exit(c=='X'?0:3);
 }
 close(p[0]);pollfd wait{listener,POLLIN,0};assert(poll(&wait,1,3000)==1);int conn=accept4(listener,nullptr,nullptr,SOCK_CLOEXEC);assert(conn>=0);return {pid,p[1],conn};
}
static void finish(Child& c){assert(write(c.control,"X",1)==1);close(c.control);int status;assert(waitpid(c.pid,&status,0)==c.pid);assert(WIFEXITED(status)&&WEXITSTATUS(status)==0);close(c.socket);}
int main(){
 char directory[]="/tmp/bscp-peer-lifetime-XXXXXX";assert(mkdtemp(directory));sockaddr_un a{};a.sun_family=AF_UNIX;std::snprintf(a.sun_path,sizeof(a.sun_path),"%s/s",directory);
 int listener=socket(AF_UNIX,SOCK_STREAM|SOCK_CLOEXEC,0);assert(listener>=0);assert(bind(listener,reinterpret_cast<sockaddr*>(&a),sizeof(a))==0);assert(listen(listener,2)==0);
 auto one=child(listener,a);auto two=child(listener,a);
 std::thread([]{std::this_thread::sleep_for(std::chrono::seconds(20));std::_Exit(124);}).detach();
 int baseline=fds();
 {
 Peer p,q,other;assert(capture(one.socket,getuid(),getgid(),&p)==0);assert(p.process==one.pid);
 assert(capture(one.socket,getuid(),getgid(),&q)==0);assert(p==q);
 assert(capture(two.socket,getuid(),getgid(),&other)==0);assert(!(p==other));
 Peer denied;assert(capture(one.socket,0,getgid(),&denied)==EACCES);assert(!denied.handle);
 assert(capture(-1,getuid(),getgid(),&denied)==EBADF);assert(!denied.handle);
 int datagram=socket(AF_UNIX,SOCK_DGRAM|SOCK_CLOEXEC,0);assert(capture(datagram,getuid(),getgid(),&denied)==EPROTOTYPE);close(datagram);
 assert(!current());
 {Scope outer(p);assert(current()->process==one.pid);
  {Scope blank(std::nullopt);assert(!current());}
  assert(current()->process==one.pid);
  {Scope inner(other);assert(current()->process==two.pid);}
  assert(current()->process==one.pid);
  std::thread independent([&]{assert(!current());Scope scope(other);assert(current()->process==two.pid);});independent.join();
  assert(current()->process==one.pid);
 }
 assert(!current());
 finish(one);assert(!p.alive());assert(!(p==q));{Scope stale(p);assert(!current());int pid=0;unsigned uid=0,gid=0;assert(BSCP_RpcCallingLinuxCredentials(&pid,&uid,&gid)==ENOENT);}
 finish(two);assert(!other.alive());
 }
 assert(fds()==baseline-4); // two original sockets and two control pipes closed; all pidfds released
 close(listener);assert(!current());std::printf("same_peer=PASS different_peer=DENIED exited_original=DENIED nested_restore=PASS thread_isolation=PASS fd_release=PASS no_signals=PASS path=%s\n",directory);
}
