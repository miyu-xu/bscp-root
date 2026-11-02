#pragma once
#include "WindowsPipeTransport.h"
namespace android {
// Legacy Host-local frame envelope: LE32 payload length, LE32 handle count,
// then handle values and payload. This implementation admits byte-only frames.
// Nonzero handle counts are rejected without interpreting or closing wire values.
// It must not be advertised as descriptor-capable transport.
class RpcTransportWindowsPipeFrame final : public RpcTransport {
public:
    static constexpr uint32_t kMaxFrameBytes = 64 * 1024 * 1024;
    explicit RpcTransportWindowsPipeFrame(std::unique_ptr<RpcTransportWindowsPipe> bytes);
    status_t pollRead() override;
    status_t interruptableWriteFully(FdTrigger*,iovec*,int,
        const std::optional<binder::impl::SmallFunction<status_t()>>&,
        const std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*) override;
    status_t interruptableReadFully(FdTrigger*,iovec*,int,
        const std::optional<binder::impl::SmallFunction<status_t()>>&,
        std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*) override;
    bool isWaiting() override { return mBytes && mBytes->isWaiting(); }
    bool poisoned() const { return mPoisoned; }
    HANDLE borrowedHandle() const { return mBytes?mBytes->borrowedHandle():INVALID_HANDLE_VALUE; }
private:
    status_t fail(status_t status){mPoisoned=true;return status;}
    std::unique_ptr<RpcTransportWindowsPipe> mBytes;
    uint32_t mRemaining=0;
    bool mPoisoned=false;
};
}
