#include "WindowsPipeTransport.h"
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
 {auto p=pair();if(!p.a||!p.b)return 1;auto t=FdTrigger::make();if(!t)return 2;
  if(p.a->pollRead()!=WOULD_BLOCK)return 3;
  unsigned char sent[16]={0x81,0x72,0x63,0x54,0x45,0x36,0x27,0x18,0xfe,0xdc,0xba,0x98,0x76,0x54,0x32,0x10},got[16]{};
  iovec w{sent,sizeof(sent)},r{got,sizeof(got)};
  if(p.b->interruptableWriteFully(t.get(),&w,1,{},nullptr)!=OK || w.iov_len)return 4;
  if(p.a->pollRead()!=OK)return 5;
  if(p.a->interruptableReadFully(t.get(),&r,1,{},nullptr)!=OK || r.iov_len || std::memcmp(sent,got,16))return 6;
  std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>> fds;fds.emplace_back(binder::borrowed_fd{-1});
  w={sent,1};if(p.b->interruptableWriteFully(t.get(),&w,1,{},&fds)!=-EOPNOTSUPP || p.a->pollRead()!=WOULD_BLOCK)return 7;
  p.b.reset();r={got,1};if(p.a->interruptableReadFully(t.get(),&r,1,{},nullptr)!=DEAD_OBJECT)return 8;
 }
 {auto p=pair();if(!p.a||!p.b)return 9;auto t=FdTrigger::make();if(!t)return 10;
  char byte=0;iovec r{&byte,1};status_t result=OK;
  std::thread reader([&]{result=p.a->interruptableReadFully(t.get(),&r,1,{},nullptr);});
  auto deadline=std::chrono::steady_clock::now()+std::chrono::seconds(2);
  while(!p.a->isWaiting()&&std::chrono::steady_clock::now()<deadline)std::this_thread::sleep_for(std::chrono::milliseconds(1));
  bool wasWaiting=p.a->isWaiting();t->trigger();reader.join();
  if(!wasWaiting || result!=DEAD_OBJECT || p.a->isWaiting() || r.iov_len!=1)return 11;
 }
 {auto p=pair();if(!p.a||!p.b)return 12;auto t=FdTrigger::make();char byte=0;iovec r{&byte,1};
  std::optional<binder::impl::SmallFunction<status_t()>> fail{[]{return BAD_VALUE;}};
  if(p.a->interruptableReadFully(t.get(),&r,1,fail,nullptr)!=BAD_VALUE || p.a->isWaiting())return 13;
 }
 std::puts("A17_PIPE_BYTES_CANCEL_COMPLETION_EOF_ALTPOLL_FD_REJECT_PASS");return 0;
}
int main(){std::mutex m;std::condition_variable cv;bool done=false;
 std::thread deadline([&]{std::unique_lock<std::mutex> l(m);if(!cv.wait_for(l,std::chrono::seconds(15),[&]{return done;}))std::_Exit(90);});
 int r=test();{std::lock_guard<std::mutex> l(m);done=true;}cv.notify_one();deadline.join();return r;
}
