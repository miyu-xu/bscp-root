#include "WindowsSocketApi.h"
#include <ws2tcpip.h>
#include <limits>
namespace bscp::windows {
namespace {
constexpr int closeExec=0x80000,nonBlock=0x800,dontWait=0x40;
int error(){
    switch(WSAGetLastError()){
    case WSAEWOULDBLOCK:errno=EAGAIN;break;
    case WSAEINPROGRESS:case WSAEALREADY:errno=EINPROGRESS;break;
    case WSAEACCES:errno=EACCES;break;
    case WSAEINVAL:errno=EINVAL;break;
    case WSAEINTR:errno=EINTR;break;
    case WSAECONNRESET:errno=ECONNRESET;break;
    case WSAECONNREFUSED:errno=ECONNREFUSED;break;
    case WSAENOTSOCK:errno=EBADF;break;
    case WSAENOBUFS:errno=ENOMEM;break;
    case WSAEAFNOSUPPORT:errno=EAFNOSUPPORT;break;
    case WSAETIMEDOUT:errno=ETIMEDOUT;break;
    default:errno=EIO;break;
    }return -1;
}
struct Runtime {int status;Runtime(){WSADATA data{};status=WSAStartup(MAKEWORD(2,2),&data);}~Runtime(){if(!status)WSACleanup();}};
Runtime& runtime(){static Runtime value;return value;}
bool lengthOk(size_t size){if(size>size_t(std::numeric_limits<int>::max())){errno=EINVAL;return false;}return true;}
SocketLease lease(int fd){auto value=acquireSocket(fd);if(!value)errno=EBADF;return value;}
}
int initializeSockets(){int status=runtime().status;if(status){WSASetLastError(status);errno=EIO;}return status;}
int socket(int domain,int type,int protocol){
    if(runtime().status){errno=EIO;return -1;}
    if((type&~(closeExec|nonBlock))!=SOCK_STREAM){errno=EPROTONOSUPPORT;return -1;}
    SOCKET value=WSASocketW(domain,SOCK_STREAM,protocol,nullptr,0,WSA_FLAG_OVERLAPPED|WSA_FLAG_NO_HANDLE_INHERIT);
    if(value==INVALID_SOCKET)return error();
    // RPC descriptors are always nonblocking and noninheritable.
    u_long mode=1;if(ioctlsocket(value,FIONBIO,&mode)){int saved=WSAGetLastError();closesocket(value);WSASetLastError(saved);return error();}
    return adoptSocket(value);
}
int getsockname(int fd,sockaddr* addr,int* length){auto s=lease(fd);if(!s)return -1;return ::getsockname(s->socket,addr,length)==SOCKET_ERROR?error():0;}
int getpeername(int fd,sockaddr* addr,int* length){auto s=lease(fd);if(!s)return -1;return ::getpeername(s->socket,addr,length)==SOCKET_ERROR?error():0;}
int bind(int fd,const sockaddr* addr,size_t length){auto s=lease(fd);if(!s||!lengthOk(length))return -1;return ::bind(s->socket,addr,int(length))==SOCKET_ERROR?error():0;}
int connect(int fd,const sockaddr* addr,size_t length){auto s=lease(fd);if(!s||!lengthOk(length))return -1;int r=::connect(s->socket,addr,int(length));if(r==SOCKET_ERROR){int e=WSAGetLastError();if(e==WSAEWOULDBLOCK){errno=EINPROGRESS;return -1;}return error();}return 0;}
int listen(int fd,int backlog){auto s=lease(fd);if(!s)return -1;return ::listen(s->socket,backlog)==SOCKET_ERROR?error():0;}
int accept4(int fd,sockaddr* addr,int* length,int flags){
    if(flags&~(closeExec|nonBlock)){errno=EINVAL;return -1;}
    auto s=lease(fd);if(!s)return -1;SOCKET value=::accept(s->socket,addr,length);if(value==INVALID_SOCKET)return error();
    DWORD inherited=0;
    if(!GetHandleInformation(reinterpret_cast<HANDLE>(value),&inherited) || (inherited&HANDLE_FLAG_INHERIT)){
        closesocket(value);errno=EACCES;return -1;
    }
    // Remove inherited listener event association before creating a new connection owner.
    if(WSAEventSelect(value,nullptr,0)){int e=WSAGetLastError();closesocket(value);WSASetLastError(e);return error();}
    u_long mode=1;if(ioctlsocket(value,FIONBIO,&mode)){int e=WSAGetLastError();closesocket(value);WSASetLastError(e);return error();}
    return adoptSocket(value);
}
int getsockopt(int fd,int level,int option,void* value,int* length){auto s=lease(fd);if(!s)return -1;return ::getsockopt(s->socket,level,option,static_cast<char*>(value),length)==SOCKET_ERROR?error():0;}
int setsockopt(int fd,int level,int option,const void* value,int length){auto s=lease(fd);if(!s)return -1;return ::setsockopt(s->socket,level,option,static_cast<const char*>(value),length)==SOCKET_ERROR?error():0;}
int recv(int fd,void* data,size_t length,int flags){auto s=lease(fd);if(!s||!lengthOk(length))return -1;int r=::recv(s->socket,static_cast<char*>(data),int(length),flags&~dontWait);return r==SOCKET_ERROR?error():r;}
int send(int fd,const void* data,size_t length,int flags){auto s=lease(fd);if(!s||!lengthOk(length))return -1;int r=::send(s->socket,static_cast<const char*>(data),int(length),flags&~dontWait);return r==SOCKET_ERROR?error():r;}
int duplicateSocket(int fd){
    auto s=lease(fd);if(!s)return -1;WSAPROTOCOL_INFOW info{};
    if(WSADuplicateSocketW(s->socket,GetCurrentProcessId(),&info))return error();
    SOCKET value=WSASocketW(FROM_PROTOCOL_INFO,FROM_PROTOCOL_INFO,FROM_PROTOCOL_INFO,&info,0,WSA_FLAG_OVERLAPPED|WSA_FLAG_NO_HANDLE_INHERIT);
    if(value==INVALID_SOCKET)return error();
    int copy=adoptSocket(value);if(copy<0)return -1;
    acquireSocket(copy)->shared=s->shared;
    return copy;
}
int socketpair(int domain,int type,int protocol,int out[2]){
    // Windows has no native AF_UNIX socketpair. Use a private loopback pair, verifying its
    // exact connecting endpoint. This supplies a byte stream, not caller authentication.
    if(!out||domain!=AF_UNIX||(type&~(closeExec|nonBlock))!=SOCK_STREAM||protocol){errno=EINVAL;return -1;}
    int listener=socket(AF_INET,SOCK_STREAM,IPPROTO_TCP);if(listener<0)return -1;
    int client=-1,server=-1;int result=-1;
    auto cleanup=[&]{closeSocket(listener);if(result<0){if(client>=0)closeSocket(client);if(server>=0)closeSocket(server);}};
    sockaddr_in addr{};addr.sin_family=AF_INET;addr.sin_addr.s_addr=htonl(INADDR_LOOPBACK);
    if(bind(listener,reinterpret_cast<sockaddr*>(&addr),sizeof(addr))||listen(listener,1)){cleanup();return -1;}
    int size=sizeof(addr);auto l=lease(listener);
    if(::getsockname(l->socket,reinterpret_cast<sockaddr*>(&addr),&size)){error();cleanup();return -1;}
    client=socket(AF_INET,SOCK_STREAM,IPPROTO_TCP);if(client<0){cleanup();return -1;}
    if(connect(client,reinterpret_cast<sockaddr*>(&addr),sizeof(addr)) && errno!=EINPROGRESS){cleanup();return -1;}
    WSAPOLLFD ready{l->socket,POLLIN,0};if(WSAPoll(&ready,1,5000)!=1){errno=ETIMEDOUT;cleanup();return -1;}
    sockaddr_in peer{};size=sizeof(peer);server=accept4(listener,reinterpret_cast<sockaddr*>(&peer),&size,closeExec|nonBlock);
    if(server<0){cleanup();return -1;}
    sockaddr_in local{};size=sizeof(local);auto c=lease(client);
    if(::getsockname(c->socket,reinterpret_cast<sockaddr*>(&local),&size)||peer.sin_addr.s_addr!=local.sin_addr.s_addr||peer.sin_port!=local.sin_port){errno=EACCES;cleanup();return -1;}
    int connectError=0;size=sizeof(connectError);
    if(::getsockopt(c->socket,SOL_SOCKET,SO_ERROR,reinterpret_cast<char*>(&connectError),&size)||connectError){errno=EIO;cleanup();return -1;}
    out[0]=client;out[1]=server;result=0;cleanup();return 0;
}
}
