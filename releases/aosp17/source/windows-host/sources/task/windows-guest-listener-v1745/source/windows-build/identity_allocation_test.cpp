#include "WindowsCallerIdentity.h"
#include <atomic>
#include <cstdlib>
#include <new>
#include <cstdio>
static std::atomic<bool> failAllocation{false};
void* operator new(std::size_t n){if(failAllocation.exchange(false))throw std::bad_alloc();if(void* p=std::malloc(n))return p;throw std::bad_alloc();}
void* operator new[](std::size_t n){return ::operator new(n);}
void operator delete(void* p) noexcept{std::free(p);}
void operator delete[](void* p) noexcept{std::free(p);}
void operator delete(void* p,std::size_t) noexcept{std::free(p);}
void operator delete[](void* p,std::size_t) noexcept{std::free(p);}
static HANDLE observed=nullptr;static unsigned closes=0;
static BOOL (WINAPI *realClose)(HANDLE)=::CloseHandle;
static BOOL countedClose(HANDLE value){if(value==observed)++closes;return realClose(value);}
#define CloseHandle countedClose
#include "WindowsCallerIdentity.cpp"
#undef CloseHandle
int wmain(){
 observed=CreateEventW(nullptr,TRUE,FALSE,nullptr);if(!observed)return 1;
 bool caught=false;
 {
 bscp::windows::Handle owner;owner.value=observed;
 failAllocation=true;
 try {auto shared=bscp::windows::shareOwnedHandle(owner);(void)shared;}
 catch(const std::bad_alloc&){caught=true;}
 failAllocation=false;
 if(!caught||owner.value||closes!=1)return 2;
 }
 if(closes!=1)return 3;
 if(WaitForSingleObject(observed,0)!=WAIT_FAILED||GetLastError()!=ERROR_INVALID_HANDLE)return 4;
 std::puts("actual shared_ptr allocation failure: one close, original owner empty, event released PASS");return 0;
}
