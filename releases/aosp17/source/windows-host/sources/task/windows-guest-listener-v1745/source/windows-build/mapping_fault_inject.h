#pragma once
#include <windows.h>
#include <atomic>
inline std::atomic<bool> mappingFailure{false};
inline LPVOID injectedMapViewOfFile(HANDLE h,DWORD access,DWORD high,DWORD low,SIZE_T size){
    if(mappingFailure.load()){SetLastError(ERROR_NOT_ENOUGH_MEMORY);return nullptr;}
    return MapViewOfFile(h,access,high,low,size);
}
#define MapViewOfFile injectedMapViewOfFile
