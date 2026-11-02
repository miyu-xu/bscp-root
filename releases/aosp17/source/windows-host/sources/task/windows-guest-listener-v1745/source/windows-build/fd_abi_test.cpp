#include "WindowsFdAbi.h"
#include "WindowsSocketApi.h"
#include <cstdio>
#include <cerrno>
int main(){
 HANDLE rd=nullptr,wr=nullptr;if(!CreatePipe(&rd,&wr,nullptr,0))return 1;
 int fd=-1;if(bscp_fd_from_handle(rd,&fd)!=0)return 2;CloseHandle(rd);
 HANDLE out=nullptr;if(bscp_fd_to_handle(fd,&out)!=0)return 3;
 DWORD flags=0;if(!GetHandleInformation(out,&flags)||(flags&HANDLE_FLAG_INHERIT))return 4;
 int fd2=-1;if(bscp_fd_dup(fd,&fd2)!=0 || bscp_fd_close(fd)!=0)return 5;
 char c='x',v=0;DWORD n=0;if(!WriteFile(wr,&c,1,&n,nullptr)||n!=1||!ReadFile(out,&v,1,&n,nullptr)||n!=1||v!='x')return 6;
 CloseHandle(out);CloseHandle(wr);if(bscp_fd_close(fd2)!=0)return 7;
 int socket=bscp::windows::socket(AF_INET,SOCK_STREAM,0);if(socket<0)return 8;
 if(bscp_fd_to_handle(socket,&out)!=-EOPNOTSUPP || out!=nullptr)return 9;
 auto lease=bscp::windows::acquireSocket(socket);int bad=-1;
 if(bscp_fd_from_handle(reinterpret_cast<HANDLE>(lease->socket),&bad)!=-EOPNOTSUPP || bad!=-1)return 10;
 if(bscp_fd_close(socket)!=0)return 11;lease.reset();
 if(bscp_fd_from_handle(INVALID_HANDLE_VALUE,&bad)!=-EBADF)return 12;
 std::puts("A17_WINDOWS_NATIVE_FD_ABI_PIPE_DUP_NO_INHERIT_SOCKET_REJECT_PASS");return 0;
}
