#include "WindowsSocketOwner.h"
#include "WindowsCancellation.h"
#include <cstdio>
int main(){
    WSADATA data{};if(WSAStartup(MAKEWORD(2,2),&data))return 1;
    SOCKET socket=WSASocketW(AF_INET,SOCK_STREAM,IPPROTO_TCP,nullptr,0,WSA_FLAG_OVERLAPPED|WSA_FLAG_NO_HANDLE_INHERIT);
    if(socket==INVALID_SOCKET)return 2;
    int fd=bscp::windows::adoptSocket(socket);if(fd<0)return 3;
    auto lease=bscp::windows::acquireSocket(fd);if(!lease || lease->socket!=socket)return 4;
    DWORD flags=0;if(!GetHandleInformation(reinterpret_cast<HANDLE>(lease->socket),&flags) || (flags&HANDLE_FLAG_INHERIT))return 5;
    int file=_open("NUL",_O_RDONLY|_O_NOINHERIT);if(file<0)return 6;
    if(bscp::windows::acquireSocket(file) || bscp::windows::closeSocket(file))return 7;
    _close(file);
    if(!bscp::windows::closeSocket(fd) || bscp::windows::acquireSocket(fd))return 8;
    // Original stable native object remains alive for the already acquired operation lease.
    int type=0,len=sizeof(type);
    if(getsockopt(lease->socket,SOL_SOCKET,SO_TYPE,reinterpret_cast<char*>(&type),&len) || type!=SOCK_STREAM)return 9;
    int reserved=_open("NUL",_O_RDONLY|_O_NOINHERIT);if(reserved==fd)return 10;_close(reserved);
    lease.reset();
    if(bscp::windows::closeSocket(fd))return 11;
    WSACleanup();std::puts("WINDOWS_SOCKET_CRT_KIND_STABLE_LEASE_PASS");return 0;
}
