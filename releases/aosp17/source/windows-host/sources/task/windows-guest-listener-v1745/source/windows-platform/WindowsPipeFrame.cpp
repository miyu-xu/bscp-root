#include "WindowsPipeFrame.h"
#include <algorithm>
#include <cerrno>
namespace android {
namespace {
uint32_t load(const unsigned char* p){return uint32_t(p[0])|(uint32_t(p[1])<<8)|(uint32_t(p[2])<<16)|(uint32_t(p[3])<<24);}
void store(unsigned char* p,uint32_t v){for(unsigned i=0;i<4;++i)p[i]=static_cast<unsigned char>(v>>(8*i));}
bool valid(iovec* iovs,int count){if(count<0||(!iovs&&count))return false;for(int i=0;i<count;++i)if(!iovs[i].iov_base&&iovs[i].iov_len)return false;return true;}
}
RpcTransportWindowsPipeFrame::RpcTransportWindowsPipeFrame(std::unique_ptr<RpcTransportWindowsPipe> bytes):mBytes(std::move(bytes)){if(!mBytes)mPoisoned=true;}
status_t RpcTransportWindowsPipeFrame::pollRead(){return mPoisoned?DEAD_OBJECT:mBytes->pollRead();}
status_t RpcTransportWindowsPipeFrame::interruptableWriteFully(FdTrigger* trigger,iovec* iovs,int count,
 const std::optional<binder::impl::SmallFunction<status_t()>>& alt,
 const std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>* fds){
    if(mPoisoned)return DEAD_OBJECT;
    if(!valid(iovs,count))return BAD_VALUE;
    // Refuse before sending any frame or duplicating a resource.
    if(fds&&!fds->empty())return -EOPNOTSUPP;
    size_t total=0;for(int i=0;i<count;++i){if(iovs[i].iov_len>kMaxFrameBytes-total)return BAD_VALUE;total+=iovs[i].iov_len;}
    if(!total)return OK;
    unsigned char header[8]{};store(header,static_cast<uint32_t>(total));iovec head{header,sizeof(header)};
    status_t r=mBytes->interruptableWriteFully(trigger,&head,1,alt,nullptr);if(r!=OK)return fail(r);
    r=mBytes->interruptableWriteFully(trigger,iovs,count,alt,nullptr);return r==OK?OK:fail(r);
}
status_t RpcTransportWindowsPipeFrame::interruptableReadFully(FdTrigger* trigger,iovec* iovs,int count,
 const std::optional<binder::impl::SmallFunction<status_t()>>& alt,
 std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>*){
    if(mPoisoned)return DEAD_OBJECT;
    if(!valid(iovs,count))return BAD_VALUE;
    for(int i=0;i<count;++i)while(iovs[i].iov_len){
        if(!mRemaining){
            unsigned char header[8]{};iovec head{header,sizeof(header)};
            status_t r=mBytes->interruptableReadFully(trigger,&head,1,alt,nullptr);if(r!=OK)return fail(r);
            const uint32_t length=load(header),handles=load(header+4);
            if(handles)return fail(-EOPNOTSUPP);
            if(!length||length>kMaxFrameBytes)return fail(BAD_VALUE);
            mRemaining=length;
        }
        size_t amount=std::min<size_t>(mRemaining,iovs[i].iov_len);iovec part{iovs[i].iov_base,amount};
        status_t r=mBytes->interruptableReadFully(trigger,&part,1,alt,nullptr);
        size_t consumed=amount-part.iov_len;iovs[i].iov_base=static_cast<char*>(iovs[i].iov_base)+consumed;
        iovs[i].iov_len-=consumed;mRemaining-=static_cast<uint32_t>(consumed);
        if(r!=OK)return fail(r);
    }
    return OK;
}
}
