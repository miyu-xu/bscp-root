#include "RpcTransportNdr.h"
#include "FdTrigger.h"
#include "RpcWireFormat.h"
#include "WindowsFdAbi.h"
#include <algorithm>
#include <cerrno>
#include <cstring>
#include <limits>
#include <new>
#include <cstdio>
namespace android {
using bscp::windows::WindowsNdrReceivedPacket;
namespace {
status_t error(DWORD e){switch(e){case ERROR_SUCCESS:return OK;case ERROR_NO_MORE_ITEMS:case ERROR_BUSY:return WOULD_BLOCK;case ERROR_OPERATION_ABORTED:case ERROR_INVALID_HANDLE:case ERROR_PROCESS_ABORTED:return DEAD_OBJECT;case ERROR_ACCESS_DENIED:return PERMISSION_DENIED;case ERROR_NOT_ENOUGH_MEMORY:case ERROR_OUTOFMEMORY:return NO_MEMORY;case ERROR_INVALID_DATA:case ERROR_INVALID_PARAMETER:return BAD_VALUE;default:return UNKNOWN_ERROR;}}
struct Guard {std::atomic<bool>& busy;~Guard(){busy=false;}};
struct Waiting {std::atomic<unsigned>& count;explicit Waiting(std::atomic<unsigned>& c):count(c){++count;}~Waiting(){--count;}};
struct Handles {std::vector<HANDLE> values;bool close(){bool ok=true;for(auto& h:values)if(h){HANDLE x=h;h=nullptr;if(!CloseHandle(x))ok=false;}return ok;}~Handles(){close();}};
bool iovSize(iovec* v,int count,size_t* total){*total=0;if(count<0||(!v&&count))return false;for(int i=0;i<count;++i){if((!v[i].iov_base&&v[i].iov_len)||v[i].iov_len>bscp::windows::kNdrBinderPacketMax-*total)return false;*total+=v[i].iov_len;}return true;}
}
struct RpcTransportNdr::Owner {
 WlrAcceptedChannel* channel=nullptr;InitialChannel* clientChannel=nullptr;
 std::shared_ptr<bscp::windows::WindowsInitialOwner> initial;
 DWORD clientCloseStatus=ERROR_IO_PENDING;HANDLE wake=nullptr,process=nullptr;
 DWORD pid=0;uint64_t generation=0;
 Owner* retainedNext=nullptr;
 inline static std::atomic<Owner*> retainedHead{nullptr};
 static void dispose(Owner* p) noexcept {
  // Final drop has no active borrowers. A failed close preserves the exact
  // channel/root/status object without allocation or implicit retry.
  if(p->initial){
   if(p->clientChannel&&p->clientCloseStatus==ERROR_IO_PENDING)
    p->clientCloseStatus=InitialChannelClose(&p->clientChannel);
   if(p->clientChannel||(p->clientCloseStatus!=ERROR_SUCCESS&&p->clientCloseStatus!=ERROR_IO_PENDING)){
    auto* head=retainedHead.load();
    do{p->retainedNext=head;}while(!retainedHead.compare_exchange_weak(head,p));
    std::fprintf(stderr,"Initial transport retained UNKNOWN error=%lu\n",p->clientCloseStatus);return;
   }
  }
  delete p;
 }
 ~Owner(){if(channel){DWORD e=WlrAcceptedClose(&channel);if(e)std::fprintf(stderr,"NDR owner close UNKNOWN error=%lu\n",e);}if(wake&&!CloseHandle(wake))std::fprintf(stderr,"NDR wake close UNKNOWN\n");}
};
RpcTransportNdr::RpcTransportNdr(std::shared_ptr<Owner> owner):mOwner(std::move(owner)){}
status_t RpcTransportNdr::tryAccept(WlrServerAcceptor* acceptor,std::unique_ptr<RpcTransportNdr>* out){if(!out)return BAD_VALUE;out->reset();WlrAcceptedChannel* raw=nullptr;DWORD e=WlrAcceptorTryAccept(acceptor,&raw);if(e)return error(e);
 try {std::unique_ptr<Owner> owner(new Owner);owner->channel=raw;raw=nullptr;HANDLE token=nullptr;e=WlrAcceptedOwnerProof(owner->channel,&token,&owner->process,&owner->pid,&owner->generation);if(e||!token)return error(e?e:ERROR_ACCESS_DENIED);owner->wake=CreateEventW(nullptr,TRUE,FALSE,nullptr);if(!owner->wake)return UNKNOWN_ERROR;auto held=std::shared_ptr<Owner>(std::move(owner));out->reset(new RpcTransportNdr(std::move(held)));return OK;}catch(const std::bad_alloc&){if(raw)WlrAcceptedClose(&raw);return NO_MEMORY;}
}
status_t RpcTransportNdr::connectInitial(std::shared_ptr<bscp::windows::WindowsInitialOwner> initial,std::unique_ptr<RpcTransportNdr>* out) {
 if(!out)return BAD_VALUE;out->reset();if(!initial)return PERMISSION_DENIED;
 try {auto owner=std::shared_ptr<Owner>(new Owner,&Owner::dispose);owner->initial=std::move(initial);
 DWORD e=owner->initial->connect(&owner->clientChannel);if(e)return error(e);
 e=InitialChannelGeneration(owner->clientChannel,&owner->generation);if(e)return error(e);
 owner->wake=CreateEventW(nullptr,TRUE,FALSE,nullptr);if(!owner->wake)return UNKNOWN_ERROR;
 out->reset(new RpcTransportNdr(std::move(owner)));return OK;
 }catch(const std::bad_alloc&){return NO_MEMORY;}
}
status_t RpcTransportNdr::closeInitialChannel() {
 if(!mOwner->initial)return INVALID_OPERATION;
 if(mReading||mWriting)return WOULD_BLOCK;
 shutdown();mPacket.reset();
 if(!mOwner->clientChannel)return error(mOwner->clientCloseStatus);
 DWORD e=InitialChannelClose(&mOwner->clientChannel);mOwner->clientCloseStatus=e;
 return error(e);
}

RpcTransport::NdrDirection RpcTransportNdr::ndrDirection() const noexcept { return mOwner->initial?NdrDirection::ClientBound:NdrDirection::ServerAccepted; }
uint64_t RpcTransportNdr::authenticatedSessionGeneration() const noexcept { return mOwner->generation; }
RpcTransportNdr::~RpcTransportNdr(){if(mReading||mWriting)std::terminate();shutdown();mPacket.reset();mOwner.reset();}
status_t RpcTransportNdr::shutdown(){if(mStopped.exchange(true))return mShutdownResult.load();auto owner=mOwner;BOOL event=SetEvent(owner->wake);DWORD e=owner->channel?WlrAcceptedShutdown(owner->channel):InitialChannelShutdown(owner->clientChannel);status_t result=event?error(e):UNKNOWN_ERROR;mShutdownResult.store(result);return result;}
status_t RpcTransportNdr::fail(status_t s){shutdown();return s==OK?UNKNOWN_ERROR:s;}
status_t RpcTransportNdr::validateShape(){if(!mPacket||!mPhaseOpen)return OK;size_t n=mPacket->payloadSize();switch(mPhase){case NdrPhase::ConnectionHeader:if(n!=sizeof(RpcConnectionHeader))return fail(BAD_VALUE);break;case NdrPhase::SessionId:if(n!=32)return fail(BAD_VALUE);break;case NdrPhase::NewSessionResponse:if(n!=sizeof(RpcNewSessionResponse))return fail(BAD_VALUE);break;case NdrPhase::ConnectionInit:if(n!=sizeof(RpcOutgoingConnectionInit))return fail(BAD_VALUE);break;case NdrPhase::Command:if(n<sizeof(RpcWireHeader))return fail(BAD_VALUE);return OK;default:return fail(BAD_VALUE);}return mPacket->handleCount()?fail(BAD_VALUE):OK;}
status_t RpcTransportNdr::loadPacket(){if(mStopped)return DEAD_OBJECT;if(mPacket)return OK;DWORD e= mOwner->initial ? InitialChannelCheckLive(mOwner->clientChannel) : ERROR_SUCCESS;if(e)return fail(error(e));e= mOwner->initial ? WindowsNdrReceivedPacket::tryReceiveInitial(mOwner->clientChannel,&mPacket) : WindowsNdrReceivedPacket::tryPopAccepted(mOwner->channel,&mPacket);if(e==ERROR_NO_MORE_ITEMS)return WOULD_BLOCK;if(e)return fail(error(e));if(mPacket->sequence()!=mNextRead||mPacket->role()!=(mOwner->initial?WindowsNdrReceivedPacket::Role::ClientReply:WindowsNdrReceivedPacket::Role::ServerInbound))return fail(BAD_VALUE);return validateShape();}
status_t RpcTransportNdr::pollRead(){bool expected=false;if(!mReading.compare_exchange_strong(expected,true))return INVALID_OPERATION;Guard guard{mReading};status_t r=loadPacket();if(r!=OK)return r;return mReadBytes<mPacket->payloadSize()?OK:WOULD_BLOCK;}
status_t RpcTransportNdr::beginNdrRecord(NdrPhase phase){bool expected=false;if(!mReading.compare_exchange_strong(expected,true))return INVALID_OPERATION;Guard guard{mReading};if(mStopped)return DEAD_OBJECT;if(mPhaseOpen)return fail(INVALID_OPERATION);switch(phase){case NdrPhase::ConnectionHeader:case NdrPhase::SessionId:case NdrPhase::NewSessionResponse:case NdrPhase::ConnectionInit:case NdrPhase::Command:break;default:return fail(BAD_VALUE);}mPhase=phase;mPhaseOpen=true;mDescriptorsDone=false;mReadBytes=0;std::memset(mHeader,0,sizeof(mHeader));return validateShape();}
status_t RpcTransportNdr::finishNdrRecord(std::shared_ptr<const bscp::windows::WindowsNdrRequestSnapshot>* proof){if(!proof)return BAD_VALUE;proof->reset();bool expected=false;if(!mReading.compare_exchange_strong(expected,true))return INVALID_OPERATION;Guard guard{mReading};if(mStopped)return DEAD_OBJECT;if(!mPhaseOpen||!mPacket)return fail(INVALID_OPERATION);if(mReadBytes!=mPacket->payloadSize()||!mDescriptorsDone)return fail(BAD_VALUE);if(mPhase==NdrPhase::Command){RpcWireHeader h{};std::memcpy(&h,mHeader,sizeof(h));if(h.bodySize!=mPacket->payloadSize()-sizeof(h))return fail(BAD_VALUE);}DWORD e=mPacket->finishValidatedPhase(proof);if(e)return fail(error(e));if(mOwner->initial){if(*proof){proof->reset();return fail(PERMISSION_DENIED);}e=InitialChannelCheckLive(mOwner->clientChannel);if(e)return fail(error(e));}else{if(!*proof)return fail(PERMISSION_DENIED);bool same=false;e=(*proof)->matchesOriginalProcess(mOwner->process,&same);if(e||!same){proof->reset();return fail(error(e?e:ERROR_ACCESS_DENIED));}}mPacket.reset();mPhaseOpen=false;mReadBytes=0;if(mNextRead==std::numeric_limits<uint64_t>::max()){proof->reset();return fail(BAD_VALUE);}++mNextRead;return OK;}
status_t RpcTransportNdr::pause(FdTrigger* trigger,const std::optional<binder::impl::SmallFunction<status_t()>>& alt){Waiting waiting(mWaiting);auto owner=mOwner;if(mStopped)return DEAD_OBJECT;if(trigger){auto state=trigger->cancellation().state();if(state.outcome==bscp::windows::Cancellation::Outcome::cancelled)return -ECANCELED;if(state.outcome==bscp::windows::Cancellation::Outcome::error)return error(state.error);}if(alt)return (*alt)();if(trigger){auto state=trigger->cancellation().wait(owner->wake,5);if(state.outcome==bscp::windows::Cancellation::Outcome::cancelled)return -ECANCELED;if(state.outcome==bscp::windows::Cancellation::Outcome::error)return error(state.error);return mStopped?DEAD_OBJECT:OK;}DWORD r=WaitForSingleObject(owner->wake,5);return r==WAIT_TIMEOUT?OK:r==WAIT_OBJECT_0?DEAD_OBJECT:UNKNOWN_ERROR;}
status_t RpcTransportNdr::interruptableReadFully(FdTrigger* trigger,iovec* iovs,int count,const std::optional<binder::impl::SmallFunction<status_t()>>& alt,std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>* fds){bool expected=false;if(!mReading.compare_exchange_strong(expected,true))return INVALID_OPERATION;Guard guard{mReading};if(mStopped)return DEAD_OBJECT;size_t total=0;if(!iovSize(iovs,count,&total)||!mPhaseOpen)return fail(BAD_VALUE);if(!total)return OK;
 for(;;){if(trigger&&trigger->isTriggered())return fail(-ECANCELED);status_t r=loadPacket();if(r==OK)break;if(r!=WOULD_BLOCK)return r;r=pause(trigger,alt);if(r!=OK)return fail(r);}if(total>mPacket->payloadSize()-mReadBytes)return fail(BAD_VALUE);
 try {if(!mDescriptorsDone){if(!fds&&mPacket->handleCount())return fail(BAD_VALUE);if(fds){const size_t previous=fds->size(),n=mPacket->handleCount();if(previous>bscp::windows::kNdrDescriptorMax||n>bscp::windows::kNdrDescriptorMax-previous)return fail(BAD_VALUE);fds->reserve(previous+n);std::vector<binder::unique_fd> staged;staged.reserve(n);for(size_t i=0;i<n;++i){HANDLE h=nullptr;DWORD e=mPacket->takeDuplicatedHandle(i,&h);if(e)return fail(error(e));int fd=-1;int r=bscp_fd_from_handle(h,&fd);BOOL closed=CloseHandle(h);if(r||!closed){if(fd>=0)bscp_fd_close(fd);return fail(r?r:UNKNOWN_ERROR);}staged.emplace_back(fd);}for(auto& fd:staged)fds->emplace_back(std::move(fd));}else{DWORD e=mPacket->discardRemainingHandles();if(e)return fail(error(e));}mDescriptorsDone=true;}
 for(int i=0;i<count;++i){const size_t n=iovs[i].iov_len;DWORD e=mPacket->readPayloadExact({static_cast<unsigned char*>(iovs[i].iov_base),n});if(e)return fail(error(e));if(mReadBytes<sizeof(mHeader)){size_t save=std::min(n,sizeof(mHeader)-mReadBytes);if(save)std::memcpy(mHeader+mReadBytes,iovs[i].iov_base,save);}mReadBytes+=n;if(n)iovs[i].iov_base=static_cast<char*>(iovs[i].iov_base)+n;iovs[i].iov_len=0;}return OK;}catch(const std::bad_alloc&){return fail(NO_MEMORY);}
}
status_t RpcTransportNdr::interruptableWriteFully(FdTrigger* trigger,iovec* iovs,int count,const std::optional<binder::impl::SmallFunction<status_t()>>& alt,const std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>* fds){bool expected=false;if(!mWriting.compare_exchange_strong(expected,true))return INVALID_OPERATION;Guard guard{mWriting};if(mStopped)return DEAD_OBJECT;size_t total=0;if(!iovSize(iovs,count,&total))return fail(BAD_VALUE);if(!total)return fds&&!fds->empty()?fail(BAD_VALUE):OK;
 try {std::vector<unsigned char> bytes(total);size_t offset=0;for(int i=0;i<count;++i){if(iovs[i].iov_len)std::memcpy(bytes.data()+offset,iovs[i].iov_base,iovs[i].iov_len);offset+=iovs[i].iov_len;}Handles handles;if(fds){if(fds->size()>bscp::windows::kNdrDescriptorMax)return fail(BAD_VALUE);handles.values.reserve(fds->size());for(const auto& fd:*fds){HANDLE h=nullptr;int r=bscp_fd_to_handle(std::visit([](const auto& value){return value.get();},fd),&h);if(r)return fail(r);handles.values.push_back(h);}}
 bscp::windows::WindowsNdrEncodedPacket packet;DWORD e=bscp::windows::encodeWindowsNdrPacket(bytes,handles.values,&packet);if(e)return fail(error(e));WlrView view{};view.sequence=mNextWrite;view.data=packet.bytes.data();view.bytes=static_cast<uint32_t>(packet.bytes.size());view.files=packet.files.data();view.fileCount=static_cast<uint32_t>(packet.files.size());view.pipes=packet.pipes.data();view.pipeCount=static_cast<uint32_t>(packet.pipes.size());
 for(;;){if(mStopped)return DEAD_OBJECT;if(trigger&&trigger->isTriggered())return fail(-ECANCELED);if(mOwner->initial){e=InitialChannelCheckLive(mOwner->clientChannel);if(!e)e=InitialTrySend(mOwner->clientChannel,&view);}else e=WlrAcceptedPush(mOwner->channel,&view);if(!e)break;if(e!=ERROR_BUSY)return fail(error(e));status_t r=pause(trigger,alt);if(r!=OK)return fail(r);}if(!handles.close())return fail(UNKNOWN_ERROR);if(mNextWrite==std::numeric_limits<uint64_t>::max())return fail(BAD_VALUE);++mNextWrite;for(int i=0;i<count;++i){if(iovs[i].iov_len)iovs[i].iov_base=static_cast<char*>(iovs[i].iov_base)+iovs[i].iov_len;iovs[i].iov_len=0;}return OK;}catch(const std::bad_alloc&){return fail(NO_MEMORY);}
}
}



