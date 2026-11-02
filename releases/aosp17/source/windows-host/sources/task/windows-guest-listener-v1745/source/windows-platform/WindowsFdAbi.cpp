#include "WindowsFdAbi.h"
#include "WindowsSocketOwner.h"
#include "WindowsOverlappedPipeFd.h"
#include "OS.h"
#include <io.h>
#include <fcntl.h>
#include <cerrno>
namespace {
int handleKind(HANDLE h){
 if(!h || h==INVALID_HANDLE_VALUE)return -EBADF;
 DWORD flags=0;if(!GetHandleInformation(h,&flags))return -EBADF;
 DWORD kind=GetFileType(h);
 if(kind==FILE_TYPE_DISK)return 0;
 // SOCKETs may appear as FILE_TYPE_PIPE, but are not named/anonymous pipe HANDLEs.
 if(kind==FILE_TYPE_PIPE && GetNamedPipeInfo(h,nullptr,nullptr,nullptr,nullptr))return 0;
 return -EOPNOTSUPP;
}
int copy(HANDLE h,HANDLE* out){
 int r=handleKind(h);if(r)return r;
 HANDLE value=nullptr;
 if(!DuplicateHandle(GetCurrentProcess(),h,GetCurrentProcess(),&value,0,FALSE,DUPLICATE_SAME_ACCESS))
  return GetLastError()==ERROR_ACCESS_DENIED?-EACCES:-EIO;
 *out=value;return 0;
}
}
extern "C" int bscp_fd_close(int fd){
 if(fd<0)return -EBADF;
 return android::binder::os::closeFileDescriptor(fd)==0?0:-errno;
}
extern "C" int bscp_fd_dup(int fd,int* out){
 if(!out)return -EINVAL;*out=-1;
 return android::binder::os::dupFileDescriptor(fd,out);
}
extern "C" int bscp_fd_from_handle(HANDLE input,int* out){
 if(!out)return -EINVAL;*out=-1;HANDLE value=nullptr;
 int r=handleKind(input);if(r)return r;
 if(GetFileType(input)==FILE_TYPE_PIPE){bscp::windows::PipeSnapshot snapshot;DWORD e=bscp::windows::inspectNativePipe(input,&snapshot);
  if(e)return -EIO; // unknown native mode must never fall through to CRT
  if(!(snapshot.mode&(0x10u|0x20u)))return bscp::windows::registerOverlappedPipe(input,out);
 }
 r=copy(input,&value);if(r)return r;
 int fd=_open_osfhandle(reinterpret_cast<intptr_t>(value),_O_BINARY|_O_NOINHERIT);
 if(fd<0){int error=errno;CloseHandle(value);return -error;}
 *out=fd;return 0;
}
extern "C" int bscp_fd_to_handle(int fd,HANDLE* out){
 if(!out)return -EINVAL;*out=nullptr;
 if(fd<0)return -EBADF;
 if(auto pipe=bscp::windows::acquireOverlappedPipe(fd))return bscp::windows::copyOverlappedPipe(pipe,out);
 if(bscp::windows::isOverlappedPipeSlot(fd))return -EBADF;
 if(bscp::windows::acquireSocket(fd))return -EOPNOTSUPP;
 intptr_t value=_get_osfhandle(fd);if(value==-1)return -EBADF;
 return copy(reinterpret_cast<HANDLE>(value),out);
}
