#include "WindowsGuestListener.h"
#include "WindowsPipeFrame.h"
#include "FdTrigger.h"
#include <new>
#include <log/log.h>
namespace android {
namespace {
status_t failure(DWORD error){
    switch(error){
        case ERROR_ACCESS_DENIED:return PERMISSION_DENIED;
        case ERROR_OPERATION_ABORTED:case ERROR_BROKEN_PIPE:case ERROR_PIPE_NOT_CONNECTED:return DEAD_OBJECT;
        case ERROR_INVALID_HANDLE:case ERROR_INVALID_PARAMETER:return BAD_VALUE;
        case ERROR_NOT_ENOUGH_MEMORY:case ERROR_OUTOFMEMORY:return NO_MEMORY;
        default:ALOGE("Windows Guest listener failed, Win32=%lu",static_cast<unsigned long>(error));return UNKNOWN_ERROR;
    }
}
struct Event {HANDLE value=CreateEventW(nullptr,TRUE,FALSE,nullptr);~Event(){if(value)CloseHandle(value);}};
}
// Every real frame I/O retains the original process object and validates the
// actual pipe client. This is Guest bridge identity, never desktop caller state.
class BoundGuestTransport final:public RpcTransport {
public:
    BoundGuestTransport(std::unique_ptr<RpcTransportWindowsPipeFrame> bytes,
        std::shared_ptr<const WindowsGuestProof> proof):mBytes(std::move(bytes)),mProof(std::move(proof)){}
    status_t check() const {return mProof->check();}
    std::shared_ptr<const WindowsGuestProof> guestProof() const override{return mProof;}
    status_t pollRead() override {auto s=check();return s==OK?mBytes->pollRead():s;}
    bool isWaiting() override{return mBytes->isWaiting();}
    status_t interruptableWriteFully(FdTrigger* t,iovec* v,int n,
        const std::optional<binder::impl::SmallFunction<status_t()>>& p,
        const std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>* f) override {
        auto s=check();if(s!=OK)return s;s=mBytes->interruptableWriteFully(t,v,n,p,f);return s==OK?check():s;
    }
    status_t interruptableReadFully(FdTrigger* t,iovec* v,int n,
        const std::optional<binder::impl::SmallFunction<status_t()>>& p,
        std::vector<std::variant<binder::unique_fd,binder::borrowed_fd>>* f) override {
        auto s=check();if(s!=OK)return s;s=mBytes->interruptableReadFully(t,v,n,p,f);return s==OK?check():s;
    }
private:
    std::unique_ptr<RpcTransportWindowsPipeFrame> mBytes;
    std::shared_ptr<const WindowsGuestProof> mProof;
};
status_t WindowsGuestProof::capture(HANDLE pipe,const std::shared_ptr<const bscp::windows::BoundPipePeer>& peer,
    const bscp::windows::CallerIdentity& allowed,std::shared_ptr<const WindowsGuestProof>* out){
    if(!out||!peer)return BAD_VALUE;out->reset();
    auto p=std::shared_ptr<WindowsGuestProof>(new WindowsGuestProof);
    p->mPeer=peer;p->mAllowed=allowed;
    if(!DuplicateHandle(GetCurrentProcess(),pipe,GetCurrentProcess(),&p->mPipe,0,FALSE,DUPLICATE_SAME_ACCESS))return failure(GetLastError());
    status_t s=p->check();if(s!=OK)return s;*out=std::move(p);return OK;
}
WindowsGuestProof::~WindowsGuestProof(){if(mPipe)bscp::windows::releaseOrRetainPipe(mPipe);}
status_t WindowsGuestProof::check() const{return mPeer->validate(mPipe,mAllowed)?PERMISSION_DENIED:OK;}
status_t WindowsGuestListener::create(uint32_t cid,uint32_t port,std::unique_ptr<WindowsGuestListener>* out){
    if(!out||cid<3||cid==UINT32_MAX||!port||port==UINT32_MAX)return BAD_VALUE;
    out->reset();
    auto owner=std::unique_ptr<WindowsGuestListener>(new(std::nothrow) WindowsGuestListener(
        L"\\\\.\\pipe\\binder_rpc_vsock_"+std::to_wstring(cid)+L"_"+std::to_wstring(port)));
    if(!owner)return NO_MEMORY;
    DWORD error=bscp::windows::CallerIdentity::currentProcess(&owner->mAllowed);
    if(error)return failure(error);
    status_t s=owner->createInstance(true,&owner->mPending);if(s!=OK)return s;
    *out=std::move(owner);return OK;
}
status_t WindowsGuestListener::createInstance(bool first,HANDLE* out){
    *out=INVALID_HANDLE_VALUE;
    HANDLE pipe=CreateNamedPipeW(mName.c_str(),PIPE_ACCESS_DUPLEX|FILE_FLAG_OVERLAPPED|
        (first?FILE_FLAG_FIRST_PIPE_INSTANCE:0),PIPE_TYPE_BYTE|PIPE_READMODE_BYTE|PIPE_WAIT|PIPE_REJECT_REMOTE_CLIENTS,
        PIPE_UNLIMITED_INSTANCES,4096,4096,0,nullptr);
    if(pipe==INVALID_HANDLE_VALUE)return failure(GetLastError());
    DWORD flags=0;if(!GetHandleInformation(pipe,&flags)||flags&HANDLE_FLAG_INHERIT){
        DWORD e=GetLastError();bscp::windows::releaseOrRetainPipe(pipe);return failure(e?e:ERROR_INVALID_DATA);
    }
    *out=pipe;return OK;
}
WindowsGuestListener::~WindowsGuestListener(){
    // accept() drains its exact pending I/O before returning. Caller joins the
    // server loop before destroying this listener; no close is used to cancel.
    if(mPending!=INVALID_HANDLE_VALUE)bscp::windows::releaseOrRetainPipe(mPending);
    if(mOriginal&&!CloseHandle(mOriginal))ALOGE("Closing Guest peer process handle failed: %lu",static_cast<unsigned long>(GetLastError()));
}
status_t WindowsGuestListener::bindOriginalProcess(HANDLE original){
    if(mAcceptStarted||mOriginal||!original||original==INVALID_HANDLE_VALUE)return INVALID_OPERATION;
    HANDLE copy=nullptr;
    if(!DuplicateHandle(GetCurrentProcess(),original,GetCurrentProcess(),&copy,
        SYNCHRONIZE|PROCESS_QUERY_LIMITED_INFORMATION,FALSE,0))return failure(GetLastError());
    DWORD state=WaitForSingleObject(copy,0);DWORD pid=GetProcessId(copy);
    if(state!=WAIT_TIMEOUT||!pid){DWORD e=GetLastError();CloseHandle(copy);return failure(e?e:ERROR_PROCESS_ABORTED);}
    mOriginal=copy;return OK;
}
status_t WindowsGuestListener::accept(FdTrigger* trigger,std::unique_ptr<RpcTransport>* out){
    if(!out||!trigger||!mOriginal||mPending==INVALID_HANDLE_VALUE)return INVALID_OPERATION;
    out->reset();mAcceptStarted=true;
    auto cancel=trigger->cancellation();if(!cancel.valid())return UNKNOWN_ERROR;
    Event event;if(!event.value)return failure(GetLastError());
    OVERLAPPED op{};op.hEvent=event.value;DWORD bytes=0;
    BOOL connected=ConnectNamedPipe(mPending,&op);DWORD e=connected?ERROR_SUCCESS:GetLastError();
    if(!connected&&e==ERROR_IO_PENDING){
        const auto wait=cancel.wait(event.value,INFINITE);
        bool cancelled=wait.outcome!=bscp::windows::Cancellation::Outcome::ready;
        DWORD cancelError=ERROR_SUCCESS;
        if(cancelled&&!CancelIoEx(mPending,&op)&&GetLastError()!=ERROR_NOT_FOUND)cancelError=GetLastError();
        connected=GetOverlappedResult(mPending,&op,&bytes,TRUE);e=connected?ERROR_SUCCESS:GetLastError();
        if(cancelError)return failure(cancelError);
        if(cancelled)return wait.outcome==bscp::windows::Cancellation::Outcome::cancelled?DEAD_OBJECT:UNKNOWN_ERROR;
    }
    if(!connected&&e!=ERROR_PIPE_CONNECTED)return failure(e);
    std::shared_ptr<const bscp::windows::BoundPipePeer> peer;
    DWORD authentication=bscp::windows::BoundPipePeer::bind(mPending,mOriginal,mAllowed,&peer);
    HANDLE next=INVALID_HANDLE_VALUE;status_t s=createInstance(false,&next);
    if(s!=OK)return s;
    HANDLE accepted=mPending;mPending=next;
    if(authentication){bscp::windows::releaseOrRetainPipe(accepted);return PERMISSION_DENIED;}
    std::shared_ptr<const WindowsGuestProof> proof;
    s=WindowsGuestProof::capture(accepted,peer,mAllowed,&proof);
    if(s!=OK){bscp::windows::releaseOrRetainPipe(accepted);return s;}
    auto bytesTransport=RpcTransportWindowsPipe::adoptOverlapped(accepted);
    if(!bytesTransport)return BAD_TYPE;
    auto frame=std::make_unique<RpcTransportWindowsPipeFrame>(std::move(bytesTransport));
    *out=std::make_unique<BoundGuestTransport>(std::move(frame),std::move(proof));
    return OK;
}
}
