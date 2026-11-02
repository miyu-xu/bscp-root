#pragma once
#include "WindowsSocketOwner.h"
#include <cstddef>
namespace bscp::windows {
int initializeSockets();
int socket(int domain,int type,int protocol);
int getsockname(int fd,sockaddr* addr,int* length);
int getpeername(int fd,sockaddr* addr,int* length);
int bind(int fd,const sockaddr* addr,size_t length);
int connect(int fd,const sockaddr* addr,size_t length);
int listen(int fd,int backlog);
int accept4(int fd,sockaddr* addr,int* length,int flags);
int getsockopt(int fd,int level,int option,void* value,int* length);
int setsockopt(int fd,int level,int option,const void* value,int length);
int recv(int fd,void* data,size_t length,int flags);
int send(int fd,const void* data,size_t length,int flags);
int socketpair(int domain,int type,int protocol,int out[2]);
int duplicateSocket(int fd);
}
