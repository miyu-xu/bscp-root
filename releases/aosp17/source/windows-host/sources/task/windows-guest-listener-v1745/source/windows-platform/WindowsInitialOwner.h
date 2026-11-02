#pragma once
#include "InitialBootstrap.h"
#include <atomic>
#include <memory>
#include <mutex>
#include <cstdio>
namespace android { class RpcSession; }
namespace bscp::windows {
// Configured root ownership, NOT an authenticated-channel capability. Only the
// facade's successful per-channel InitialConnect grants a transport admission.
class WindowsInitialOwner final {
    InitialAssociation* root_=nullptr;
    InitialChannel* pendingClose_=nullptr;
    std::mutex operation_;
    std::atomic<bool> stopping_{false};
    DWORD failure_=0,closeFailure_=0;
    bool closeTerminal_=false,closed_=false;
    WindowsInitialOwner* retainedNext_=nullptr;
    inline static std::atomic<WindowsInitialOwner*> retainedHead_{nullptr};
    inline static std::atomic<size_t> retainedCount_{0};
    static void dispose(WindowsInitialOwner* p) noexcept {
        if(!p->root_&&!p->pendingClose_&&!p->closeTerminal_){delete p;return;}
        // Allocation-free retention of the exact owner, including pendingClose
        // and mutex/state. No new alias may act on it, and no implicit retry.
        auto* head=retainedHead_.load();
        do {p->retainedNext_=head;}while(!retainedHead_.compare_exchange_weak(head,p));
        ++retainedCount_;
        std::fprintf(stderr,"Initial owner retained UNKNOWN for controlled recovery; total=%zu\n",retainedCount_.load());
    }
    WindowsInitialOwner()=default;
    friend class ::android::RpcSession;
    void commit(InitialAssociation** owned) noexcept {root_=*owned;*owned=nullptr;}
public:
    ~WindowsInitialOwner()=default;
    static size_t retainedCount()noexcept{return retainedCount_.load();}
    WindowsInitialOwner(const WindowsInitialOwner&)=delete;
    WindowsInitialOwner& operator=(const WindowsInitialOwner&)=delete;
    static DWORD prepare(InitialAssociation* borrowed,std::shared_ptr<WindowsInitialOwner>* out){
        if(!out)return ERROR_INVALID_PARAMETER;out->reset();
        DWORD e=InitialCheckConfigured(borrowed);if(e)return e;
        *out=std::shared_ptr<WindowsInitialOwner>(new WindowsInitialOwner,&WindowsInitialOwner::dispose);return 0;
    }
    void requestStop()noexcept{stopping_=true;}
    DWORD connect(InitialChannel** out){
        if(!out)return ERROR_INVALID_PARAMETER;*out=nullptr;
        std::unique_lock<std::mutex> lock(operation_,std::try_to_lock);if(!lock.owns_lock())return ERROR_BUSY;
        if(!root_||stopping_||failure_||closeTerminal_)return ERROR_OPERATION_ABORTED;
        DWORD e=InitialConnect(root_,out);
        if(e){failure_=e;return e;}
        if(stopping_){pendingClose_=*out;*out=nullptr;e=InitialChannelClose(&pendingClose_);failure_=e?e:ERROR_OPERATION_ABORTED;return failure_;}
        return 0;
    }
    // Caller has stopped/joined users and closed channels before final root close.
    // BUSY is a local no-action result. Other failed cleanup is terminal UNKNOWN.
    DWORD close(){
        requestStop();std::unique_lock<std::mutex> lock(operation_,std::try_to_lock);
        if(!lock.owns_lock())return ERROR_BUSY;if(closed_)return 0;if(closeTerminal_)return closeFailure_;
        if(pendingClose_){DWORD e=InitialChannelClose(&pendingClose_);if(e){if(e!=ERROR_BUSY){closeTerminal_=true;closeFailure_=e;}return e;}}
        if(!root_){closed_=true;return 0;}
        DWORD e=InitialRelease(&root_);if(!e){closed_=true;return 0;}
        if(e!=ERROR_BUSY){closeTerminal_=true;closeFailure_=e;}return e;
    }
};
}
