#include <windows.h>
#include <io.h>
#include <fcntl.h>
#include <cstdio>
#include "OS.h"
int main() {
    int copy = -1;
    if (android::binder::os::dupFileDescriptor(-1, &copy) == android::OK || copy != -1) return 1;
    HANDLE original = CreateFileW(L"fd-test-owned.tmp", GENERIC_READ | GENERIC_WRITE, 0, nullptr,
        CREATE_NEW, FILE_ATTRIBUTE_TEMPORARY | FILE_FLAG_DELETE_ON_CLOSE, nullptr);
    if (original == INVALID_HANDLE_VALUE) return 2;
    int fd = _open_osfhandle(reinterpret_cast<intptr_t>(original), _O_BINARY | _O_NOINHERIT);
    if (fd < 0) { CloseHandle(original); return 3; }
    if (android::binder::os::dupFileDescriptor(fd, &copy) != android::OK) { _close(fd); return 4; }
    DWORD flags=0;
    bool noninherit=GetHandleInformation(reinterpret_cast<HANDLE>(_get_osfhandle(copy)), &flags) && !(flags & HANDLE_FLAG_INHERIT);
    _close(fd);
    const bool usable = _write(copy,"A17",3)==3 && _lseek(copy,0,SEEK_SET)==0;
    char data[3]={}; const bool readback=usable && _read(copy,data,3)==3 && data[0]=='A' && data[2]=='7';
    _close(copy);
    if (!noninherit || !readback) return 5;
    std::puts("A17_WINDOWS_DUPLICATE_NONINHERITED_OWNERSHIP_PASS"); return 0;
}
