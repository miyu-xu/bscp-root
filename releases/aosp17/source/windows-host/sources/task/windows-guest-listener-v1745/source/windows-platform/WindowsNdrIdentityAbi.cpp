#include "WindowsCallerIdentity.h"
#include "WindowsIdentityAbi.h"
// Exposes only the authenticated record's original process identity.
extern "C" DWORD bscp_rpc_current_pid(DWORD* out) {
    if (!out) return ERROR_INVALID_PARAMETER;
    *out = 0;
    const auto* caller = bscp::windows::CallerScope::current();
    if (!caller || !caller->queryTokenForAccessCheck()) return ERROR_NO_TOKEN;
    const DWORD owner = bscp_rpc_check_owner();
    if (owner != ERROR_SUCCESS) return owner;
    return caller->checkedCallingPid(out);
}
