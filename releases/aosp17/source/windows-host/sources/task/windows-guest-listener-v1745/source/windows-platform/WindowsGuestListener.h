#pragma once
#include <windows.h>
#include <memory>
#include <string>
#include <binder/RpcTransport.h>
#include "WindowsCallerIdentity.h"
namespace android {
class FdTrigger;
class WindowsGuestProof final {
public:
    static status_t capture(HANDLE,const std::shared_ptr<const bscp::windows::BoundPipePeer>&,
        const bscp::windows::CallerIdentity&,std::shared_ptr<const WindowsGuestProof>*);
    ~WindowsGuestProof();
    status_t check() const;
private:
    WindowsGuestProof()=default;
    HANDLE mPipe=nullptr;
    std::shared_ptr<const bscp::windows::BoundPipePeer> mPeer;
    bscp::windows::CallerIdentity mAllowed;
};
// Host-local r14 endpoint and frame contract. No socket surrogate, CRT adoption,
// credential, custom security descriptor or descriptor-bearing Guest protocol.
class WindowsGuestListener final {
public:
    static status_t create(uint32_t cid,uint32_t port,std::unique_ptr<WindowsGuestListener>* out);
    ~WindowsGuestListener();
    WindowsGuestListener(const WindowsGuestListener&)=delete;
    WindowsGuestListener& operator=(const WindowsGuestListener&)=delete;
    // Before any accept/dispatch. Duplicate only the original spawn object;
    // the caller retains its original HANDLE and lifecycle ownership.
    status_t bindOriginalProcess(HANDLE original);
    bool bound() const {return mOriginal!=nullptr;}
    status_t accept(FdTrigger*,std::unique_ptr<RpcTransport>* out);
private:
    explicit WindowsGuestListener(std::wstring name):mName(std::move(name)){}
    status_t createInstance(bool first,HANDLE* out);
    std::wstring mName;
    HANDLE mPending=INVALID_HANDLE_VALUE;
    HANDLE mOriginal=nullptr;
    bscp::windows::CallerIdentity mAllowed;
    bool mAcceptStarted=false;
};
}
