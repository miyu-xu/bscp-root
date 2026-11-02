#include "WindowsNdrReceivedPacket.h"
#include <atomic>
#include <cstring>
#include <new>
namespace bscp::windows {
namespace {
std::atomic<bool> failControl{false};
template<class T> struct PacketAllocator {using value_type=T;PacketAllocator()=default;template<class U> PacketAllocator(const PacketAllocator<U>&){}T* allocate(size_t n){if(failControl.exchange(false))throw std::bad_alloc();return std::allocator<T>{}.allocate(n);}void deallocate(T* p,size_t n){std::allocator<T>{}.deallocate(p,n);}template<class U>bool operator==(const PacketAllocator<U>&)const{return true;}};
template<class T> std::shared_ptr<T> share(std::unique_ptr<T>& p){T* raw=p.release();return std::shared_ptr<T>(raw,std::default_delete<T>{},PacketAllocator<T>{});}
}
#ifdef BSCP_WLR_ISOLATED_COMPONENT
void windowsNdrPacketTestFailControlBlock(){failControl=true;}
#endif
DWORD WindowsNdrReceivedPacket::decodeOwned(const WlrView& original){WlrView view=original;view.data=encoded_.data();view.bytes=static_cast<uint32_t>(encoded_.size());WindowsNdrDecodedPacket decoded;DWORD e=decodeWindowsNdrPacket(view,&decoded);if(e)return e;payloadOffset_=static_cast<size_t>(decoded.binderBytes.data()-encoded_.data());payloadSize_=decoded.binderBytes.size();handles_=std::move(decoded.orderedHandles);disposed_.resize(handles_.size());return 0;}
DWORD WindowsNdrReceivedPacket::tryPopAccepted(WlrAcceptedChannel* a,std::shared_ptr<WindowsNdrReceivedPacket>* out){if(!out)return ERROR_INVALID_PARAMETER;out->reset();std::shared_ptr<WindowsNdrReceivedRecord> record;DWORD e=WindowsNdrReceivedRecord::tryPopAccepted(a,&record);if(e)return e;
 try {std::unique_ptr<WindowsNdrReceivedPacket> p(new WindowsNdrReceivedPacket(Role::ServerInbound));p->inbound_=std::move(record);p->sequence_=p->inbound_->sequence();if(p->inbound_->byteCount()>kNdrEnvelopeMax)return ERROR_INVALID_DATA;p->encoded_.resize(p->inbound_->byteCount());e=p->inbound_->readExact(p->encoded_);if(e)return e;WlrView v{};v.files=p->inbound_->files().data();v.fileCount=static_cast<uint32_t>(p->inbound_->files().size());v.pipes=p->inbound_->pipes().data();v.pipeCount=static_cast<uint32_t>(p->inbound_->pipes().size());e=p->decodeOwned(v);if(e)return e;*out=share(p);return 0;}catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
}
DWORD WindowsNdrReceivedPacket::tryReceiveInitial(InitialChannel* c,std::shared_ptr<WindowsNdrReceivedPacket>* out){if(!out)return ERROR_INVALID_PARAMETER;out->reset();WlrRecord* raw=nullptr;DWORD e=InitialTryReceive(c,&raw);if(e)return e;std::unique_ptr<WlrRecord,Release> record(raw);
 try {std::unique_ptr<WindowsNdrReceivedPacket> p(new WindowsNdrReceivedPacket(Role::ClientReply));p->reply_=std::move(record);WlrView v{};e=WlrRecordView(p->reply_.get(),&v);if(e)return e;p->sequence_=v.sequence;if(v.bytes>kNdrEnvelopeMax||(!v.data&&v.bytes))return ERROR_INVALID_DATA;if(v.bytes)p->encoded_.assign(v.data,v.data+v.bytes);e=p->decodeOwned(v);if(e)return e;*out=share(p);return 0;}catch(const std::bad_alloc&){return ERROR_NOT_ENOUGH_MEMORY;}
}
DWORD WindowsNdrReceivedPacket::readPayloadExact(std::span<unsigned char> dest){if(failed_||finished_)return ERROR_OPERATION_ABORTED;if(dest.size()>payloadSize_-consumed_){failed_=true;return ERROR_INVALID_DATA;}if(!dest.empty())std::memcpy(dest.data(),encoded_.data()+payloadOffset_+consumed_,dest.size());consumed_+=dest.size();return 0;}
DWORD WindowsNdrReceivedPacket::takeDuplicatedHandle(size_t index,HANDLE* out){if(!out)return ERROR_INVALID_PARAMETER;*out=nullptr;if(failed_||finished_)return ERROR_OPERATION_ABORTED;if(index>=handles_.size()||disposed_[index]){failed_=true;return ERROR_INVALID_DATA;}if(!DuplicateHandle(GetCurrentProcess(),handles_[index],GetCurrentProcess(),out,0,FALSE,DUPLICATE_SAME_ACCESS)){failed_=true;return GetLastError();}disposed_[index]=1;return 0;}
DWORD WindowsNdrReceivedPacket::discardRemainingHandles(){if(failed_||finished_)return ERROR_OPERATION_ABORTED;for(auto& d:disposed_)if(!d)d=2;return 0;}
DWORD WindowsNdrReceivedPacket::finishValidatedPhase(std::shared_ptr<const WindowsNdrRequestSnapshot>* proof){if(!proof)return ERROR_INVALID_PARAMETER;proof->reset();if(failed_||finished_)return ERROR_OPERATION_ABORTED;if(consumed_!=payloadSize_){failed_=true;return ERROR_INVALID_DATA;}for(auto d:disposed_)if(!d){failed_=true;return ERROR_INVALID_DATA;}
 if(role_==Role::ServerInbound){DWORD e=inbound_->sealConsumedRecord();if(!e)e=inbound_->makeSnapshot(proof);if(e){failed_=true;return e;}}
 LONG s,c,r,before,after;WlrCounters(&s,&c,&r,&before);finished_=true;inbound_.reset();reply_.reset();for(auto& h:handles_)h=nullptr;WlrCounters(&s,&c,&r,&after);if(after!=before){failed_=true;proof->reset();return ERROR_GEN_FAILURE;}return 0;
}
}

