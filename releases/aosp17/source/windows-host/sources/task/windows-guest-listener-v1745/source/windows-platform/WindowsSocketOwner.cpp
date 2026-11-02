#include "WindowsSocketOwner.h"
namespace bscp::windows {
static std::mutex socketMutex;
static std::unordered_map<int,SocketLease> sockets;
// Takes ownership of a newly created socket on all paths. Caller must not use it after adoption.
int adoptSocket(SOCKET socket){
    if(socket==INVALID_SOCKET){errno=EBADF;return -1;}
    if(!SetHandleInformation(reinterpret_cast<HANDLE>(socket),HANDLE_FLAG_INHERIT,0)){
        closesocket(socket);errno=EACCES;return -1;
    }
    int slot=_open("NUL",_O_RDWR|_O_BINARY|_O_NOINHERIT);
    if(slot<0){closesocket(socket);return -1;}
    SocketLease owner;
    try{owner=std::make_shared<SocketOwner>(socket,slot);}catch(...){_close(slot);closesocket(socket);errno=ENOMEM;return -1;}
    try{
        std::lock_guard<std::mutex> lock(socketMutex);
        if(!sockets.emplace(slot,owner).second){errno=EIO;return -1;}
    }catch(...){errno=ENOMEM;return -1;}
    return slot;
}
SocketLease acquireSocket(int fd){
    std::lock_guard<std::mutex> lock(socketMutex);
    auto i=sockets.find(fd);return i==sockets.end()?nullptr:i->second;
}
bool closeSocket(int fd){
    SocketLease released;
    {std::lock_guard<std::mutex> lock(socketMutex);auto i=sockets.find(fd);
     if(i==sockets.end())return false;released=std::move(i->second);sockets.erase(i);}
    return true;
}
}
