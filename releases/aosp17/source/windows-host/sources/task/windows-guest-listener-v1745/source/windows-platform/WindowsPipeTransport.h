#pragma once
#include <windows.h>
#include <atomic>
#include <binder/RpcTransport.h>
#include "WindowsOverlappedPipeFd.h"
namespace android {
// Byte-stream transport only. No framing changes and no implicit FD/identity transfer.
// adoptOverlapped consumes a uniquely owned, noninheritable, FILE_FLAG_OVERLAPPED
// local connected byte-mode named-pipe endpoint; native mode is revalidated. The caller must create/open it with those flags and
// must not keep another owner or issue I/O on it after adoption.
class RpcTransportWindowsPipe final : public RpcTransport {
public:
    static std::unique_ptr<RpcTransportWindowsPipe> adoptOverlapped(HANDLE pipe);
    static std::unique_ptr<RpcTransportWindowsPipe> fromRegistered(bscp::windows::OverlappedPipeLease owner);
    ~RpcTransportWindowsPipe() override;
    status_t pollRead() override;
    status_t interruptableWriteFully(FdTrigger*, iovec*, int,
        const std::optional<binder::impl::SmallFunction<status_t()>>&,
        const std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*) override;
    status_t interruptableReadFully(FdTrigger*, iovec*, int,
        const std::optional<binder::impl::SmallFunction<status_t()>>&,
        std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*) override;
    bool isWaiting() override { return mWaiting.load(std::memory_order_acquire); }
    // Borrowed only for a synchronous post-read caller-authentication operation.
    // No close, I/O, impersonation lifetime escape, or storage beyond this owner.
    HANDLE borrowedHandle() const { return mPipe; }
private:
    explicit RpcTransportWindowsPipe(HANDLE pipe) : mPipe(pipe) {}
    status_t transfer(bool write, FdTrigger*, iovec*, int,
        const std::optional<binder::impl::SmallFunction<status_t()>>&);
    explicit RpcTransportWindowsPipe(bscp::windows::OverlappedPipeLease owner)
        : mPipe(bscp::windows::borrowedOverlappedPipe(owner)),mOwner(std::move(owner)) {}
    HANDLE mPipe;
    bscp::windows::OverlappedPipeLease mOwner;
    std::atomic<bool> mBusy{false};
    std::atomic<bool> mWaiting{false};
};
}
