#include <windows.h>
#include <io.h>
#include <fcntl.h>
#include <cerrno>
#include "OS.h"
#include "WindowsSocketApi.h"
#include "WindowsOverlappedPipeFd.h"
namespace android::binder::os {
int closeFileDescriptor(int fd) {
    int pipeStatus=0;if(bscp::windows::closeOverlappedPipe(fd,&pipeStatus)){if(pipeStatus){errno=-pipeStatus;return -1;}return 0;}
    if (bscp::windows::closeSocket(fd)) return 0;
    return _close(fd);
}
status_t dupFileDescriptor(int oldFd, int* newFd) {
    if (!newFd || oldFd < 0) return BAD_VALUE;
    if(auto pipe=bscp::windows::acquireOverlappedPipe(oldFd))return bscp::windows::duplicateOverlappedPipe(pipe,newFd);
    if(bscp::windows::isOverlappedPipeSlot(oldFd))return -EBADF;
    // Socket duplication needs WSADuplicateSocket and is not a CRT HANDLE duplication.
    if (bscp::windows::acquireSocket(oldFd)) {
        int copy=bscp::windows::duplicateSocket(oldFd);
        if(copy<0)return -errno;
        *newFd=copy;return OK;
    }
    const intptr_t original = _get_osfhandle(oldFd);
    if (original == -1) return -EBADF;
    HANDLE copy = INVALID_HANDLE_VALUE;
    if (!DuplicateHandle(GetCurrentProcess(), reinterpret_cast<HANDLE>(original),
                         GetCurrentProcess(), &copy, 0, FALSE, DUPLICATE_SAME_ACCESS)) {
        const DWORD error = GetLastError();
        return error == ERROR_ACCESS_DENIED ? -EACCES : -EIO;
    }
    // Ownership transfers only on successful CRT registration; never narrow HANDLE to int.
    const int result = _open_osfhandle(reinterpret_cast<intptr_t>(copy), _O_BINARY | _O_NOINHERIT);
    if (result < 0) {
        const int error = errno;
        CloseHandle(copy);
        return -error;
    }
    *newFd = result;
    return OK;
}
}
