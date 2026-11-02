#pragma once
#include <winsock2.h>
#include <io.h>
#include <fcntl.h>
#include <memory>
#include <mutex>
#include <unordered_map>
#include <cerrno>
#include <vector>
#include <cstdio>
#include <atomic>
namespace bscp::windows {
// Opaque CRT-reserved descriptor numbers are never cast to SOCKET or HANDLE.
// A lease retains both the numeric slot and native socket until an in-flight operation ends.
struct SocketSharedState final {
    std::mutex pollingMutex;
    std::vector<WSAEVENT> retainedEvents;
    std::atomic<bool> closeUncertain{false};
    ~SocketSharedState(){
        if(!closeUncertain.load()){for(auto event:retainedEvents)WSACloseEvent(event);}
    }
};
struct SocketOwner final {
    SOCKET socket=INVALID_SOCKET;
    int slot=-1;
    std::shared_ptr<SocketSharedState> shared=std::make_shared<SocketSharedState>();

    SocketOwner(SOCKET value,int descriptor):socket(value),slot(descriptor){}
    SocketOwner(const SocketOwner&)=delete;
    ~SocketOwner(){
        const bool closed=socket==INVALID_SOCKET || closesocket(socket)==0;
        if(!closed){shared->closeUncertain=true;std::fputs("Windows socket close failed; associated events retained, resource state unknown\n",stderr);}
        if(slot>=0)_close(slot);
    }
};
using SocketLease=std::shared_ptr<SocketOwner>;
int adoptSocket(SOCKET socket);
SocketLease acquireSocket(int fd);
bool closeSocket(int fd);
}
