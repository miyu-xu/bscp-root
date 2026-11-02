#pragma once
// Linux desktop RPC transport identity. No Guest/wire identity fields.
#include <memory>
#include <optional>
#include <sys/types.h>
namespace android::desktop_rpc {
class ProcessHandle;
struct __attribute__((visibility("default"))) Peer {
    pid_t process = -1;
    uid_t user = 0;
    gid_t group = 0;
    std::shared_ptr<const ProcessHandle> handle;
    bool alive() const;
    bool operator==(const Peer& other) const;
};
__attribute__((visibility("default"))) int capture(int fd, uid_t owner, gid_t group, Peer* result);
class __attribute__((visibility("default"))) Scope {
    std::optional<Peer> previous;
public:
    explicit Scope(const std::optional<Peer>& peer);
    ~Scope();
    Scope(const Scope&)=delete;
    Scope& operator=(const Scope&)=delete;
};
__attribute__((visibility("default"))) std::optional<Peer> current();
}
extern "C" __attribute__((visibility("default"))) int BSCP_RpcCallingLinuxCredentials(int* pid, unsigned* uid, unsigned* gid);
