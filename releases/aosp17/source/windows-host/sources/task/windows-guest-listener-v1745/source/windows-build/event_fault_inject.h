#pragma once
#include <winsock2.h>
#include <atomic>
inline std::atomic<bool> failEventClear{false};
inline int injectedEventSelect(SOCKET s,WSAEVENT event,long flags){
    if(failEventClear.load() && !event && flags==0){WSASetLastError(WSAEINVAL);return SOCKET_ERROR;}
    return WSAEventSelect(s,event,flags);
}
#define WSAEventSelect injectedEventSelect
