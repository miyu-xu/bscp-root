#pragma once
#include <binder/RpcTransport.h>
#include "WindowsNdrReceivedPacket.h"
#include "WindowsInitialOwner.h"
#include <atomic>
namespace android {
class RpcTransportNdr final : public RpcTransport {
public:
 // Consumes only a freshly accepted authenticated opaque original channel.
 // No endpoint-string, numeric-ID or untrusted ClientBound construction exists.
 static status_t tryAccept(WlrServerAcceptor*, std::unique_ptr<RpcTransportNdr>*);
 static status_t connectInitial(std::shared_ptr<bscp::windows::WindowsInitialOwner>,std::unique_ptr<RpcTransportNdr>*);
     uint64_t authenticatedSessionGeneration() const noexcept;
~RpcTransportNdr() override;
 NdrDirection ndrDirection()const noexcept override;
 status_t beginNdrRecord(NdrPhase)override;
 status_t finishNdrRecord(std::shared_ptr<const bscp::windows::WindowsNdrRequestSnapshot>*)override;
 status_t pollRead()override;
 status_t interruptableWriteFully(FdTrigger*,iovec*,int,const std::optional<binder::impl::SmallFunction<status_t()>>&,const std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*)override;
 status_t interruptableReadFully(FdTrigger*,iovec*,int,const std::optional<binder::impl::SmallFunction<status_t()>>&,std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*)override;
 bool isWaiting()override{return mWaiting.load()!=0;}
 status_t shutdown();
 status_t closeInitialChannel();
 status_t shutdownStatus() const noexcept {return mStopped?mShutdownResult.load():INVALID_OPERATION;}
private:
 struct Owner;
 explicit RpcTransportNdr(std::shared_ptr<Owner>);
 status_t loadPacket();
 status_t validateShape();
 status_t fail(status_t);
 status_t pause(FdTrigger*,const std::optional<binder::impl::SmallFunction<status_t()>>&);
 std::shared_ptr<Owner> mOwner;
 std::shared_ptr<bscp::windows::WindowsNdrReceivedPacket> mPacket;
 std::atomic<bool> mStopped{false},mReading{false},mWriting{false};
 std::atomic<unsigned> mWaiting{0};
 std::atomic<status_t> mShutdownResult{WOULD_BLOCK};
 bool mPhaseOpen=false,mDescriptorsDone=false;
 NdrPhase mPhase=NdrPhase::Command;
 size_t mReadBytes=0;
 uint64_t mNextRead=1,mNextWrite=1;
 unsigned char mHeader[16]{};
};
}

