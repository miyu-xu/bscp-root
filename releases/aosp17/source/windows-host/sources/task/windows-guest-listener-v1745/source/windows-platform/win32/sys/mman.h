#pragma once
#include <windows.h>
#include <io.h>
#include <sys/types.h>
#include <cerrno>
#include <cstdint>
#include <limits>
#include <mutex>
#include <unordered_map>
#ifdef NO_ERROR
#undef NO_ERROR
#endif
#define PROT_READ 1
#define PROT_WRITE 2
#define PROT_EXEC 4
#define PROT_NONE 0
#define MAP_SHARED 1
#define MAP_PRIVATE 2
#define MAP_FIXED 16
#define MAP_ANONYMOUS 32
#define MAP_FAILED ((void*)-1)
namespace bscp::windows::mapping {
struct Entry { void* base; size_t length; };
inline std::mutex mutex;
inline std::unordered_map<void*,Entry> entries;
}
// This RPC recording mapping contract is read-only, regular-file and whole-view unmap.
// Unsupported protections/anonymous/fixed mappings fail explicitly, never masquerade as supported.
inline void* mmap(void* address,size_t length,int protection,int flags,int fd,off_t offset) {
    if(address || !length || protection!=PROT_READ || flags!=MAP_SHARED || fd<0 || offset<0) {
        errno=EINVAL;return MAP_FAILED;
    }
    SYSTEM_INFO info{};GetSystemInfo(&info);
    if(!info.dwAllocationGranularity || !info.dwPageSize || uint64_t(offset)%info.dwPageSize) {
        errno=EINVAL;return MAP_FAILED;
    }
    const auto native=_get_osfhandle(fd);
    if(native==-1){errno=EBADF;return MAP_FAILED;}
    HANDLE file=reinterpret_cast<HANDLE>(native);
    LARGE_INTEGER size{};
    if(GetFileType(file)!=FILE_TYPE_DISK || !GetFileSizeEx(file,&size)){errno=EIO;return MAP_FAILED;}
    if(uint64_t(offset)>uint64_t(size.QuadPart) || length>uint64_t(size.QuadPart)-uint64_t(offset)) {
        errno=EINVAL;return MAP_FAILED;
    }
    uint64_t baseOffset=uint64_t(offset)/info.dwAllocationGranularity*info.dwAllocationGranularity;
    size_t delta=static_cast<size_t>(uint64_t(offset)-baseOffset);
    if(length>std::numeric_limits<size_t>::max()-delta){errno=EOVERFLOW;return MAP_FAILED;}
    HANDLE map=CreateFileMappingW(file,nullptr,PAGE_READONLY,0,0,nullptr);
    if(!map){errno=EIO;return MAP_FAILED;}
    void* base=MapViewOfFile(map,FILE_MAP_READ,DWORD(baseOffset>>32),DWORD(baseOffset),length+delta);
    CloseHandle(map);
    if(!base){errno=EIO;return MAP_FAILED;}
    void* result=static_cast<unsigned char*>(base)+delta;
    try {
        std::lock_guard<std::mutex> lock(bscp::windows::mapping::mutex);
        if(!bscp::windows::mapping::entries.emplace(result,bscp::windows::mapping::Entry{base,length}).second){
            UnmapViewOfFile(base);errno=EIO;return MAP_FAILED;
        }
    } catch(...) {UnmapViewOfFile(base);errno=ENOMEM;return MAP_FAILED;}
    return result;
}
inline int munmap(void* address,size_t length) {
    std::lock_guard<std::mutex> lock(bscp::windows::mapping::mutex);
    auto i=bscp::windows::mapping::entries.find(address);
    if(i==bscp::windows::mapping::entries.end() || i->second.length!=length){errno=EINVAL;return -1;}
    if(!UnmapViewOfFile(i->second.base)){errno=EIO;return -1;}
    bscp::windows::mapping::entries.erase(i);return 0;
}
