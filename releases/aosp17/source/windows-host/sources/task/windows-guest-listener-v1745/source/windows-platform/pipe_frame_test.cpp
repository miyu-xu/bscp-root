#include "WindowsPipeFrame.h"
#include "FdTrigger.h"
#include <thread>
#include <chrono>
#include <condition_variable>
#include <cstdio>
#include <cstring>
#include <cstdlib>
using namespace android;
struct Pair {std::unique_ptr<RpcTransportWindowsPipe> a,b;};
Pair pair(){
 static unsigned seq=0;std::wstring name=L"\\\\.\\pipe\\bscp-a17-pipe-test-"+std::to_wstring(GetCurrentProcessId())+L"-"+std::to_wstring(++seq);
 HANDLE a=CreateNamedPipeW(name.c_str(),PIPE_ACCESS_DUPLEX|FILE_FLAG_OVERLAPPED|FILE_FLAG_FIRST_PIPE_INSTANCE,
     PIPE_TYPE_BYTE|PIPE_READMODE_BYTE|PIPE_WAIT|PIPE_REJECT_REMOTE_CLIENTS,1,65536,65536,0,nullptr);
 if(a==INVALID_HANDLE_VALUE)return {};
 HANDLE ev=CreateEventW(nullptr,TRUE,FALSE,nullptr);OVERLAPPED ov{};ov.hEvent=ev;
 if(!ev){CloseHandle(a);return {};}
 BOOL connected=ConnectNamedPipe(a,&ov);DWORD e=connected?ERROR_SUCCESS:GetLastError();
 if(!connected&&e!=ERROR_IO_PENDING&&e!=ERROR_PIPE_CONNECTED){CloseHandle(ev);CloseHandle(a);return {};}
 HANDLE b=CreateFileW(name.c_str(),GENERIC_READ|GENERIC_WRITE,0,nullptr,OPEN_EXISTING,FILE_FLAG_OVERLAPPED,nullptr);
 if(b==INVALID_HANDLE_VALUE){if(e==ERROR_IO_PENDING){CancelIoEx(a,&ov);DWORD n;GetOverlappedResult(a,&ov,&n,TRUE);}CloseHandle(ev);CloseHandle(a);return {};}
 if(e==ERROR_IO_PENDING){DWORD n;if(!GetOverlappedResult(a,&ov,&n,TRUE)){CloseHandle(b);CloseHandle(ev);CloseHandle(a);return {};}}
 CloseHandle(ev);return {RpcTransportWindowsPipe::adoptOverlapped(a),RpcTransportWindowsPipe::adoptOverlapped(b)};
}
int test(){
 {auto p=pair();if(!p.a||!p.b)return 1;auto t=FdTrigger::make();RpcTransportWindowsPipeFrame frame(std::move(p.a));
  char value[]="A17";iovec w{value,3};if(frame.interruptableWriteFully(t.get(),&w,1,{},nullptr)!=OK)return 2;
  unsigned char wire[11]{};iovec r{wire,sizeof(wire)};if(p.b->interruptableReadFully(t.get(),&r,1,{},nullptr)!=OK)return 3;
  const unsigned char expected[]={3,0,0,0,0,0,0,0,'A','1','7'};if(std::memcmp(wire,expected,sizeof(wire)))return 4;
  unsigned char incoming[]={2,0,0,0,0,0,0,0,'a','b',3,0,0,0,0,0,0,0,'c','d','e'};w={incoming,sizeof(incoming)};
  if(p.b->interruptableWriteFully(t.get(),&w,1,{},nullptr)!=OK)return 5;
  char out[5]{};iovec split[2]={{out,1},{out+1,4}};if(frame.interruptableReadFully(t.get(),split,2,{},nullptr)!=OK||std::memcmp(out,"abcde",5))return 6;
  std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>> fds;fds.emplace_back(binder::borrowed_fd{-1});w={value,3};
  if(frame.interruptableWriteFully(t.get(),&w,1,{},&fds)!=-EOPNOTSUPP||p.b->pollRead()!=WOULD_BLOCK||frame.poisoned())return 7;
 }
 {auto p=pair();if(!p.a||!p.b)return 8;auto t=FdTrigger::make();RpcTransportWindowsPipeFrame frame(std::move(p.a));
  HANDLE canary=CreateEventW(nullptr,TRUE,FALSE,nullptr);if(!canary)return 9;
  unsigned char wire[17]={1,0,0,0,1,0,0,0};uint64_t forged=reinterpret_cast<uintptr_t>(canary);std::memcpy(wire+8,&forged,8);wire[16]=42;
  iovec w{wire,sizeof(wire)};if(p.b->interruptableWriteFully(t.get(),&w,1,{},nullptr)!=OK)return 10;
  char out;iovec r{&out,1};auto result=frame.interruptableReadFully(t.get(),&r,1,{},nullptr);DWORD flags=0;bool alive=GetHandleInformation(canary,&flags);CloseHandle(canary);
  if(result!=-EOPNOTSUPP||!alive||!frame.poisoned()||frame.pollRead()!=DEAD_OBJECT)return 11;
 }
 {auto p=pair();if(!p.a||!p.b)return 12;auto t=FdTrigger::make();RpcTransportWindowsPipeFrame frame(std::move(p.a));
  unsigned char truncated[]={1,0,0,0};iovec w{truncated,sizeof(truncated)};if(p.b->interruptableWriteFully(t.get(),&w,1,{},nullptr)!=OK)return 13;
  p.b.reset();char out;iovec r{&out,1};if(frame.interruptableReadFully(t.get(),&r,1,{},nullptr)!=DEAD_OBJECT||!frame.poisoned())return 14;
 }
 {auto p=pair();if(!p.a||!p.b)return 15;auto t=FdTrigger::make();RpcTransportWindowsPipeFrame frame(std::move(p.a));
  unsigned char wire[]={1,0,0,4,0,0,0,0};iovec w{wire,sizeof(wire)};if(p.b->interruptableWriteFully(t.get(),&w,1,{},nullptr)!=OK)return 16;
  char out;iovec r{&out,1};if(frame.interruptableReadFully(t.get(),&r,1,{},nullptr)!=BAD_VALUE||!frame.poisoned())return 17;
 }
 {auto p=pair();if(!p.a||!p.b)return 18;auto t=FdTrigger::make();RpcTransportWindowsPipeFrame frame(std::move(p.a));
  unsigned char wire[]={4,0,0,0,0,0,0,0,'a','b'};iovec w{wire,sizeof(wire)};if(p.b->interruptableWriteFully(t.get(),&w,1,{},nullptr)!=OK)return 19;
  char out[4]{};iovec r{out,4};status_t result=OK;std::thread reader([&]{result=frame.interruptableReadFully(t.get(),&r,1,{},nullptr);});
  auto limit=std::chrono::steady_clock::now()+std::chrono::seconds(2);while(!frame.isWaiting()&&std::chrono::steady_clock::now()<limit)std::this_thread::sleep_for(std::chrono::milliseconds(1));
  t->trigger();reader.join();if(result!=DEAD_OBJECT||!frame.poisoned()||frame.isWaiting())return 20;
 }
 std::puts("A17_PIPE_LEGACY_ZERO_FD_FRAME_CROSS_BOUNDARY_TRUNCATION_CANCEL_FORGED_HANDLE_REJECT_PASS");return 0;
}
int main(){std::mutex m;std::condition_variable cv;bool done=false;
 std::thread deadline([&]{std::unique_lock<std::mutex> l(m);if(!cv.wait_for(l,std::chrono::seconds(15),[&]{return done;}))std::_Exit(90);});
 int r=test();{std::lock_guard<std::mutex> l(m);done=true;}cv.notify_one();deadline.join();return r;
}
