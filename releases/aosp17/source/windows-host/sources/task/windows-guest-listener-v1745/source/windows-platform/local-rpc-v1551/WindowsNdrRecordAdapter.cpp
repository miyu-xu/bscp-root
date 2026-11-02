#include "WindowsNdrRecordAdapter.h"
#include <atomic>
#include <cstring>
#include <new>
namespace bscp::windows {
namespace {
std::atomic<unsigned> failStage{0};
template<class T> struct ControlledAllocator {
    using value_type=T;unsigned stage;
    explicit ControlledAllocator(unsigned s):stage(s){}
    template<class U> ControlledAllocator(const ControlledAllocator<U>& a):stage(a.stage){}
    T* allocate(size_t n){unsigned expected=stage;if(failStage.compare_exchange_strong(expected,0))throw std::bad_alloc();return std::allocator<T>{}.allocate(n);}
    void deallocate(T* p,size_t n){std::allocator<T>{}.deallocate(p,n);}
    template<class U> bool operator==(const ControlledAllocator<U>& b)const{return stage==b.stage;}
};
template<class T> std::shared_ptr<T> shareOwned(std::unique_ptr<T>& owned,unsigned stage){
    // shared_ptr invokes the deleter if its control-block allocation throws.
    // Relinquish the previous owner BEFORE construction to avoid double close.
    T* p=owned.release();return std::shared_ptr<T>(p,std::default_delete<T>{},ControlledAllocator<T>{stage});
}
DWORD alive(HANDLE p){DWORD w=WaitForSingleObject(p,0);return w==WAIT_TIMEOUT?ERROR_SUCCESS:w==WAIT_OBJECT_0?ERROR_PROCESS_ABORTED:w==WAIT_FAILED?GetLastError():ERROR_INVALID_DATA;}
}
#ifdef BSCP_WLR_ISOLATED_COMPONENT
void windowsNdrTestFailControlBlock(unsigned stage){failStage.store(stage);}
#endif
WindowsNdrRequestSnapshot::~WindowsNdrRequestSnapshot(){if(token_)CloseHandle(token_);if(process_)CloseHandle(process_);}
DWORD WindowsNdrRequestSnapshot::checkedCallingPid(DWORD* out)const{if(!out)return ERROR_INVALID_PARAMETER;*out=0;DWORD e=alive(process_);if(e)return e;if(!pid_||GetProcessId(process_)!=pid_)return ERROR_INVALID_DATA;*out=pid_;return 0;}
DWORD WindowsNdrRequestSnapshot::matchesOriginalProcess(HANDLE expected,bool* same)const{
 if(!same)return ERROR_INVALID_PARAMETER;*same=false;if(!expected||expected==INVALID_HANDLE_VALUE)return ERROR_INVALID_HANDLE;
 DWORD pid=0,e=checkedCallingPid(&pid);if(e)return e;if(!GetProcessId(expected))return GetLastError();e=alive(expected);if(e)return e;
 using Compare=BOOL(WINAPI*)(HANDLE,HANDLE);HMODULE kernel=GetModuleHandleW(L"kernelbase.dll");if(!kernel)return ERROR_MOD_NOT_FOUND;FARPROC address=GetProcAddress(kernel,"CompareObjectHandles");if(!address)return ERROR_PROC_NOT_FOUND;Compare compare=nullptr;static_assert(sizeof(compare)==sizeof(address));std::memcpy(&compare,&address,sizeof(compare));
 SetLastError(ERROR_SUCCESS);BOOL equal=compare(process_,expected);DWORD comparisonError=GetLastError();if(!equal&&comparisonError!=ERROR_SUCCESS&&comparisonError!=ERROR_NOT_SAME_OBJECT)return comparisonError;
 e=checkedCallingPid(&pid);if(e)return e;e=alive(expected);if(e)return e;*same=equal!=FALSE;return ERROR_SUCCESS;
}
DWORD WindowsNdrRequestSnapshot::matchesOriginalSnapshot(const WindowsNdrRequestSnapshot& other,bool* same) const {
 if (!same) return ERROR_INVALID_PARAMETER; *same=false; DWORD pid=0;
 DWORD e=other.checkedCallingPid(&pid); if(e)return e;
 return matchesOriginalProcess(other.process_,same);
}
WindowsNdrReceivedRecord::WindowsNdrReceivedRecord(std::unique_ptr<WlrRecord,Release> r,const WlrView& v):record_(std::move(r)),view_(v){}
DWORD WindowsNdrReceivedRecord::tryPop(uint64_t g,uint64_t id,std::shared_ptr<WindowsNdrReceivedRecord>* out){if(!out)return ERROR_INVALID_PARAMETER;out->reset();WlrRecord* raw=nullptr;DWORD e=WlrServerPop(g,id,&raw);if(e)return e;std::unique_ptr<WlrRecord,Release> record(raw);WlrView view{};
    if(WlrRecordView(raw,&view))return ERROR_INVALID_DATA;
    HANDLE token=nullptr,process=nullptr;DWORD pid=0;e=WlrRecordQueryRequestIdentity(raw,&token,&process,&pid);if(e)return e;
    if(!token||!process||!pid)return ERROR_ACCESS_DENIED;
    try {auto owned=std::unique_ptr<WindowsNdrReceivedRecord>(new WindowsNdrReceivedRecord(std::move(record),view));*out=shareOwned(owned,1);return ERROR_SUCCESS;}catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
}
DWORD WindowsNdrReceivedRecord::tryPopAccepted(WlrAcceptedChannel* channel,std::shared_ptr<WindowsNdrReceivedRecord>* out){if(!out)return ERROR_INVALID_PARAMETER;out->reset();WlrRecord* raw=nullptr;DWORD e=WlrAcceptedPop(channel,&raw);if(e)return e;std::unique_ptr<WlrRecord,Release> record(raw);WlrView view{};
    if(WlrRecordView(raw,&view))return ERROR_INVALID_DATA;
    HANDLE token=nullptr,process=nullptr;DWORD pid=0;e=WlrRecordQueryRequestIdentity(raw,&token,&process,&pid);if(e)return e;
    if(!token||!process||!pid)return ERROR_ACCESS_DENIED;
    try {auto owned=std::unique_ptr<WindowsNdrReceivedRecord>(new WindowsNdrReceivedRecord(std::move(record),view));*out=shareOwned(owned,1);return ERROR_SUCCESS;}catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
}
DWORD WindowsNdrReceivedRecord::readExact(std::span<unsigned char> out){if(sealed_||out.size()>view_.bytes-consumed_)return ERROR_INVALID_DATA;if(!out.empty())std::memcpy(out.data(),view_.data+consumed_,out.size());consumed_+=out.size();return 0;}
DWORD WindowsNdrReceivedRecord::sealConsumedRecord(){if(sealed_||consumed_!=view_.bytes)return ERROR_INVALID_DATA;sealed_=true;return 0;}
DWORD WindowsNdrReceivedRecord::makeSnapshot(std::shared_ptr<const WindowsNdrRequestSnapshot>* out)const{if(!out)return ERROR_INVALID_PARAMETER;out->reset();if(!sealed_)return ERROR_INVALID_DATA;HANDLE token=nullptr,process=nullptr;DWORD pid=0;DWORD e=WlrRecordQueryRequestIdentity(record_.get(),&token,&process,&pid);if(e)return e;
    try {std::unique_ptr<WindowsNdrRequestSnapshot> s(new WindowsNdrRequestSnapshot);
      if(!DuplicateHandle(GetCurrentProcess(),token,GetCurrentProcess(),&s->token_,TOKEN_QUERY,FALSE,0)||!DuplicateHandle(GetCurrentProcess(),process,GetCurrentProcess(),&s->process_,PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,FALSE,0))return GetLastError();
      DWORD flags=0,n=0;TOKEN_TYPE type;SECURITY_IMPERSONATION_LEVEL level;TOKEN_STATISTICS stats{};
      if(!GetHandleInformation(s->token_,&flags)||(flags&HANDLE_FLAG_INHERIT)||!GetTokenInformation(s->token_,TokenType,&type,sizeof(type),&n)||type!=TokenImpersonation||!GetTokenInformation(s->token_,TokenImpersonationLevel,&level,sizeof(level),&n)||level<SecurityIdentification||!GetTokenInformation(s->token_,TokenStatistics,&stats,sizeof(stats),&n))return ERROR_ACCESS_DENIED;
      alignas(TOKEN_USER) unsigned char bytes[sizeof(TOKEN_USER)+SECURITY_MAX_SID_SIZE];if(!GetTokenInformation(s->token_,TokenUser,bytes,sizeof(bytes),&n))return GetLastError();PSID sid=reinterpret_cast<TOKEN_USER*>(bytes)->User.Sid;if(!IsValidSid(sid))return ERROR_INVALID_SID;
      DWORD length=GetLengthSid(sid);if(length>SECURITY_MAX_SID_SIZE)return ERROR_INVALID_SID;s->sid_.resize(length);if(!CopySid(length,s->sid_.data(),sid))return GetLastError();s->logon_=stats.AuthenticationId;s->pid_=pid;
      DWORD checked=0;e=s->checkedCallingPid(&checked);if(e)return e;*out=shareOwned(s,2);return ERROR_SUCCESS;
    }catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
}
}

