#include "WindowsCallerIdentity.h"
#include <cstdlib>
#include <cstring>
#include <limits>
namespace bscp::windows {
namespace {
thread_local const CallerIdentity* activeCaller=nullptr;
bool validSidBytes(const std::vector<unsigned char>& sid) {
    if(sid.size()<8 || sid[1]>SID_MAX_SUB_AUTHORITIES || sid.size()!=8+4*size_t(sid[1]))return false;
    return IsValidSid(const_cast<unsigned char*>(sid.data()))!=FALSE;
}
struct Handle {
    HANDLE value = nullptr;
    ~Handle() { if(value) CloseHandle(value); }
};
std::shared_ptr<void> shareOwnedHandle(Handle& owner) {
    // shared_ptr(raw, deleter) invokes deleter if control-block allocation fails.
    // Relinquish the original owner first, so failure has exactly one closer.
    HANDLE raw=owner.value;owner.value=nullptr;
    return std::shared_ptr<void>(raw,[](void* value){if(value)CloseHandle(value);});
}
DWORD readIdentity(HANDLE token, CallerIdentity* out) {
    if(!out) return ERROR_INVALID_PARAMETER;
    *out={};
    DWORD bytes=0;
    if(GetTokenInformation(token,TokenUser,nullptr,0,&bytes) || GetLastError()!=ERROR_INSUFFICIENT_BUFFER || !bytes)
        return ERROR_INVALID_DATA;
    std::vector<unsigned char> data(bytes);
    if(!GetTokenInformation(token,TokenUser,data.data(),bytes,&bytes))return GetLastError();
    auto user=reinterpret_cast<TOKEN_USER*>(data.data());
    if(!IsValidSid(user->User.Sid))return ERROR_INVALID_SID;
    DWORD length=GetLengthSid(user->User.Sid);
    CallerIdentity result;result.sid.resize(length);
    if(!CopySid(length,result.sid.data(),user->User.Sid))return GetLastError();
    TOKEN_STATISTICS statistics{};
    if(!GetTokenInformation(token,TokenStatistics,&statistics,sizeof(statistics),&bytes))return GetLastError();
    result.logon=statistics.AuthenticationId;*out=std::move(result);return ERROR_SUCCESS;
}
DWORD processIdentity(HANDLE process, CallerIdentity* out) {
    Handle token;
    if(!OpenProcessToken(process,TOKEN_QUERY,&token.value))return GetLastError();
    return readIdentity(token.value,out);
}
DWORD requireAlive(HANDLE process) {
    DWORD status=WaitForSingleObject(process,0);
    if(status==WAIT_TIMEOUT)return ERROR_SUCCESS;
    if(status==WAIT_OBJECT_0)return ERROR_PROCESS_ABORTED;
    return status==WAIT_FAILED?GetLastError():ERROR_INVALID_DATA;
}
struct Impersonation {
    bool active=false;
    ~Impersonation() { revert(); }
    void revert() {
        if(!active)return;
        // Microsoft requires stopping the process if restoration fails. Returning an
        // error would allow the worker to serve another request in client context.
        if(!RevertToSelf())std::abort();
        active=false;
        Handle token;
        if(OpenThreadToken(GetCurrentThread(),TOKEN_QUERY,TRUE,&token.value) || GetLastError()!=ERROR_NO_TOKEN)
            std::abort();
    }
};
}
bool CallerIdentity::sameAs(const CallerIdentity& other) const {
    return validSidBytes(sid) && validSidBytes(other.sid) &&
           EqualSid(const_cast<unsigned char*>(sid.data()),const_cast<unsigned char*>(other.sid.data())) &&
           logon.LowPart==other.logon.LowPart && logon.HighPart==other.logon.HighPart;
}
DWORD CallerIdentity::currentProcess(CallerIdentity* out) {return processIdentity(GetCurrentProcess(),out);}
CallerScope::CallerScope(const CallerIdentity* authenticated)
    : identity_(authenticated?std::optional<CallerIdentity>(*authenticated):std::nullopt),
      previous_(activeCaller) { activeCaller=identity_?&*identity_:nullptr; }
CallerScope::~CallerScope(){activeCaller=previous_;}
const CallerIdentity* CallerScope::current(){return activeCaller;}
BoundPipePeer::~BoundPipePeer(){if(process_)CloseHandle(process_);}
DWORD BoundPipePeer::bind(HANDLE pipe,HANDLE original,const CallerIdentity& allowed,
                         std::shared_ptr<const BoundPipePeer>* out) {
    if(!out || !original || original==INVALID_HANDLE_VALUE)return ERROR_INVALID_PARAMETER;
    out->reset();
    auto peer=std::shared_ptr<BoundPipePeer>(new BoundPipePeer);
    // Do not request DUP_HANDLE here. Authentication does not authorize descriptor
    // delegation; that requires its own transaction/ownership contract.
    if(!DuplicateHandle(GetCurrentProcess(),original,GetCurrentProcess(),&peer->process_,
                        SYNCHRONIZE|PROCESS_QUERY_LIMITED_INFORMATION,FALSE,0))return GetLastError();
    peer->pid_=GetProcessId(peer->process_);if(!peer->pid_)return GetLastError();
    FILETIME exit{},kernel{},user{};
    if(!GetProcessTimes(peer->process_,&peer->creation_,&exit,&kernel,&user))return GetLastError();
    DWORD status=processIdentity(peer->process_,&peer->identity_);if(status)return status;
    if(!peer->identity_.sameAs(allowed))return ERROR_ACCESS_DENIED;
    status=peer->validate(pipe,allowed);if(status)return status;
    *out=std::move(peer);return ERROR_SUCCESS;
}
DWORD BoundPipePeer::validate(HANDLE pipe,const CallerIdentity& request) const {
    if(!request.sameAs(identity_))return ERROR_ACCESS_DENIED;
    DWORD status=requireAlive(process_);if(status)return status;
    ULONG pipePid=0;if(!GetNamedPipeClientProcessId(pipe,&pipePid))return GetLastError();
    if(pipePid!=pid_ || GetProcessId(process_)!=pid_)return ERROR_ACCESS_DENIED;
    FILETIME creation{},exit{},kernel{},user{};
    if(!GetProcessTimes(process_,&creation,&exit,&kernel,&user))return GetLastError();
    if(CompareFileTime(&creation,&creation_)!=0)return ERROR_ACCESS_DENIED;
    CallerIdentity current;status=processIdentity(process_,&current);if(status)return status;
    if(!current.sameAs(identity_))return ERROR_ACCESS_DENIED;
    return requireAlive(process_);
}
DWORD authenticatePipeRequest(HANDLE pipe,const CallerIdentity& allowed,
                              const BoundPipePeer* peer,CallerIdentity* out) {
    if(!out)return ERROR_INVALID_PARAMETER;
    *out={};
    // Reject an already impersonating dispatch thread; do not destroy somebody
    // else's token and do not authenticate using an ambient process fallback.
    Handle prior;
    if(OpenThreadToken(GetCurrentThread(),TOKEN_QUERY,TRUE,&prior.value))return ERROR_BAD_IMPERSONATION_LEVEL;
    DWORD error=GetLastError();if(error!=ERROR_NO_TOKEN)return error;
    Impersonation scope;
    if(!ImpersonateNamedPipeClient(pipe))return GetLastError();
    scope.active=true;
    Handle token;
    if(!OpenThreadToken(GetCurrentThread(),TOKEN_QUERY|TOKEN_DUPLICATE,TRUE,&token.value))return GetLastError();
    SECURITY_IMPERSONATION_LEVEL level=SecurityAnonymous;DWORD bytes=0;
    if(!GetTokenInformation(token.value,TokenImpersonationLevel,&level,sizeof(level),&bytes))return GetLastError();
    if(level<SecurityIdentification)return ERROR_BAD_IMPERSONATION_LEVEL;
    Handle snapshot;
    if(!DuplicateTokenEx(token.value,TOKEN_QUERY,nullptr,SecurityIdentification,
                         TokenImpersonation,&snapshot.value))return GetLastError();
    DWORD flags=0;
    if(!GetHandleInformation(snapshot.value,&flags))return GetLastError();
    if(flags&HANDLE_FLAG_INHERIT)return ERROR_ACCESS_DENIED;
    CallerIdentity request;error=readIdentity(snapshot.value,&request);
    scope.revert();
    if(error)return error;
    if(!request.sameAs(allowed))return ERROR_ACCESS_DENIED;
    if(peer){error=peer->validate(pipe,request);if(error)return error;}
    request.queryToken_=shareOwnedHandle(snapshot);
    *out=std::move(request);return ERROR_SUCCESS;
}
}
