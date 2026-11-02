#include "WindowsCallerIdentity.h"
#include "WindowsNdrRecordAdapter.h"
#include <new>
namespace bscp::windows {
DWORD importAuthenticatedWindowsNdrSnapshot(
        const std::shared_ptr<const WindowsNdrRequestSnapshot>& proof,
        const CallerIdentity& allowed, CallerIdentity* out) {
    if (!out) return ERROR_INVALID_PARAMETER;
    *out = {};
    if (!proof) return ERROR_NO_TOKEN;
    DWORD pid = 0;
    DWORD status = proof->checkedCallingPid(&pid);
    if (status != ERROR_SUCCESS) return status;
    if (!pid || !proof->token_ || proof->token_ == INVALID_HANDLE_VALUE)
        return ERROR_INVALID_DATA;
    try {
        CallerIdentity result;
        result.sid = proof->sid_;
        result.logon = proof->logon_;
        if (!result.sameAs(allowed)) return ERROR_ACCESS_DENIED;
        // Aliasing ownership keeps the exact authenticated snapshot alive;
        // no duplicate closer and no server-token fallback.
        result.queryToken_ = std::shared_ptr<void>(proof, proof->token_);
        result.ndrSnapshot_ = proof;
        status = proof->checkedCallingPid(&pid);
        if (status != ERROR_SUCCESS) return status;
        *out = std::move(result);
        return ERROR_SUCCESS;
    } catch (const std::bad_alloc&) {
        return ERROR_NOT_ENOUGH_MEMORY;
    }
}
DWORD CallerIdentity::checkedCallingPid(DWORD* out) const {
    if (!out) return ERROR_INVALID_PARAMETER;
    *out = 0;
    if (!ndrSnapshot_) return ERROR_NO_TOKEN;
    return ndrSnapshot_->checkedCallingPid(out);
}
}
