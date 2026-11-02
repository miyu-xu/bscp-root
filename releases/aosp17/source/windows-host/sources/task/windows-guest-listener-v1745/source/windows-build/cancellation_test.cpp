#include "WindowsCancellation.h"
#include <thread>
#include <cstdio>
using bscp::windows::Cancellation;
using O=Cancellation::Outcome;
int main(){
    HANDLE completion=CreateEventW(nullptr,TRUE,FALSE,nullptr);
    if(!completion)return 1;
    Cancellation cancel;
    if(!cancel.valid() || cancel.state().outcome!=O::ready)return 2;
    if(cancel.wait(completion,1).outcome!=O::timeout)return 3;
    if(cancel.wait(nullptr,0).outcome!=O::error)return 4;
    if(!SetEvent(completion) || cancel.wait(completion,0).outcome!=O::ready)return 5;
    ResetEvent(completion);
    Cancellation::Result observed{O::error,0};
    std::thread waiter([copy=cancel,completion,&observed]{observed=copy.wait(completion,2000);});
    if(!cancel.trigger()){waiter.join();return 6;}
    waiter.join();
    if(observed.outcome!=O::cancelled)return 7;
    SetEvent(completion);
    if(cancel.wait(completion,0).outcome!=O::cancelled)return 8;
    // Dropping one owner does not invalidate the original event used by another owner.
    auto retained=cancel;cancel=Cancellation();
    if(retained.state().outcome!=O::cancelled || cancel.state().outcome!=O::ready)return 9;
    CloseHandle(completion);
    std::puts("WINDOWS_ORIGINAL_EVENT_CANCEL_TIMEOUT_LIFETIME_PASS");return 0;
}
