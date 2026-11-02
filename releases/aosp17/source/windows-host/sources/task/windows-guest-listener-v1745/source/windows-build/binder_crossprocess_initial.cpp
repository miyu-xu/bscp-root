#include <binder/Binder.h>
#include <binder/Parcel.h>
#include <binder/RpcServer.h>
#include <binder/RpcSession.h>
#include "WindowsLocalRpcBridge.h"
#include "InitialBootstrap.h"
#include "WindowsFdAbi.h"
#include "WindowsCallerIdentity.h"
#include <atomic>
#include <thread>
#include <cstdio>
#include <vector>
#include <string>
using namespace android; using namespace bscp::windows;
static HANDLE begun,releaseFirst,secondDone,ready,stopServer;
static std::atomic<unsigned> first{0},second{0},third{0},markers{0},foreign{0},errors{0};
static std::atomic<unsigned> normalCallbacks{0},stopCallbacks{0},fdUnexpected{0};
static std::atomic<status_t> callbackStopStatus{UNKNOWN_ERROR};
static LUID firstToken{}; static DWORD firstPid=0;
static bool identity(DWORD*pid,LUID*id){auto*c=CallerScope::current();TOKEN_STATISTICS t{};DWORD n=0;return c&&!c->checkedCallingPid(pid)&&GetTokenInformation(c->queryTokenForAccessCheck(),TokenStatistics,&t,sizeof(t),&n)&&(*id=t.TokenId,true);}
static void watchdog(DWORD ms){std::thread([ms]{Sleep(ms);fprintf(stderr,"SELF_DEADLINE_FAIL pid=%lu\n",GetCurrentProcessId());fflush(nullptr);ExitProcess(90);}).detach();}
#define NEED(x) do{if(!(x)){fprintf(stderr,"FAIL line=%d pid=%lu err=%lu\n",__LINE__,GetCurrentProcessId(),GetLastError());return 1;}}while(0)
class Callback final:public BBinder {wp<RpcSession> session;bool stop;
 public:Callback(const sp<RpcSession>& s,bool doStop):session(s),stop(doStop){}
 status_t onTransact(uint32_t code,const Parcel&,Parcel* reply,uint32_t)override{
  if(code!=1||CallerScope::current())return PERMISSION_DENIED;
  if(stop){auto s=session.promote();if(!s)return DEAD_OBJECT;uint32_t state=0;int32_t setup=0;status_t e=s->shutdownJoinWindowsInitial(10,&state,&setup);callbackStopStatus=e;++stopCallbacks;printf("CALLBACK_STOP result=%d state=%u setup=%d\n",e,state,setup);if(e!=WOULD_BLOCK)return BAD_VALUE;}
  else ++normalCallbacks;
  return reply?reply->writeInt32(42):OK;
 }
};
/* This helper asserts resource closure only. Sticky protocol/setup status is
 * preserved verbatim; no stop race or status whitelist turns it into success.
 * Business correctness is asserted separately by each reply/handler count. */
static bool closeClient(sp<RpcSession>& s,const char* label){if(!s)return true;uint32_t state=0;int32_t setup=0;status_t close=s->shutdownJoinWindowsInitial(5000,&state,&setup);status_t release=close==OK?s->releaseWindowsInitialStatus():WOULD_BLOCK;printf("CLIENT_CLEANUP name=%s result=%d state=%u stickySetup=%d release=%d\n",label,close,state,setup,release);if(close!=OK||release!=OK){/* Exact owner deliberately retained on failure. */auto* retained=new sp<RpcSession>(s);(void)retained;return false;}s.clear();return state==5;}
class Service final:public BBinder{status_t onTransact(uint32_t code,const Parcel&data,Parcel*reply,uint32_t)override{
 DWORD pid=0;LUID token{};if(!identity(&pid,&token)){++errors;return PERMISSION_DENIED;}
 if(code==1){firstPid=pid;firstToken=token;++first;SetEvent(begun);if(WaitForSingleObject(releaseFirst,7000)!=WAIT_OBJECT_0){++errors;return TIMED_OUT;}DWORD after=0;LUID t{};if(!identity(&after,&t)||after!=pid||t.LowPart!=token.LowPart||t.HighPart!=token.HighPart)++errors;return OK;}
 if(code==2){bool distinct=token.LowPart!=firstToken.LowPart||token.HighPart!=firstToken.HighPart;bool valid=first==1&&pid==firstPid&&distinct;if(!valid)++errors;++second;printf("DEQUEUED pid=%lu firstToken=%ld:%lu secondToken=%ld:%lu distinct=%d\n",pid,firstToken.HighPart,firstToken.LowPart,token.HighPart,token.LowPart,distinct);fflush(stdout);return valid?OK:BAD_VALUE;}
 if(code==3){++third;SetEvent(secondDone);return OK;}
 if(code==4){bool held=WaitForSingleObject(releaseFirst,0)==WAIT_TIMEOUT;bool valid=first==1&&second==0&&held;++markers;if(!valid)++errors;printf("SECOND_CONNECTION_MARKER first=%u second=%u held=%d pid=%lu\n",first.load(),second.load(),held,pid);fflush(stdout);return reply?reply->writeInt32(valid?1:0):BAD_VALUE;}
 if(code==5){++foreign;return reply?reply->writeInt32(static_cast<int32_t>(pid)):BAD_VALUE;}
 if(code==6||code==7){auto cb=data.readStrongBinder();if(!cb)return BAD_VALUE;Parcel in,out;in.markForBinder(cb);status_t e=cb->transact(1,in,&out,0);if(code==7){printf("STOP_CALLBACK_OUTBOUND result=%d\n",e);return e;}int32_t v=0;if(e||out.readInt32(&v)||v!=42||!identity(&pid,&token))return BAD_VALUE;return reply?reply->writeInt32(v):BAD_VALUE;}
 if(code==8){++fdUnexpected;return BAD_VALUE;}
 if(code==9)return reply?reply->writeInt32(static_cast<int32_t>(fdUnexpected.load())):BAD_VALUE;
 return BAD_VALUE;
}};
static sp<RpcSession> configured(const wchar_t*endpoint,HANDLE original,RpcSession::FileDescriptorTransportMode mode){InitialAssociation*owner=nullptr;if(InitialCreate(endpoint,original,&owner))return nullptr;auto s=RpcSession::makeWindowsInitial(&owner);if(!s){if(owner)InitialRelease(&owner);return nullptr;}s->setMaxIncomingThreads(1);s->setMaxOutgoingConnections(2);s->setFileDescriptorTransportMode(mode);return s;}
static sp<RpcSession> connect(const wchar_t*endpoint,HANDLE original){auto s=configured(endpoint,original,RpcSession::FileDescriptorTransportMode::UNIX);if(s&&s->setupWindowsDesktopClient()!=OK){closeClient(s,"setup-failed");return nullptr;}return s;}
static HANDLE parse(const wchar_t*s){return reinterpret_cast<HANDLE>(wcstoull(s,nullptr,10));}
static std::wstring value(HANDLE h){return std::to_wstring(reinterpret_cast<uintptr_t>(h));}
struct Child{HANDLE process=nullptr,output=nullptr;DWORD pid=0;};
static bool spawn(const std::wstring&args,std::vector<HANDLE> inherit,Child*out){
 SECURITY_ATTRIBUTES sa{sizeof(sa),nullptr,TRUE};HANDLE rd=nullptr,wr=nullptr; if(!CreatePipe(&rd,&wr,&sa,0))return false;SetHandleInformation(rd,HANDLE_FLAG_INHERIT,0);HANDLE input=CreateFileW(L"NUL",GENERIC_READ,FILE_SHARE_READ|FILE_SHARE_WRITE,&sa,OPEN_EXISTING,0,nullptr);if(input==INVALID_HANDLE_VALUE){CloseHandle(rd);CloseHandle(wr);return false;}inherit.push_back(wr);inherit.push_back(input);
 SIZE_T n=0;InitializeProcThreadAttributeList(nullptr,1,0,&n);std::vector<unsigned char> storage(n);auto*attrs=reinterpret_cast<LPPROC_THREAD_ATTRIBUTE_LIST>(storage.data());if(!InitializeProcThreadAttributeList(attrs,1,0,&n)){CloseHandle(rd);CloseHandle(wr);CloseHandle(input);return false;}bool ok=UpdateProcThreadAttribute(attrs,0,PROC_THREAD_ATTRIBUTE_HANDLE_LIST,inherit.data(),inherit.size()*sizeof(HANDLE),nullptr,nullptr)!=FALSE;
 wchar_t exe[32768];DWORD len=GetModuleFileNameW(nullptr,exe,32768);std::wstring cmd=L"\""+std::wstring(exe,len)+L"\" "+args;STARTUPINFOEXW si{};si.StartupInfo.cb=sizeof(si);si.StartupInfo.dwFlags=STARTF_USESTDHANDLES;si.StartupInfo.hStdInput=input;si.StartupInfo.hStdOutput=wr;si.StartupInfo.hStdError=wr;si.lpAttributeList=attrs;PROCESS_INFORMATION pi{};if(ok)ok=CreateProcessW(nullptr,cmd.data(),nullptr,nullptr,TRUE,EXTENDED_STARTUPINFO_PRESENT|CREATE_NO_WINDOW,nullptr,nullptr,&si.StartupInfo,&pi)!=FALSE;DeleteProcThreadAttributeList(attrs);CloseHandle(wr);CloseHandle(input);if(!ok){CloseHandle(rd);return false;}CloseHandle(pi.hThread);out->process=pi.hProcess;out->output=rd;out->pid=pi.dwProcessId;return true;
}
static bool reap(Child&c,const char*name){if(!c.process)return true;DWORD w=WaitForSingleObject(c.process,35000),exit=999;bool ok=w==WAIT_OBJECT_0&&GetExitCodeProcess(c.process,&exit);printf("ORIGINAL_PROCESS name=%s pid=%lu wait=%lu exit=%lu\n",name,c.pid,w,exit);if(w!=WAIT_OBJECT_0)return false;char buf[4096];DWORD n=0;while(ReadFile(c.output,buf,sizeof(buf),&n,nullptr)&&n)fwrite(buf,1,n,stdout);DWORD e=GetLastError();bool eof=e==ERROR_BROKEN_PIPE;printf("REAL_EOF name=%s value=%d error=%lu\n",name,eof,e);CloseHandle(c.output);CloseHandle(c.process);c={};return ok&&exit==0&&eof;}
int wmain(int argc,wchar_t**argv){setvbuf(stdout,nullptr,_IONBF,0);watchdog(argc>1?25000:60000);
 if(argc>1&&std::wstring(argv[1])==L"service"){
  NEED(argc==8);ready=parse(argv[3]);stopServer=parse(argv[4]);begun=parse(argv[5]);releaseFirst=parse(argv[6]);secondDone=parse(argv[7]);WlrServerAcceptor*a=nullptr;NEED(!WlrStartOwnedListener(argv[2],&a));auto server=RpcServer::makeWindowsDesktop(&a);NEED(server&&!a);server->setMaxThreads(2);server->setSupportedFileDescriptorTransportModes({RpcSession::FileDescriptorTransportMode::UNIX,RpcSession::FileDescriptorTransportMode::NONE});server->setRootObject(sp<Service>::make());server->start();NEED(SetEvent(ready));DWORD stop=WaitForSingleObject(stopServer,20000);SetEvent(releaseFirst);bool down=server->shutdown();server.clear();LONG s,c,r,f;WlrCounters(&s,&c,&r,&f);DWORD st=WlrStop();printf("SERVICE_END first=%u second=%u third=%u markers=%u foreign=%u errors=%u shutdown=%d counters=%ld,%ld,%ld,%ld stop=%lu\n",first.load(),second.load(),third.load(),markers.load(),foreign.load(),errors.load(),down,s,c,r,f,st);for(HANDLE h:{ready,stopServer,begun,releaseFirst,secondDone})CloseHandle(h);return stop==WAIT_OBJECT_0&&down&&!s&&!c&&!r&&!f&&!st&&first==1&&second==1&&third==1&&markers==1&&foreign==1&&!errors?0:1;
 }
 if(argc>1&&std::wstring(argv[1])==L"client"){
  NEED(argc==4);HANDLE original=parse(argv[3]);auto client=connect(argv[2],original);NEED(client);auto root=client->getRootObject();NEED(root);Parcel in,out;in.markForRpc(client);NEED(root->transact(5,in,&out,0)==OK);int32_t pid=0;NEED(out.readInt32(&pid)==OK&&static_cast<DWORD>(pid)==GetCurrentProcessId());printf("DELEGATED_ORIGINAL_CAP_CLIENT actual_scope_pid=%ld self=%lu\n",static_cast<long>(pid),GetCurrentProcessId());{Parcel callbackIn,callbackOut;callbackIn.markForRpc(client);NEED(callbackIn.writeStrongBinder(sp<Callback>::make(client,true))==OK);status_t task=root->transact(7,callbackIn,&callbackOut,0);printf("CALLBACK_STOP_TASK result=%d callbackStatus=%d callbacks=%u\n",task,callbackStopStatus.load(),stopCallbacks.load());NEED(stopCallbacks==1&&callbackStopStatus==WOULD_BLOCK);}
 root.clear();NEED(closeClient(client,"callback-external-join"));NEED(CloseHandle(original));return 0;
 }
 SECURITY_ATTRIBUTES sa{sizeof(sa),nullptr,TRUE};ready=CreateEventW(&sa,TRUE,FALSE,nullptr);stopServer=CreateEventW(&sa,TRUE,FALSE,nullptr);begun=CreateEventW(&sa,TRUE,FALSE,nullptr);releaseFirst=CreateEventW(&sa,TRUE,FALSE,nullptr);secondDone=CreateEventW(&sa,TRUE,FALSE,nullptr);NEED(ready&&stopServer&&begun&&releaseFirst&&secondDone);std::wstring endpoint=L"binder-cross-"+std::to_wstring(GetCurrentProcessId());Child service,other;std::wstring args=L"service "+endpoint+L" "+value(ready)+L" "+value(stopServer)+L" "+value(begun)+L" "+value(releaseFirst)+L" "+value(secondDone);NEED(spawn(args,{ready,stopServer,begun,releaseFirst,secondDone},&service));
 bool ok=WaitForSingleObject(ready,5000)==WAIT_OBJECT_0;sp<RpcSession>client;sp<IBinder>root;
 if(ok){client=connect(endpoint.c_str(),service.process);ok=client!=nullptr;if(ok){root=client->getRootObject();ok=root!=nullptr;}}
 if(ok){Parcel in,out;in.markForRpc(client);int32_t v=0;ok=in.writeStrongBinder(sp<Callback>::make(client,false))==OK&&root->transact(6,in,&out,0)==OK&&out.readInt32(&v)==OK&&v==42&&normalCallbacks==1;}
 if(ok){auto none=configured(endpoint.c_str(),service.process,RpcSession::FileDescriptorTransportMode::NONE);ok=none&&none->setupWindowsDesktopClient()==OK;if(ok){auto nr=none->getRootObject();std::wstring path=L"binder-none-fd-"+std::to_wstring(GetCurrentProcessId());HANDLE h=CreateFileW(path.c_str(),GENERIC_READ|GENERIC_WRITE,0,nullptr,CREATE_NEW,FILE_FLAG_DELETE_ON_CLOSE,nullptr);int fd=-1;ok=nr&&h!=INVALID_HANDLE_VALUE&&!bscp_fd_from_handle(h,&fd);if(h!=INVALID_HANDLE_VALUE)CloseHandle(h);if(ok){Parcel in,out;in.markForRpc(none);status_t written=in.writeFileDescriptor(fd,false);status_t sent=written==OK?nr->transact(8,in,&out,0):written;printf("NONE_FD write=%d transact=%d\n",written,sent);ok=sent!=OK&&!bscp_fd_close(fd);Parcel probe,answer;probe.markForRpc(none);int32_t count=-1;ok=ok&&nr->transact(9,probe,&answer,0)==OK&&answer.readInt32(&count)==OK&&count==0;}}ok=closeClient(none,"none-mode")&&ok;}
 if(ok){auto early=configured(endpoint.c_str(),service.process,RpcSession::FileDescriptorTransportMode::UNIX);if(!early)ok=false;else{uint32_t state=0;int32_t sticky=0;status_t stop=early->shutdownJoinWindowsInitial(5000,&state,&sticky);status_t setup=early->setupWindowsDesktopClient();printf("EARLY_STOP stop=%d state=%u sticky=%d setupAfter=%d\n",stop,state,sticky,setup);bool ended=closeClient(early,"early-stop");ok=stop==OK&&setup!=OK&&ended;}}
 if(ok){Parcel in;in.markForRpc(client);ok=root->transact(1,in,nullptr,IBinder::FLAG_ONEWAY)==OK&&WaitForSingleObject(begun,3000)==WAIT_OBJECT_0;}
 if(ok){for(uint32_t code:{2u,3u}){Parcel in;in.markForRpc(client);ok=ok&&root->transact(code,in,nullptr,IBinder::FLAG_ONEWAY)==OK;}Parcel in,out;in.markForRpc(client);int32_t marker=0;ok=ok&&root->transact(4,in,&out,0)==OK&&out.readInt32(&marker)==OK&&marker==1;}
 HANDLE inheritedOriginal=nullptr;if(ok){ok=DuplicateHandle(GetCurrentProcess(),service.process,GetCurrentProcess(),&inheritedOriginal,PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,TRUE,0)!=FALSE;if(ok){ok=spawn(L"client "+endpoint+L" "+value(inheritedOriginal),{inheritedOriginal},&other);CloseHandle(inheritedOriginal);if(ok)ok=reap(other,"delegated-original-cap-client");}}
 SetEvent(releaseFirst);if(ok)ok=WaitForSingleObject(secondDone,3000)==WAIT_OBJECT_0;root.clear();if(client){bool down=closeClient(client,"main");ok=ok&&down;}SetEvent(stopServer);bool otherEnd=reap(other,"delegated-original-cap-client");bool serviceEnd=reap(service,"service");for(HANDLE h:{ready,stopServer,begun,releaseFirst,secondDone})CloseHandle(h);printf("CROSS_PROCESS_INITIAL_TASK_AND_RESOURCE_ASSERTIONS=%s protocol_health_NOT_INFERRED_from_cleanup=1 sticky_status_preserved=1 independent_CLI_not_tested=1\n",ok&&otherEnd&&serviceEnd?"PASS":"FAIL");return ok&&otherEnd&&serviceEnd?0:1;
}
