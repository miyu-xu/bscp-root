#include "WindowsOverlappedPipeFd.h"
#include <atomic>
#include <mutex>
#include <unordered_map>
#include <io.h>
#include <fcntl.h>
#include <cerrno>
#include <cstring>
#include <new>
#include <cstdio>
#include <exception>
namespace bscp::windows {
namespace {
constexpr DWORD synchronousModes=0x10u|0x20u;
// Registry/retention lock failure cannot be interpreted as "not registered",
// which would misroute an owned pipe slot to CRT. Fail fast, never cross FFI.
struct Lock {std::mutex& mutex;explicit Lock(std::mutex& m):mutex(m){try{mutex.lock();}catch(...){RaiseFailFastException(nullptr,nullptr,0);std::terminate();}}~Lock(){mutex.unlock();}};
struct IoStatus {union {LONG status;void* pointer;};ULONG_PTR information;};
static_assert(sizeof(IoStatus)==2*sizeof(void*));
using NativeQuery=LONG(NTAPI*)(HANDLE,IoStatus*,void*,ULONG,ULONG);
struct QueryOwner {HANDLE handle=nullptr;IoStatus io{};DWORD value=0;QueryOwner* next=nullptr;};
std::mutex queryMutex;
QueryOwner* retainedQueries=nullptr;
void retainQuery(QueryOwner* q){Lock lock(queryMutex);q->next=retainedQueries;retainedQueries=q;std::fputs("Native pipe query/close outcome UNKNOWN; exact handle and buffers retained\n",stderr);}
DWORD releaseQuery(QueryOwner* q){if(q->handle&&!CloseHandle(q->handle)){DWORD e=GetLastError();retainQuery(q);return e;}delete q;return 0;}
DWORD queryValue(HANDLE input,ULONG kind,DWORD* value){
 HMODULE module=GetModuleHandleW(L"ntdll.dll");if(!module)return ERROR_MOD_NOT_FOUND;FARPROC symbol=GetProcAddress(module,"NtQueryInformationFile");if(!symbol)return ERROR_PROC_NOT_FOUND;NativeQuery query=nullptr;static_assert(sizeof(query)==sizeof(symbol));std::memcpy(&query,&symbol,sizeof(query));
 auto* q=new(std::nothrow) QueryOwner;if(!q)return ERROR_NOT_ENOUGH_MEMORY;
 if(!DuplicateHandle(GetCurrentProcess(),input,GetCurrentProcess(),&q->handle,0,FALSE,DUPLICATE_SAME_ACCESS)){DWORD e=GetLastError();delete q;return e;}
 LONG status=query(q->handle,&q->io,&q->value,sizeof(q->value),kind);
 // A pending native query has no OVERLAPPED event we own. Do not free its
 // buffers, close its original duplicate, or pretend that error cancels it.
 if(status==0x103){retainQuery(q);return ERROR_IO_PENDING;}
 DWORD e=(status==0&&q->io.status==0&&q->io.information==sizeof(q->value))?0:ERROR_INVALID_DATA;
 DWORD result=q->value,closed=releaseQuery(q);if(closed)return closed;if(e)return e;*value=result;return 0;
}
struct CloseState {std::atomic<DWORD> error{0};};
}
struct OverlappedPipeOwner {HANDLE handle=nullptr;int slot=-1;bool registered=false;std::shared_ptr<CloseState> close=std::make_shared<CloseState>();OverlappedPipeOwner* retainedNext=nullptr;};
namespace {
std::mutex pipeMutex,retainedMutex;
struct PipeSlot {OverlappedPipeLease active;};
// This map entry is allocated at registration. Empty active is a tombstone
// retained until final native close + reserved CRT slot close both succeed.
// Process-lifetime namespace: do not run map destructors which would recursively
// erase entries while active owners are being destroyed at CRT teardown. Normal
// explicit close empties entries; UNKNOWN owners remain retained until process exit.
std::unordered_map<int,PipeSlot>& pipes=*new std::unordered_map<int,PipeSlot>;
OverlappedPipeOwner* retainedPipes=nullptr;
void retainPipe(OverlappedPipeOwner* p,DWORD e){p->close->error=e;Lock lock(retainedMutex);p->retainedNext=retainedPipes;retainedPipes=p;std::fputs("Native pipe close outcome UNKNOWN; original HANDLE/slot retained\n",stderr);}
struct ReleaseOwner {void operator()(OverlappedPipeOwner* p)const noexcept {
 if(p->handle&&!CloseHandle(p->handle)){retainPipe(p,GetLastError());return;}p->handle=nullptr;
 {Lock lock(pipeMutex);
  if(p->slot>=0&&_close(p->slot)){retainPipe(p,ERROR_INVALID_HANDLE);return;}
  if(p->registered)pipes.erase(p->slot);p->slot=-1;
 }delete p;
}};
int errorCode(DWORD e){return e==ERROR_NOT_ENOUGH_MEMORY?-ENOMEM:e==ERROR_ACCESS_DENIED?-EACCES:e==ERROR_INVALID_HANDLE?-EBADF:-EIO;}
}
DWORD inspectNativePipe(HANDLE h,PipeSnapshot* out){if(!out)return ERROR_INVALID_PARAMETER;*out={};DWORD flags=0;if(!h||h==INVALID_HANDLE_VALUE||!GetHandleInformation(h,&flags))return ERROR_INVALID_HANDLE;
 if(GetFileType(h)!=FILE_TYPE_PIPE)return ERROR_INVALID_HANDLE;
 PipeSnapshot s; if(!GetNamedPipeInfo(h,&s.pipeFlags,nullptr,nullptr,nullptr))return GetLastError();
 DWORD e=queryValue(h,8,&s.access);if(!e)e=queryValue(h,16,&s.mode);if(e)return e;
 if(!GetNamedPipeHandleStateW(h,&s.pipeState,nullptr,nullptr,nullptr,nullptr,0))return GetLastError();*out=s;return 0;
}
DWORD validateRawOverlappedPipeHandle(HANDLE h){PipeSnapshot s;DWORD e=inspectNativePipe(h,&s);if(e)return e;DWORD flags=0;if(!GetHandleInformation(h,&flags))return GetLastError();
 if((flags&HANDLE_FLAG_INHERIT)||(s.mode&synchronousModes)||(s.access&3u)!=3u||(s.pipeFlags&PIPE_TYPE_MESSAGE)||(s.pipeState&(PIPE_READMODE_MESSAGE|PIPE_NOWAIT)))return ERROR_INVALID_PARAMETER;
 // Only after proving async: Peek is a non-consuming connected-state check.
 DWORD available=0;if(!PeekNamedPipe(h,nullptr,0,nullptr,&available,nullptr))return GetLastError();return 0;
}
OverlappedPipeLease acquireOverlappedPipe(int fd){Lock lock(pipeMutex);auto i=pipes.find(fd);return i==pipes.end()?nullptr:i->second.active;}
bool isOverlappedPipeSlot(int fd){Lock lock(pipeMutex);return pipes.find(fd)!=pipes.end();}
HANDLE borrowedOverlappedPipe(const OverlappedPipeLease& p){return p?p->handle:nullptr;}
DWORD validateRawOverlappedPipe(const OverlappedPipeLease& p){return p?validateRawOverlappedPipeHandle(p->handle):ERROR_INVALID_HANDLE;}
int registerOverlappedPipe(HANDLE input,int* out){if(!out)return -EINVAL;*out=-1;PipeSnapshot snapshot;DWORD e=inspectNativePipe(input,&snapshot);if(e)return errorCode(e);if(snapshot.mode&synchronousModes)return -EOPNOTSUPP;
 try {std::unique_ptr<OverlappedPipeOwner,ReleaseOwner> owner(new OverlappedPipeOwner);
  // Reserve a shared descriptor number only. The real pipe HANDLE never enters
  // the CRT. This prevents collisions with both files and socket registry slots.
  owner->slot=_open("NUL",_O_RDWR|_O_BINARY|_O_NOINHERIT);if(owner->slot<0)return -errno;
  if(!DuplicateHandle(GetCurrentProcess(),input,GetCurrentProcess(),&owner->handle,0,FALSE,DUPLICATE_SAME_ACCESS))return errorCode(GetLastError());
  e=inspectNativePipe(owner->handle,&snapshot);if(e)return errorCode(e);if(snapshot.mode&synchronousModes)return -EOPNOTSUPP;
  OverlappedPipeLease shared(std::move(owner));int slot=shared->slot;{Lock lock(pipeMutex);if(!pipes.emplace(slot,PipeSlot{shared}).second)return -EIO;shared->registered=true;}*out=slot;return 0;
 }catch(const std::bad_alloc&){return -ENOMEM;}catch(...){return -EIO;}
}
int copyOverlappedPipe(const OverlappedPipeLease& p,HANDLE* out){if(!out)return -EINVAL;*out=nullptr;if(!p)return -EBADF;
 if(!DuplicateHandle(GetCurrentProcess(),p->handle,GetCurrentProcess(),out,0,FALSE,DUPLICATE_SAME_ACCESS))return errorCode(GetLastError());return 0;
}
int duplicateOverlappedPipe(const OverlappedPipeLease& p,int* out){if(!p){if(out)*out=-1;return -EBADF;}return registerOverlappedPipe(p->handle,out);}
bool closeOverlappedPipe(int fd,int* result){OverlappedPipeLease released;{Lock lock(pipeMutex);auto i=pipes.find(fd);if(i==pipes.end())return false;if(!i->second.active){if(result)*result=-EBADF;return true;}released=std::move(i->second.active);}auto close=released->close;released.reset();if(result)*result=close->error?errorCode(close->error):0;return true;}
DWORD closeOrRetainPipe(HANDLE handle){if(!handle||handle==INVALID_HANDLE_VALUE)return ERROR_INVALID_HANDLE;if(CloseHandle(handle))return 0;DWORD e=GetLastError();
 // No kernel I/O is pending on this adoption-error path. If retaining ownership
 // metadata cannot be allocated, fail fast rather than silently lose the HANDLE.
 try {auto* p=new OverlappedPipeOwner;p->handle=handle;retainPipe(p,e);}catch(...){RaiseFailFastException(nullptr,nullptr,0);}
 return e;
}
void releaseOrRetainPipe(HANDLE handle){(void)closeOrRetainPipe(handle);}
}
extern "C" int bscp_fd_from_overlapped_pipe_handle(HANDLE input,int* out){try{return bscp::windows::registerOverlappedPipe(input,out);}catch(const std::bad_alloc&){if(out)*out=-1;return -ENOMEM;}catch(...){if(out)*out=-1;return -EIO;}}
