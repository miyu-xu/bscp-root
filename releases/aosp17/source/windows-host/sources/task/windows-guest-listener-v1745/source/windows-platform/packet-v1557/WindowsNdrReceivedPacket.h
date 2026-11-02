#pragma once
#include "WindowsNdrRecordAdapter.h"
#include "WindowsNdrPacketCodec.h"
namespace android { class RpcTransportNdr; }
namespace bscp::windows {
#ifdef BSCP_WLR_ISOLATED_COMPONENT
class WindowsNdrPacketTestHarness;
void windowsNdrPacketTestFailControlBlock();
#endif
class WindowsNdrReceivedPacket final {
public:
 enum class Role { ServerInbound, ClientReply };
 ~WindowsNdrReceivedPacket()=default;
 static DWORD tryPopAccepted(WlrAcceptedChannel*,std::shared_ptr<WindowsNdrReceivedPacket>*);
 static DWORD tryReceive(void* clientChannel,uint64_t generation,std::shared_ptr<WindowsNdrReceivedPacket>*);
 WindowsNdrReceivedPacket(const WindowsNdrReceivedPacket&)=delete;
 WindowsNdrReceivedPacket& operator=(const WindowsNdrReceivedPacket&)=delete;
 Role role()const{return role_;}
 size_t payloadSize()const{return payloadSize_;}
 size_t handleCount()const{return handles_.size();}
 DWORD readPayloadExact(std::span<unsigned char>);
 // A successful claim returns a new noninheritable owned HANDLE exactly once.
 // Caller must close it on later CRT adoption/dispatch failure. Original typed
 // record handles remain owned until finish/destruction; there is no raw getter.
 DWORD takeDuplicatedHandle(size_t index,HANDLE* owned);
 // Explicit no-ancillary disposition; original resources close at finish/dtor.
 DWORD discardRemainingHandles();
private:
 struct Release {void operator()(WlrRecord* r)const{WlrRecordRelease(r);}};
 Role role_;
 std::shared_ptr<WindowsNdrReceivedRecord> inbound_;
 std::unique_ptr<WlrRecord,Release> reply_;
 std::vector<unsigned char> encoded_;
 std::vector<HANDLE> handles_;
 std::vector<unsigned char> disposed_;
 size_t payloadOffset_=0,payloadSize_=0,consumed_=0;
 bool failed_=false,finished_=false;
 explicit WindowsNdrReceivedPacket(Role r):role_(r){}
 DWORD decodeOwned(const WlrView&);
 // Only the production transport's reviewed phase hooks may call this AFTER
 // upstream Binder/handshake validation. It does not invent protocol validity.
 DWORD finishValidatedPhase(std::shared_ptr<const WindowsNdrRequestSnapshot>*);
 friend class ::android::RpcTransportNdr;
#ifdef BSCP_WLR_ISOLATED_COMPONENT
 friend class WindowsNdrPacketTestHarness;
#endif
};
}

