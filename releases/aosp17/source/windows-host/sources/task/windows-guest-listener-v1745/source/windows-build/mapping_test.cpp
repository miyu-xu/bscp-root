#include <sys/mman.h>
#include <fcntl.h>
#include <vector>
#include <cstdio>
int main(){
    HANDLE h=CreateFileW(L"mapping-test-owned.tmp",GENERIC_READ|GENERIC_WRITE,0,nullptr,CREATE_NEW,
                         FILE_ATTRIBUTE_TEMPORARY|FILE_FLAG_DELETE_ON_CLOSE,nullptr);
    if(h==INVALID_HANDLE_VALUE)return 1;
    int fd=_open_osfhandle(reinterpret_cast<intptr_t>(h),_O_BINARY|_O_NOINHERIT);
    if(fd<0){CloseHandle(h);return 2;}
    SYSTEM_INFO info{};GetSystemInfo(&info);
    std::vector<unsigned char> data(info.dwAllocationGranularity+info.dwPageSize,0x5a);
    if(_write(fd,data.data(),unsigned(data.size()))!=int(data.size())){_close(fd);return 3;}
    void* p=mmap(nullptr,info.dwPageSize,PROT_READ,MAP_SHARED,fd,info.dwPageSize);
    if(p==MAP_FAILED){_close(fd);return 4;}
    bool ok=static_cast<unsigned char*>(p)[0]==0x5a && static_cast<unsigned char*>(p)[info.dwPageSize-1]==0x5a;
    ok=ok && munmap(p,info.dwPageSize-1)==-1 && munmap(p,info.dwPageSize)==0 && munmap(p,info.dwPageSize)==-1;
    ok=ok && mmap(nullptr,1,PROT_WRITE,MAP_SHARED,fd,0)==MAP_FAILED;
    ok=ok && mmap(nullptr,1,PROT_READ,MAP_SHARED,fd,1)==MAP_FAILED;
    ok=ok && mmap(nullptr,data.size()+1,PROT_READ,MAP_SHARED,fd,0)==MAP_FAILED;
    _close(fd);if(!ok)return 5;
    std::puts("WINDOWS_MAPPING_OFFSET_OWNERSHIP_REJECTION_PASS");return 0;
}
