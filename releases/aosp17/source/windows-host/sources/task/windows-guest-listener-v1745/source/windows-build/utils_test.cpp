#include <utils/String8.h>
#include <utils/String16.h>
#include "SharedBuffer.h"
#include <cstdio>
#include <cstring>
int main() {
 android::String8 text("A17 Windows");
 android::String16 wide(text);
 android::String8 round(wide);
 if (std::strcmp(round.c_str(),"A17 Windows")!=0) return 1;
 auto* b=android::SharedBuffer::alloc(64);
 if (!b || b->size()!=64) return 2;
 std::memset(b->data(),0x5a,64);
 b->acquire(); b->release();
 if (static_cast<unsigned char*>(b->data())[63]!=0x5a) return 3;
 b->release();
 char out[64]; std::snprintf(out,sizeof(out),"%zu",static_cast<size_t>(4294967297ULL));
 if (std::strcmp(out,"4294967297")!=0) return 4;
 std::puts("A17_STRING_BUFFER_PRINTF_PASS");return 0;
}
