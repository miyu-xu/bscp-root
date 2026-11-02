#include <binder/Common.h>
#include <cstdarg>
#include <cstdio>
extern "C" LIBBINDER_EXPORTED int __android_log_print(int, const char*, const char* fmt, ...) { va_list args; va_start(args,fmt); vfprintf(stderr,fmt,args); va_end(args); return 1; }
