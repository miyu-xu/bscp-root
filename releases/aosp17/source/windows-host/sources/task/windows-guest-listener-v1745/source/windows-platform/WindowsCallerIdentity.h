#pragma once
#include <windows.h>
#include <memory>
#include <vector>
#include <optional>
namespace bscp::windows {
class BoundPipePeer;
class WindowsNdrRequestSnapshot;
// Windows SID/logon identity is not a Unix uid. Never narrow or invent one.
struct CallerIdentity {
    std::vector<unsigned char> sid;
    LUID logon{};
    bool sameAs(const CallerIdentity& other) const;
    static DWORD currentProcess(CallerIdentity* out);
    // Only an authenticated NDR snapshot proves an actual calling process.
    DWORD checkedCallingPid(DWORD* out) const;
    // Read-only TOKEN_QUERY snapshot of the actual request impersonation token.
    // currentProcess() intentionally does not create one: no server-token fallback.
    HANDLE queryTokenForAccessCheck() const { return queryToken_.get(); }
private:
    std::shared_ptr<void> queryToken_;
    std::shared_ptr<const WindowsNdrRequestSnapshot> ndrSnapshot_;
    friend DWORD importAuthenticatedWindowsNdrSnapshot(
        const std::shared_ptr<const WindowsNdrRequestSnapshot>&,
        const CallerIdentity&, CallerIdentity*);
    friend DWORD authenticatePipeRequest(HANDLE,const CallerIdentity&,const BoundPipePeer*,CallerIdentity*);
};
// Dispatch-only scope. Supply a snapshot already authenticated for this command;
// nullptr explicitly clears an outer caller for an unbound callback. A queued one-way
// command must own its snapshot; querying the pipe later would see another message.
class CallerScope final {
    std::optional<CallerIdentity> identity_;
    const CallerIdentity* previous_;
public:
    explicit CallerScope(const CallerIdentity* authenticated);
    ~CallerScope();
    CallerScope(const CallerScope&)=delete;
    CallerScope& operator=(const CallerScope&)=delete;
    static const CallerIdentity* current();
};
class BoundPipePeer final {
    HANDLE process_ = nullptr;
    DWORD pid_ = 0;
    FILETIME creation_{};
    CallerIdentity identity_;
    BoundPipePeer() = default;
public:
    ~BoundPipePeer();
    BoundPipePeer(const BoundPipePeer&) = delete;
    BoundPipePeer& operator=(const BoundPipePeer&) = delete;
    // originalProcess must be a process HANDLE already owned by the supervisor, not
    // obtained by resolving a PID read from the pipe/request. The returned owner holds
    // a duplicate of that exact process object, preventing PID-reuse substitution.
    static DWORD bind(HANDLE serverPipe, HANDLE originalProcess,
                      const CallerIdentity& allowed,
                      std::shared_ptr<const BoundPipePeer>* out);
    DWORD validate(HANDLE serverPipe, const CallerIdentity& request) const;
    HANDLE processHandle() const { return process_; } // borrowed, held by shared owner
    DWORD pid() const { return pid_; }
};
// Call after reading the complete request on the server end, before dispatch. Caller
// must serialize reads + authentication on this connection: impersonation describes
// the last message read. No creator-token fallback and no ambient impersonation.
// A failed RevertToSelf is fail-stop; execution must never continue under client token.
DWORD authenticatePipeRequest(HANDLE serverPipe, const CallerIdentity& allowed,
                              const BoundPipePeer* boundPeer, CallerIdentity* out);
}
