#include "WindowsAccessControl.h"
#include <aclapi.h>
#include <vector>
#include <cstring>
namespace bscp::windows {
namespace {
struct LocalMemory {void* value=nullptr;~LocalMemory(){if(value)LocalFree(value);}};
struct ObjectBasicInformation {
    ULONG attributes;ACCESS_MASK grantedAccess;ULONG handleCount,pointerCount;ULONG reserved[10];
};
DWORD handleAccess(HANDLE object,ACCESS_MASK* out) {
    // Documented NtQueryObject ObjectBasicInformation, dynamically resolved from the
    // already loaded system ntdll. Unknown API/status fails closed; no private fields.
    using Query=LONG (NTAPI*)(HANDLE,ULONG,void*,ULONG,ULONG*);
    HMODULE module=GetModuleHandleW(L"ntdll.dll");if(!module)return GetLastError();
    FARPROC address=GetProcAddress(module,"NtQueryObject");Query query=nullptr;
    static_assert(sizeof(query)==sizeof(address));std::memcpy(&query,&address,sizeof(query));
    if(!query)return ERROR_NOT_SUPPORTED;
    ObjectBasicInformation info{};ULONG length=0;
    LONG status=query(object,0,&info,sizeof(info),&length);
    if(status<0 || length<sizeof(info))return ERROR_INVALID_DATA;
    *out=info.grantedAccess;return ERROR_SUCCESS;
}
bool trustedSystemSid(PSID sid) {
    return IsWellKnownSid(sid,WinLocalSystemSid)||IsWellKnownSid(sid,WinBuiltinAdministratorsSid);
}
GENERIC_MAPPING fileMapping(){return {FILE_GENERIC_READ,FILE_GENERIC_WRITE,FILE_GENERIC_EXECUTE,FILE_ALL_ACCESS};}
DWORD noForeignWrite(PACL acl,PSID owner) {
    if(!acl||!IsValidAcl(acl))return ERROR_INVALID_ACL;
    for(DWORD i=0;i<acl->AceCount;++i){
        void* raw=nullptr;if(!GetAce(acl,i,&raw))return GetLastError();
        auto header=static_cast<ACE_HEADER*>(raw);
        if(header->AceType==ACCESS_DENIED_ACE_TYPE)continue;
        if(header->AceType!=ACCESS_ALLOWED_ACE_TYPE)return ERROR_NOT_SUPPORTED;
        auto ace=static_cast<ACCESS_ALLOWED_ACE*>(raw);
        if(header->AceSize<sizeof(ACCESS_ALLOWED_ACE))return ERROR_INVALID_ACL;
        auto sid=reinterpret_cast<PSID>(&ace->SidStart);
        // IsValidAcl validates ACE layout; also bound the variable SID before access.
        auto bytes=static_cast<unsigned char*>(sid);
        size_t available=header->AceSize-FIELD_OFFSET(ACCESS_ALLOWED_ACE,SidStart);
        if(available<8 || bytes[1]>SID_MAX_SUB_AUTHORITIES || 8+4*size_t(bytes[1])>available || !IsValidSid(sid))return ERROR_INVALID_SID;
        ACCESS_MASK mask=ace->Mask;auto mapping=fileMapping();MapGenericMask(&mask,&mapping);
        constexpr ACCESS_MASK writes=FILE_WRITE_DATA|FILE_APPEND_DATA|FILE_WRITE_EA|FILE_WRITE_ATTRIBUTES|
                                     FILE_DELETE_CHILD|DELETE|WRITE_DAC|WRITE_OWNER;
        if((mask&writes) && !EqualSid(owner,sid) && !trustedSystemSid(sid))return ERROR_ACCESS_DENIED;
    }
    return ERROR_SUCCESS;
}
}
DWORD checkCurrentCallerFile(HANDLE file,FileAccessKind kind,FileAccessEvidence* out) {
    if(!out)return ERROR_INVALID_PARAMETER;*out={};
    const CallerIdentity* caller=CallerScope::current();
    if(!caller || !caller->queryTokenForAccessCheck())return ERROR_NO_TOKEN;
    if(!file || file==INVALID_HANDLE_VALUE)return ERROR_INVALID_HANDLE;
    DWORD type=GetFileType(file);if(type!=FILE_TYPE_DISK)return ERROR_INVALID_DATA;
    BY_HANDLE_FILE_INFORMATION info{};if(!GetFileInformationByHandle(file,&info))return GetLastError();
    if(info.dwFileAttributes&FILE_ATTRIBUTE_REPARSE_POINT)return ERROR_ACCESS_DENIED;
    bool directory=(info.dwFileAttributes&FILE_ATTRIBUTE_DIRECTORY)!=0;
    if(directory!=(kind==FileAccessKind::ReadOwnedDirectory))return ERROR_INVALID_DATA;
    ACCESS_MASK desired;
    switch(kind){
    case FileAccessKind::ReadImage:case FileAccessKind::ReadOwnedDirectory:desired=FILE_READ_DATA;break;
    case FileAccessKind::WriteInstance:desired=FILE_READ_DATA|FILE_WRITE_DATA;break;
    default:return ERROR_INVALID_PARAMETER;
    }
    ACCESS_MASK handleGranted=0;DWORD status=handleAccess(file,&handleGranted);if(status)return status;
    if((handleGranted&desired)!=desired)return ERROR_ACCESS_DENIED;
    PSID owner=nullptr,group=nullptr;PACL dacl=nullptr;PSECURITY_DESCRIPTOR descriptor=nullptr;
    status=GetSecurityInfo(file,SE_FILE_OBJECT,OWNER_SECURITY_INFORMATION|GROUP_SECURITY_INFORMATION|DACL_SECURITY_INFORMATION,
                          &owner,&group,&dacl,nullptr,&descriptor);
    LocalMemory memory{descriptor};if(status)return status;
    if(!descriptor || !IsValidSecurityDescriptor(descriptor)||!owner||!group||!IsValidSid(owner)||!IsValidSid(group))return ERROR_INVALID_SECURITY_DESCR;
    BOOL present=FALSE,defaulted=FALSE;PACL checked=nullptr;
    if(!GetSecurityDescriptorDacl(descriptor,&present,&checked,&defaulted))return GetLastError();
    if(!present||!checked||checked!=dacl)return ERROR_INVALID_ACL;
    if(!caller->sameAs(*caller))return ERROR_INVALID_SID;
    bool own=EqualSid(owner,const_cast<unsigned char*>(caller->sid.data()))!=FALSE;
    if(!own && (kind!=FileAccessKind::ReadImage || !trustedSystemSid(owner)))return ERROR_ACCESS_DENIED;
    status=noForeignWrite(dacl,owner);if(status)return status;
    auto mapping=fileMapping();ACCESS_MASK callerGranted=0;BOOL allowed=FALSE;
    std::vector<unsigned char> privileges(sizeof(PRIVILEGE_SET)+8*sizeof(LUID_AND_ATTRIBUTES));
    DWORD privilegeBytes=static_cast<DWORD>(privileges.size());
    if(!AccessCheck(descriptor,caller->queryTokenForAccessCheck(),desired,&mapping,
                    reinterpret_cast<PRIVILEGE_SET*>(privileges.data()),&privilegeBytes,&callerGranted,&allowed)){
        status=GetLastError();
        if(status!=ERROR_INSUFFICIENT_BUFFER || privilegeBytes>65536)return status;
        privileges.resize(privilegeBytes);
        if(!AccessCheck(descriptor,caller->queryTokenForAccessCheck(),desired,&mapping,
                        reinterpret_cast<PRIVILEGE_SET*>(privileges.data()),&privilegeBytes,&callerGranted,&allowed))return GetLastError();
    }
    if(!allowed || (callerGranted&desired)!=desired)return ERROR_ACCESS_DENIED;
    out->volumeSerial=info.dwVolumeSerialNumber;
    out->fileIndex=(std::uint64_t(info.nFileIndexHigh)<<32)|info.nFileIndexLow;
    out->links=info.nNumberOfLinks;out->attributes=info.dwFileAttributes;
    out->handleGranted=handleGranted;out->callerGranted=callerGranted;
    return ERROR_SUCCESS;
}
}
