#include "WindowsSocketOwner.h"
#include "FdTrigger.h"
#include <log/log.h>
#include <binder/Functional.h>
namespace android {
std::unique_ptr<FdTrigger> FdTrigger::make(){
    auto value=std::make_unique<FdTrigger>();
    if(!value->mCancellation.valid())return nullptr;
    return value;
}
void FdTrigger::trigger(){LOG_ALWAYS_FATAL_IF(!mCancellation.trigger(),"Windows cancellation SetEvent failed");}
bool FdTrigger::isTriggered(){return mCancellation.state().outcome!=bscp::windows::Cancellation::Outcome::ready;}
status_t FdTrigger::triggerablePoll(const RpcTransportFd& fd,int16_t event){
    using O=bscp::windows::Cancellation::Outcome;
    if(event!=POLLIN && event!=POLLOUT)return BAD_VALUE;
    if(isTriggered())return DEAD_OBJECT;
    auto owner=bscp::windows::acquireSocket(fd.fd.get());
    if(!owner)return BAD_TYPE; // Never reinterpret a CRT descriptor as a native socket.
    std::unique_lock<std::mutex> nativePoll(owner->shared->pollingMutex,std::try_to_lock);
    if(!nativePoll.owns_lock())return INVALID_OPERATION;
    LOG_ALWAYS_FATAL_IF(fd.isInPollingState(),"Concurrent polling on one RPC transport");
    fd.setPollingState(true);
    auto stateGuard=binder::impl::make_scope_guard([&]{fd.setPollingState(false);});
    WSAEVENT ready=WSACreateEvent();
    if(ready==WSA_INVALID_EVENT)return NO_MEMORY;
    bool registered=false;
    // Resolve event association before returning any success. Failed unregistration retains
    // the original event with the socket owner; it is closed only after the socket is closed.
    auto finish=[&](status_t status) -> status_t {
        if(registered && WSAEventSelect(owner->socket,nullptr,0)==SOCKET_ERROR){
            owner->shared->retainedEvents.push_back(ready);
            ready=WSA_INVALID_EVENT;registered=false;
            return UNKNOWN_ERROR;
        }
        registered=false;
        if(ready!=WSA_INVALID_EVENT){WSACloseEvent(ready);ready=WSA_INVALID_EVENT;}
        return status;
    };
    long mask=(event==POLLIN?FD_READ|FD_ACCEPT:FD_WRITE|FD_CONNECT)|FD_CLOSE;
    if(WSAEventSelect(owner->socket,ready,mask)==SOCKET_ERROR)return finish(UNKNOWN_ERROR);
    registered=true;
    for(;;){
        // Poll returns current readiness; FD_CLOSE alone must not discard buffered bytes.
        WSAPOLLFD pollfd{owner->socket,event,0};
        int n=WSAPoll(&pollfd,1,0);
        if(n==SOCKET_ERROR)return finish(UNKNOWN_ERROR);
        if(isTriggered())return finish(DEAD_OBJECT);
        if(n){
            if(pollfd.revents&POLLNVAL)return finish(BAD_VALUE);
            if(pollfd.revents&event)return finish(OK);
            if(pollfd.revents&(POLLERR|POLLHUP))return finish(DEAD_OBJECT);
            return finish(UNKNOWN_ERROR);
        }
        auto outcome=mCancellation.wait(ready,INFINITE);
        if(outcome.outcome==O::cancelled)return finish(DEAD_OBJECT);
        if(outcome.outcome!=O::ready)return finish(UNKNOWN_ERROR);
        WSANETWORKEVENTS observed{};
        if(WSAEnumNetworkEvents(owner->socket,ready,&observed)==SOCKET_ERROR)return finish(UNKNOWN_ERROR);
        for(int i=0;i<FD_MAX_EVENTS;++i){
            if((observed.lNetworkEvents&(1L<<i)) && observed.iErrorCode[i])return finish(DEAD_OBJECT);
        }
        if(observed.lNetworkEvents&FD_CLOSE){
            WSAPOLLFD finalPoll{owner->socket,event,0};
            if(WSAPoll(&finalPoll,1,0)==SOCKET_ERROR)return finish(UNKNOWN_ERROR);
            return finish(finalPoll.revents&event?OK:DEAD_OBJECT);
        }
    }
}
}
