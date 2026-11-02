#include "DesktopRpcIdentity.h"
#include <cerrno>
#include <fcntl.h>
#include <poll.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <utility>
#ifndef SO_PEERPIDFD
// Linux UAPI asm-generic/socket.h. Unsupported kernels fail closed at getsockopt.
#define SO_PEERPIDFD 77
#endif
namespace android::desktop_rpc {
class ProcessHandle {
    int fd;
public:
    explicit ProcessHandle(int value):fd(value){}
    ~ProcessHandle(){if(fd>=0)close(fd);}
    ProcessHandle(const ProcessHandle&)=delete;
    ProcessHandle& operator=(const ProcessHandle&)=delete;
    bool alive() const {
        pollfd item{fd,POLLIN,0};
        // EINTR, HUP, invalid handle and all query errors deny; no unbounded retry.
        return fd>=0 && poll(&item,1,0)==0 && item.revents==0;
    }
};
bool Peer::alive() const {return handle && handle->alive();}
bool Peer::operator==(const Peer& other) const {
    // Both original pidfds must remain live. A retained exited pidfd never becomes
    // live on PID reuse; numeric PID equality alone never admits an attachment.
    return process==other.process && user==other.user && group==other.group &&
           alive() && other.alive() && alive();
}
static thread_local std::optional<Peer> active;
Scope::Scope(const std::optional<Peer>& peer):previous(active){active=peer;}
Scope::~Scope(){active=previous;}
std::optional<Peer> current(){if(active && active->alive())return active;return std::nullopt;}
int capture(int fd,uid_t owner,gid_t group,Peer* result){
    if(!result || owner==0 || owner!=getuid() || owner!=geteuid() || group!=getgid() || group!=getegid())return EACCES;
    int kind=0;socklen_t size=sizeof(kind);
    if(getsockopt(fd,SOL_SOCKET,SO_TYPE,&kind,&size)!=0)return errno;
    if(size!=sizeof(kind) || kind!=SOCK_STREAM)return EPROTOTYPE;
    sockaddr_storage address{};size=sizeof(address);
    if(getpeername(fd,reinterpret_cast<sockaddr*>(&address),&size)!=0)return errno;
    if(size<sizeof(sa_family_t) || address.ss_family!=AF_UNIX)return EAFNOSUPPORT;
    ucred credentials{};size=sizeof(credentials);
    if(getsockopt(fd,SOL_SOCKET,SO_PEERCRED,&credentials,&size)!=0)return errno;
    if(size!=sizeof(credentials) || credentials.pid<=0 || credentials.uid!=owner || credentials.gid!=group)return EACCES;
    int original=-1;size=sizeof(original);
    if(getsockopt(fd,SOL_SOCKET,SO_PEERPIDFD,&original,&size)!=0)return errno;
    // Guard the received descriptor even if allocation throws.
    struct Guard { int fd; ~Guard(){if(fd>=0)close(fd);} } guard{original};
    auto handle=std::make_shared<ProcessHandle>(original);
    guard.fd=-1;
    if(size!=sizeof(original) || original<0)return EPROTO;
    int flags=fcntl(original,F_GETFD);
    if(flags<0)return errno;
    if(!(flags&FD_CLOEXEC))return EPROTO;
    Peer candidate{credentials.pid,credentials.uid,credentials.gid,std::move(handle)};
    if(!candidate.alive())return ESRCH;
    *result=std::move(candidate);return 0;
}
}
extern "C" int BSCP_RpcCallingLinuxCredentials(int* pid,unsigned* uid,unsigned* gid){
    if(!pid || !uid || !gid)return EINVAL;
    auto peer=android::desktop_rpc::current();if(!peer)return ENOENT;
    *pid=peer->process;*uid=peer->user;*gid=peer->group;return 0;
}
