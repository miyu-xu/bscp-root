#pragma once
#include <windows.h>
inline int getpagesize() { SYSTEM_INFO info{}; GetSystemInfo(&info); return static_cast<int>(info.dwPageSize); }
