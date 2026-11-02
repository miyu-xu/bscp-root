#include <binder/RpcTransportRaw.h>
#include "WindowsOverlappedPipeFd.h"
#include "WindowsFdAbi.h"
#include "WindowsSocketApi.h"
#include "FdTrigger.h"
#include "OS.h"
#include <thread>
#include <atomic>
#include <cstdio>
#include <cstring>
#include <string>
#include <cerrno>
#include <stdexcept>
using namespace android;
struct H {HANDLE h=nullptr;~H(){if(h&&h!=INVALID_HANDLE_VALUE)CloseHandle(h);}HANDLE take(){HANDLE v=h;h=nullptr;return v;}};
#define NEED(x) do{if(!(x)){std::fprintf(stderr,"FAIL line=%d win=%lu errno=%d\n",__LINE__,GetLastError(),errno);return 1;}}while(0)
static unsigned serial=0;
static bool pair(H& server,H& client,bool async=true,bool message=false){
 std::wstring name=LR"(\\.\pipe\bscp-guest-raw-)"+std::to_wstring(GetCurrentProcessId())+L"-"+std::to_wstring(++serial);
 server.h=CreateNamedPipeW(name.c_str(),PIPE_ACCESS_DUPLEX|(async?FILE_FLAG_OVERLAPPED:0),PIPE_WAIT|(message?PIPE_TYPE_MESSAGE|PIPE_READMODE_MESSAGE:PIPE_TYPE_BYTE|PIPE_READMODE_BYTE),1,4096,4096,0,nullptr);
 if(server.h==INVALID_HANDLE_VALUE)return false;
 client.h=CreateFileW(name.c_str(),GENERIC_READ|GENERIC_WRITE,0,nullptr,OPEN_EXISTING,FILE_FLAG_OVERLAPPED,nullptr);
 if(client.h==INVALID_HANDLE_VALUE)return false;
 H event;event.h=CreateEventW(nullptr,TRUE,FALSE,nullptr);if(!event.h)return false;OVERLAPPED ov{};ov.hEvent=event.h;
 BOOL ok=ConnectNamedPipe(server.h,async?&ov:nullptr);DWORD e=ok?0:GetLastError();
 if(e==ERROR_IO_PENDING){DWORD n=0;return GetOverlappedResult(server.h,&ov,&n,TRUE)!=FALSE;}
 return ok||e==ERROR_PIPE_CONNECTED;
}
static bool io(HANDLE h,bool write,void* data,DWORD size,DWORD* error=nullptr){
 H event;event.h=CreateEventW(nullptr,TRUE,FALSE,nullptr);if(!event.h)return false;OVERLAPPED ov{};ov.hEvent=event.h;DWORD n=0;
 BOOL done=write?WriteFile(h,data,size,&n,&ov):ReadFile(h,data,size,&n,&ov);DWORD e=done?0:GetLastError();
 if(!done&&e==ERROR_IO_PENDING){DWORD w=WaitForSingleObject(event.h,3000);if(w!=WAIT_OBJECT_0){if(!CancelIoEx(h,&ov)&&GetLastError()!=ERROR_NOT_FOUND)std::fputs("precise helper cancel failed\n",stderr);GetOverlappedResult(h,&ov,&n,TRUE);if(error)*error=ERROR_TIMEOUT;return false;}done=GetOverlappedResult(h,&ov,&n,TRUE);e=done?0:GetLastError();}
 if(error)*error=e;return done&&n==size;
}
static std::unique_ptr<RpcTransport> raw(int fd){auto factory=RpcTransportCtxFactoryRaw::make();auto ctx=factory->newClientCtx();return ctx->newTransport(RpcTransportFd{binder::unique_fd(fd)},nullptr);}
int run(){
 H a,b;NEED(pair(a,b));int fd=-1;NEED(!bscp_fd_from_handle(a.h,&fd));NEED(bscp::windows::isOverlappedPipeSlot(fd));NEED(binder::os::setNonBlocking(binder::borrowed_fd(fd))==OK);
 auto observed=bscp::windows::acquireOverlappedPipe(fd);NEED(observed);HANDLE original=bscp::windows::borrowedOverlappedPipe(observed);observed.reset();
 int duplicate=-1;NEED(!bscp_fd_dup(fd,&duplicate)&&duplicate!=fd);NEED(!bscp_fd_close(duplicate));
 H event;event.h=CreateEventW(nullptr,TRUE,FALSE,nullptr);int invalid=-1;NEED(bscp_fd_from_overlapped_pipe_handle(event.h,&invalid)!=0&&invalid==-1);
 std::wstring filename=L"guest-raw-file-"+std::to_wstring(GetCurrentProcessId());H file;file.h=CreateFileW(filename.c_str(),GENERIC_READ|GENERIC_WRITE,0,nullptr,CREATE_NEW,FILE_FLAG_DELETE_ON_CLOSE,nullptr);NEED(file.h!=INVALID_HANDLE_VALUE);int disk=-1;NEED(!bscp_fd_from_handle(file.h,&disk)&&disk!=fd);NEED(!raw(disk));
 int sockets[2];NEED(!bscp::windows::socketpair(AF_UNIX,SOCK_STREAM,0,sockets));NEED(sockets[0]!=fd&&sockets[1]!=fd);auto socketTransport=raw(sockets[0]);NEED(socketTransport);NEED(!bscp_fd_close(sockets[1]));socketTransport.reset();
 auto transport=raw(fd);NEED(transport);NEED(!bscp::windows::acquireOverlappedPipe(fd)&&bscp::windows::isOverlappedPipeSlot(fd));
 NEED(bscp_fd_close(fd)==-EBADF);int nofd=123;NEED(bscp_fd_dup(fd,&nofd)==-EBADF);HANDLE noh=nullptr;NEED(bscp_fd_to_handle(fd,&noh)==-EBADF&&!noh);
 NEED(CloseHandle(a.take()));auto trigger=FdTrigger::make();NEED(trigger);
 char sent[]="A17",buffer[3]{};NEED(io(b.h,true,sent,3));iovec read{buffer,3};NEED(transport->interruptableReadFully(trigger.get(),&read,1,std::nullopt,nullptr)==OK&&!std::memcmp(buffer,sent,3));
 iovec write{sent,3};NEED(transport->interruptableWriteFully(trigger.get(),&write,1,std::nullopt,nullptr)==OK);std::memset(buffer,0,3);NEED(io(b.h,false,buffer,3)&&!std::memcmp(buffer,sent,3));
 bool altCalled=false;char thrownRead=0;iovec thrownVector{&thrownRead,1};
 std::optional<binder::impl::SmallFunction<status_t()>> throwing(std::in_place,[&]()->status_t{altCalled=true;throw std::runtime_error("fixture pending altPoll");});
 NEED(transport->interruptableReadFully(trigger.get(),&thrownVector,1,throwing,nullptr)==UNKNOWN_ERROR&&altCalled&&!transport->isWaiting());
 char check='Q';NEED(io(b.h,true,&check,1));iovec afterThrow{&thrownRead,1};NEED(transport->interruptableReadFully(trigger.get(),&afterThrow,1,std::nullopt,nullptr)==OK&&thrownRead=='Q');
 status_t cancelled=UNKNOWN_ERROR;char pending=0;std::thread reader([&]{iovec v{&pending,1};cancelled=transport->interruptableReadFully(trigger.get(),&v,1,std::nullopt,nullptr);});
 for(unsigned i=0;i<3000&&!transport->isWaiting();++i)Sleep(1);bool pendingObserved=transport->isWaiting();trigger->trigger();reader.join();NEED(pendingObserved&&cancelled==DEAD_OBJECT&&!transport->isWaiting());
 DWORD info=0;NEED(GetHandleInformation(original,&info));transport.reset();NEED(!GetHandleInformation(original,&info)&&!bscp::windows::isOverlappedPipeSlot(fd));DWORD eof=0;NEED(!io(b.h,false,&pending,1,&eof)&&(eof==ERROR_BROKEN_PIPE||eof==ERROR_PIPE_NOT_CONNECTED));
 for(bool message:{false,true}){H x,y;NEED(pair(x,y,message,message));int rejected=-1;if(message){NEED(!bscp_fd_from_overlapped_pipe_handle(x.h,&rejected));NEED(!raw(rejected));}else{NEED(bscp_fd_from_overlapped_pipe_handle(x.h,&rejected)!=0);NEED(!bscp_fd_from_handle(x.h,&rejected));NEED(!raw(rejected));}}
 std::puts("REAL_RAW_FACTORY_ASYNC_ROLE_TOMBSTONE_BIDIRECTIONAL_CANCEL_DRAIN_EOF_AND_REJECTIONS_PASS");return 0;
}
int main(){std::thread([]{Sleep(20000);std::fputs("SELF_DEADLINE_FAIL\n",stderr);ExitProcess(90);}).detach();return run();}
