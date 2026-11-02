#include "WindowsIdentityAbi.h"
#include "WindowsCallerIdentity.h"
#include "WindowsAccessControl.h"
#include <cstring>
#include <limits>
#include <new>
static_assert(sizeof(BscpFileEvidence)==32);
extern "C" DWORD bscp_rpc_current_identity(unsigned char* sid,uint32_t capacity,
 uint32_t* required,uint32_t* low,int32_t* high){
 if(required)*required=0;if(low)*low=0;if(high)*high=0;
 if(!required||!low||!high)return ERROR_INVALID_PARAMETER;
 const auto* caller=bscp::windows::CallerScope::current();
 if(!caller || !caller->queryTokenForAccessCheck())return ERROR_NO_TOKEN;
 if(!caller->sameAs(*caller))return ERROR_INVALID_SID;
 if(caller->sid.size()>std::numeric_limits<uint32_t>::max())return ERROR_INVALID_SID;
 *required=static_cast<uint32_t>(caller->sid.size());
 if(!sid||capacity<*required)return ERROR_INSUFFICIENT_BUFFER;
 std::memcpy(sid,caller->sid.data(),*required);*low=caller->logon.LowPart;*high=caller->logon.HighPart;
 return ERROR_SUCCESS;
}
extern "C" DWORD bscp_rpc_check_file(HANDLE file,uint32_t kind,BscpFileEvidence* out,uint32_t size){
 if(!out || size!=sizeof(BscpFileEvidence))return ERROR_INVALID_PARAMETER;
 *out={};if(kind>2)return ERROR_INVALID_PARAMETER;
 try {
  bscp::windows::FileAccessEvidence evidence;
  DWORD result=bscp::windows::checkCurrentCallerFile(file,static_cast<bscp::windows::FileAccessKind>(kind),&evidence);
  if(result)return result;
  *out={1,evidence.volumeSerial,evidence.fileIndex,evidence.links,evidence.attributes,evidence.handleGranted,evidence.callerGranted};
  return ERROR_SUCCESS;
 }catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
 catch(...){return ERROR_GEN_FAILURE;}
}

extern "C" DWORD bscp_rpc_check_owner(){
 try {
  const auto* caller=bscp::windows::CallerScope::current();
  if(!caller || !caller->queryTokenForAccessCheck())return ERROR_NO_TOKEN;
  bscp::windows::CallerIdentity owner;
  DWORD status=bscp::windows::CallerIdentity::currentProcess(&owner);if(status)return status;
  return caller->sameAs(owner)?ERROR_SUCCESS:ERROR_ACCESS_DENIED;
 }catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
 catch(...){return ERROR_GEN_FAILURE;}
}
