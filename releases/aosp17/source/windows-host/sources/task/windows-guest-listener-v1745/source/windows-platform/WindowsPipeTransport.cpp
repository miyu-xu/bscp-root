#include "WindowsPipeTransport.h"
#include "FdTrigger.h"
#include <algorithm>
#include <cerrno>
#include <new>
#include <log/log.h>
namespace android {
namespace {
status_t pipeError(DWORD e) {
    switch(e) {
        case ERROR_BROKEN_PIPE: case ERROR_PIPE_NOT_CONNECTED: case ERROR_NO_DATA:
        case ERROR_OPERATION_ABORTED: return DEAD_OBJECT;
        case ERROR_ACCESS_DENIED: return PERMISSION_DENIED;
        case ERROR_INVALID_HANDLE: return BAD_TYPE;
        case ERROR_INVALID_PARAMETER: return BAD_VALUE;
        case ERROR_NOT_ENOUGH_MEMORY: case ERROR_OUTOFMEMORY: return NO_MEMORY;
        default: return UNKNOWN_ERROR;
    }
}
struct Event {
    HANDLE value=CreateEventW(nullptr,TRUE,FALSE,nullptr);
    ~Event(){if(value) CloseHandle(value);}
};
struct Active {
    std::atomic<bool>& busy; std::atomic<bool>& waiting;
    ~Active(){waiting.store(false,std::memory_order_release);busy.store(false,std::memory_order_release);}
};
}
std::unique_ptr<RpcTransportWindowsPipe> RpcTransportWindowsPipe::adoptOverlapped(HANDLE pipe) {
    if(!pipe || pipe==INVALID_HANDLE_VALUE)return nullptr;
    if(bscp::windows::validateRawOverlappedPipeHandle(pipe)) {
        bscp::windows::releaseOrRetainPipe(pipe);return nullptr;
    }
    auto* transport=new(std::nothrow) RpcTransportWindowsPipe(pipe);
    if(!transport)bscp::windows::releaseOrRetainPipe(pipe);
    return std::unique_ptr<RpcTransportWindowsPipe>(transport);
}
std::unique_ptr<RpcTransportWindowsPipe> RpcTransportWindowsPipe::fromRegistered(bscp::windows::OverlappedPipeLease owner){
    if(bscp::windows::validateRawOverlappedPipe(owner))return nullptr;
    return std::unique_ptr<RpcTransportWindowsPipe>(new(std::nothrow) RpcTransportWindowsPipe(std::move(owner)));
}
RpcTransportWindowsPipe::~RpcTransportWindowsPipe(){
    // No RpcTransport implementation permits destruction concurrent with a member call.
    // Every transfer drains its precise OVERLAPPED before returning.
    LOG_ALWAYS_FATAL_IF(mBusy.load(),"destroying named-pipe transport during I/O");
    if(!mOwner)bscp::windows::releaseOrRetainPipe(mPipe);
    // Registered owner retains HANDLE and reserved descriptor slot until this
    // final lease ends, including close-failure quarantine. No CRT pipe I/O.
}
status_t RpcTransportWindowsPipe::pollRead(){
    bool expected=false;if(!mBusy.compare_exchange_strong(expected,true))return INVALID_OPERATION;
    Active active{mBusy,mWaiting};DWORD available=0;
    if(!PeekNamedPipe(mPipe,nullptr,0,nullptr,&available,nullptr))return pipeError(GetLastError());
    return available?OK:WOULD_BLOCK;
}
status_t RpcTransportWindowsPipe::interruptableWriteFully(FdTrigger* trigger,iovec* iovs,int count,
    const std::optional<binder::impl::SmallFunction<status_t()>>& altPoll,
    const std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>* fds){
    if(fds&&!fds->empty())return -EOPNOTSUPP;
    return transfer(true,trigger,iovs,count,altPoll);
}
status_t RpcTransportWindowsPipe::interruptableReadFully(FdTrigger* trigger,iovec* iovs,int count,
    const std::optional<binder::impl::SmallFunction<status_t()>>& altPoll,
    std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*){
    return transfer(false,trigger,iovs,count,altPoll);
}
status_t RpcTransportWindowsPipe::transfer(bool write,FdTrigger* trigger,iovec* iovs,int count,
    const std::optional<binder::impl::SmallFunction<status_t()>>& altPoll){
    if(!trigger||count<0||(!iovs&&count))return BAD_VALUE;
    for(int i=0;i<count;++i)if(!iovs[i].iov_base&&iovs[i].iov_len)return BAD_VALUE;
    bool expected=false;if(!mBusy.compare_exchange_strong(expected,true))return INVALID_OPERATION;
    Active active{mBusy,mWaiting};auto cancellation=trigger->cancellation();
    if(!cancellation.valid())return UNKNOWN_ERROR;
    for(int i=0;i<count;++i)while(iovs[i].iov_len){
        auto state=cancellation.state();
        if(state.outcome!=bscp::windows::Cancellation::Outcome::ready)
            return state.outcome==bscp::windows::Cancellation::Outcome::cancelled?DEAD_OBJECT:UNKNOWN_ERROR;
        Event event;if(!event.value)return NO_MEMORY;
        OVERLAPPED operation{};operation.hEvent=event.value;
        DWORD bytes=0;const DWORD requested=static_cast<DWORD>(std::min<size_t>(iovs[i].iov_len,16*1024*1024));
        BOOL completed=write?WriteFile(mPipe,iovs[i].iov_base,requested,&bytes,&operation):
                             ReadFile(mPipe,iovs[i].iov_base,requested,&bytes,&operation);
        DWORD error=completed?ERROR_SUCCESS:GetLastError();
        if(!completed && error==ERROR_IO_PENDING){
            mWaiting.store(true,std::memory_order_release);
            status_t interrupted=OK;
            try { for(;;){
                auto outcome=cancellation.state();
                if(outcome.outcome!=bscp::windows::Cancellation::Outcome::ready){
                    interrupted=outcome.outcome==bscp::windows::Cancellation::Outcome::cancelled?DEAD_OBJECT:UNKNOWN_ERROR;break;
                }
                DWORD ready=WaitForSingleObject(event.value,0);
                if(ready==WAIT_OBJECT_0)break;
                if(ready!=WAIT_TIMEOUT){interrupted=UNKNOWN_ERROR;break;}
                if(altPoll){interrupted=(*altPoll)();if(interrupted!=OK)break;}
                else {
                    auto wait=cancellation.wait(event.value,INFINITE);
                    if(wait.outcome==bscp::windows::Cancellation::Outcome::ready)break;
                    interrupted=wait.outcome==bscp::windows::Cancellation::Outcome::cancelled?DEAD_OBJECT:UNKNOWN_ERROR;break;
                }
            }
            } catch (...) {
                // altPoll is caller code and may throw while the kernel owns
                // the buffer. Convert only after requesting precise cancel and
                // draining this same OVERLAPPED below; never unwind it pending.
                interrupted=UNKNOWN_ERROR;
            }
            if(interrupted!=OK){
                // Exactly this request, once; never cancel unrelated I/O. ERROR_NOT_FOUND
                // means a completion race, not proof that the operation has completed.
                if(!CancelIoEx(mPipe,&operation) && GetLastError()!=ERROR_NOT_FOUND)
                    interrupted=UNKNOWN_ERROR;
            }
            // CancelIoEx does not complete I/O. Keep original handle, event, OVERLAPPED
            // and caller buffer alive until the kernel reports completion. Not a hard SLA.
            completed=GetOverlappedResult(mPipe,&operation,&bytes,TRUE);
            error=completed?ERROR_SUCCESS:GetLastError();
            mWaiting.store(false,std::memory_order_release);
            if(interrupted!=OK)return interrupted;
        }
        if(!completed)return pipeError(error);
        auto finalState=cancellation.state();
        if(finalState.outcome!=bscp::windows::Cancellation::Outcome::ready)
            return finalState.outcome==bscp::windows::Cancellation::Outcome::cancelled?DEAD_OBJECT:UNKNOWN_ERROR;
        if(!bytes)return DEAD_OBJECT;
        if(bytes>requested)return UNKNOWN_ERROR;
        iovs[i].iov_base=static_cast<char*>(iovs[i].iov_base)+bytes;iovs[i].iov_len-=bytes;
    }
    return OK;
}
}
