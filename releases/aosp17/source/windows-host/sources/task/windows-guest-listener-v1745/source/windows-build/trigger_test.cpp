#include "WindowsSocketApi.h"
#include "FdTrigger.h"
#include "event_fault_inject.h"
#include <condition_variable>
#include <thread>
#include <cstdio>
#include <cstdlib>
#include <chrono>
using namespace android;
int test(){
    int pair[2];if(bscp::windows::socketpair(AF_UNIX,SOCK_STREAM,0,pair))return 1;
    binder::unique_fd client(pair[0]);RpcTransportFd server{binder::unique_fd(pair[1])};
    auto trigger=FdTrigger::make();if(!trigger)return 2;
    status_t outcome=UNKNOWN_ERROR;
    std::thread waiter([&]{outcome=trigger->triggerablePoll(server,POLLIN);});
    std::this_thread::sleep_for(std::chrono::milliseconds(25));trigger->trigger();waiter.join();
    if(outcome!=DEAD_OBJECT || server.isInPollingState())return 3;
    trigger=FdTrigger::make();if(!trigger)return 4;
    if(bscp::windows::send(client.get(),"A17",3,0)!=3)return 5;
    if(trigger->triggerablePoll(server,POLLIN)!=OK)return 6;
    char data[3]{};if(bscp::windows::recv(server.fd.get(),data,3,0)!=3 || data[0]!='A'||data[2]!='7')return 7;
    client.reset();
    if(trigger->triggerablePoll(server,POLLIN)!=DEAD_OBJECT)return 8;
    int slot=server.fd.get();server.fd.reset();if(bscp::windows::acquireSocket(slot))return 9;
    int second[2];if(bscp::windows::socketpair(AF_UNIX,SOCK_STREAM,0,second))return 10;
    binder::unique_fd other(second[0]);RpcTransportFd target{binder::unique_fd(second[1])};
    auto retained=bscp::windows::acquireSocket(second[1]);
    trigger=FdTrigger::make();if(!trigger)return 11;
    if(bscp::windows::send(other.get(),"X",1,0)!=1)return 12;
    int aliasRaw=bscp::windows::duplicateSocket(second[1]);if(aliasRaw<0)return 17;
    binder::unique_fd alias(aliasRaw);
    failEventClear=true;outcome=trigger->triggerablePoll(target,POLLIN);failEventClear=false;
    if(outcome!=UNKNOWN_ERROR || retained->shared->retainedEvents.size()!=1)return 13;
    HANDLE event=retained->shared->retainedEvents[0];DWORD info=0;
    if(!GetHandleInformation(event,&info))return 14;
    target.fd.reset();if(!GetHandleInformation(event,&info))return 15;
    retained.reset();if(!GetHandleInformation(event,&info))return 16;
    alias.reset();if(GetHandleInformation(event,&info))return 18;
    std::puts("A17_FDTRIGGER_REAL_SOCKET_CANCEL_DATA_EOF_AND_UNREGISTER_ERROR_RETENTION_PASS");return 0;
}
int main(){
    std::mutex mutex;std::condition_variable cv;bool done=false;
    std::thread deadline([&]{std::unique_lock<std::mutex> l(mutex);if(!cv.wait_for(l,std::chrono::seconds(10),[&]{return done;}))std::_Exit(90);});
    int r=test();{std::lock_guard<std::mutex> l(mutex);done=true;}cv.notify_one();deadline.join();return r;
}
