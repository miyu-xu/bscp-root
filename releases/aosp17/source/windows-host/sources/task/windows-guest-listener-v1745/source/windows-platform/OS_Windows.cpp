#include "WindowsOverlappedPipeFd.h"
#include "WindowsSocketApi.h"
#include "OS.h"
#include <binder/RpcTransportRaw.h>
#include <bcrypt.h>
#include <limits>
namespace android::binder::os {
// Same no-trace/no-sysprop desktop capability contract as official non-Android Linux.
void trace_begin(uint64_t,const char*){}
void trace_end(uint64_t){}
void trace_int(uint64_t,const char*,int32_t){}
uint64_t get_trace_enabled_tags(){return 0;}
uint64_t GetThreadId(){return GetCurrentThreadId();}
bool report_sysprop_change(){return false;}
status_t getRandomBytes(uint8_t* data,size_t size){
    if((!data&&size)||size>std::numeric_limits<ULONG>::max())return BAD_VALUE;
    return BCryptGenRandom(nullptr,data,static_cast<ULONG>(size),BCRYPT_USE_SYSTEM_PREFERRED_RNG)>=0?OK:UNKNOWN_ERROR;
}
status_t setNonBlocking(borrowed_fd fd){
    if(auto pipe=bscp::windows::acquireOverlappedPipe(fd.get()))return bscp::windows::validateRawOverlappedPipe(pipe)==ERROR_SUCCESS?OK:BAD_TYPE;
    auto socket=bscp::windows::acquireSocket(fd.get());
    if(!socket)return BAD_TYPE;
    u_long mode=1;return ioctlsocket(socket->socket,FIONBIO,&mode)==0?OK:UNKNOWN_ERROR;
}
std::unique_ptr<RpcTransportCtxFactory> makeDefaultRpcTransportCtxFactory(){return RpcTransportCtxFactoryRaw::make();}
ssize_t sendMessageOnSocket(const RpcTransportFd& socket,iovec* iovs,int count,
 const std::vector<std::variant<unique_fd,borrowed_fd>>* fds){
    // Raw Winsock streams do not provide SCM_RIGHTS. Descriptor-capable named-pipe transport
    // is a separate platform implementation and must not silently fall back to this path.
    if(fds&&!fds->empty()){errno=EOPNOTSUPP;return -1;}
    if(count<0||(!iovs&&count)){errno=EINVAL;return -1;}
    for(int i=0;i<count;++i){if(iovs[i].iov_len)return bscp::windows::send(socket.fd.get(),iovs[i].iov_base,iovs[i].iov_len,0);}
    return 0;
}
ssize_t receiveMessageFromSocket(const RpcTransportFd& socket,iovec* iovs,int count,
 std::vector<std::variant<unique_fd,borrowed_fd>>*){
    if(count<0||(!iovs&&count)){errno=EINVAL;return -1;}
    for(int i=0;i<count;++i){if(iovs[i].iov_len)return bscp::windows::recv(socket.fd.get(),iovs[i].iov_base,iovs[i].iov_len,0);}
    return 0;
}
}
