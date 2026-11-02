#include <binder/RpcSession.h>
#include "WindowsInitialOwner.h"
#include "RpcTransportNdr.h"
#include "RpcState.h"
#include "FdTrigger.h"
#include "OS.h"
#include <chrono>
#include <algorithm>
#include <new>
namespace android {
void RpcSession::snapshotWindowsInitialLocked(uint32_t* state,int32_t* status){*state=mWindowsInitialState;*status=mWindowsInitialSetupStatus;}
void RpcSession::requestWindowsInitialStopLocked(){
    mWindowsInitialStop=true;mWindowsClientOwner->requestStop();
    if(mWindowsInitialState<3)mWindowsInitialState=3;
    if(mShutdownTrigger&&!mShutdownTrigger->cancellation().trigger()&&mWindowsInitialSetupStatus==OK)mWindowsInitialSetupStatus=UNKNOWN_ERROR;
    mAvailableConnectionCv.notify_all();mWindowsInitialCv.notify_all();
}
void RpcSession::requestWindowsInitialStop(){RpcMutexLockGuard lock(mMutex);requestWindowsInitialStopLocked();}
void RpcSession::windowsInitialThreadFinished(){RpcMutexLockGuard lock(mMutex);++mWindowsInitialFinished;mWindowsInitialCv.notify_all();}
status_t RpcSession::releaseWindowsInitialStatus(){
    RpcMutexLockGuard lock(mMutex);
    return mWindowsClientOwner&&mWindowsInitialState==5&&!mWindowsInitialSetupActive&&
        mConnections.mThreads.empty()&&mConnections.mIncoming.empty()&&mConnections.mOutgoing.empty()&&
        mWindowsClosingConnections.empty()&&mWindowsRetiredConnections.empty()?OK:INVALID_OPERATION;
}
status_t RpcSession::addWindowsInitialIncoming(std::unique_ptr<RpcTransport> transport){
    struct Startup {RpcMutex mutex;RpcConditionVariable cv;bool observed=false;status_t status=UNKNOWN_ERROR;};
    auto startup=std::make_shared<Startup>();
    RpcMutexUniqueLock lock(mMutex);
    if(mWindowsInitialStop)return DEAD_OBJECT;
    decltype(mConnections.mThreads) staging;
    auto slot=staging.try_emplace(RpcMaybeThread::id{});auto node=staging.extract(slot.first);
    // No thread exists until its registry node and startup state are allocated.
    RpcMaybeThread thread([session=sp<RpcSession>::fromExisting(this),startup,
                           transport=std::move(transport)]() mutable {
        sp<RpcConnection> connection;status_t status=OK;
        try {
            connection=session->assignIncomingConnectionToThisThread(std::move(transport));
            status=connection?session->state()->readConnectionInit(*connection,*session):DEAD_OBJECT;
        }catch(const std::bad_alloc&){status=NO_MEMORY;}catch(...){status=UNKNOWN_ERROR;}
        if(status!=OK)session->requestWindowsInitialStop();
        {RpcMutexLockGuard observed(startup->mutex);startup->status=status;startup->observed=true;startup->cv.notify_all();}
        session->runWindowsInitialIncoming(std::move(connection),status);
    });
    node.key()=thread.get_id();node.mapped()=std::move(thread);
    auto inserted=mConnections.mThreads.insert(std::move(node));
    LOG_ALWAYS_FATAL_IF(!inserted.inserted,"Initial live thread ID duplicated");
    lock.unlock();
    RpcMutexUniqueLock observed(startup->mutex);startup->cv.wait(observed,[&]{return startup->observed;});
    return startup->status;
}
void RpcSession::runWindowsInitialIncoming(sp<RpcConnection> connection,status_t initialStatus) noexcept {
    status_t failure=initialStatus;bool removed=true;
    try {
        if(initialStatus==OK){
            for(;;){status_t e=state()->getAndExecuteCommand(*connection,*this,RpcState::CommandType::ANY);if(e!=OK){failure=e;break;}}
        }
    }catch(const std::bad_alloc&){failure=NO_MEMORY;}catch(...){failure=UNKNOWN_ERROR;}
    requestWindowsInitialStop();
    try {
        if(connection&&!removeIncomingConnection(connection)){failure=UNKNOWN_ERROR;removed=false;}
    }catch(const std::bad_alloc&){failure=NO_MEMORY;removed=false;}catch(...){failure=UNKNOWN_ERROR;removed=false;}
    {
        RpcMutexLockGuard lock(mMutex);
        if(failure!=OK&&mWindowsInitialSetupStatus==OK)mWindowsInitialSetupStatus=failure;
        if(!removed){
            // Do not claim finished when connection removal failed. Original map,
            // incoming/retired owners remain available for controlled recovery.
            mWindowsInitialCv.notify_all();return;
        }
    }
    windowsInitialThreadFinished();
}
status_t RpcSession::shutdownJoinWindowsInitial(uint32_t ms,uint32_t* state,int32_t* setupStatus){
    if(!state||!setupStatus||ms==UINT32_MAX)return BAD_VALUE;
    std::unique_lock<std::mutex> single(mWindowsInitialJoinGate,std::try_to_lock);
    RpcMutexUniqueLock lock(mMutex);
    if(!mWindowsClientOwner){snapshotWindowsInitialLocked(state,setupStatus);return INVALID_OPERATION;}
    requestWindowsInitialStopLocked();
    // An incoming callback cannot join its own thread. Cancellation is requested,
    // but all ownership remains for the external supervisor to observe/join.
    const auto callingThread=rpc_this_thread::get_id();
    const bool ownsOutgoing=std::any_of(mConnections.mOutgoing.begin(),mConnections.mOutgoing.end(),[&](const auto& c){return c->exclusiveTid.has_value()&&*c->exclusiveTid==binder::os::GetThreadId();});
    if(mConnections.mThreads.count(callingThread)||ownsOutgoing||!single.owns_lock()){
        snapshotWindowsInitialLocked(state,setupStatus);return WOULD_BLOCK;
    }
    if(mWindowsInitialState==5){snapshotWindowsInitialLocked(state,setupStatus);return OK;}
    auto pending=[&]{
        if(mWindowsInitialSetupActive||mWindowsInitialFinished<mConnections.mThreads.size()||mConnections.mWaitingThreads)return true;
        for(const auto& c:mConnections.mOutgoing)if(c->exclusiveTid.has_value())return true;
        return false;
    };
    const auto deadline=std::chrono::steady_clock::now()+std::chrono::milliseconds(ms);
    while(pending()){
        const auto now=std::chrono::steady_clock::now();
        if(now>=deadline){snapshotWindowsInitialLocked(state,setupStatus);return TIMED_OUT;}
        mWindowsInitialCv.wait_for(lock,std::min(std::chrono::milliseconds(5),std::chrono::duration_cast<std::chrono::milliseconds>(deadline-now)));
    }
    if(mWindowsInitialState<4){
        mWindowsClosingConnections.reserve(mWindowsRetiredConnections.size()+mConnections.mOutgoing.size());
        lock.unlock();
        try {for(auto& [id,thread]:mConnections.mThreads){(void)id;if(thread.joinable())thread.join();}}
        catch(...){lock.lock();snapshotWindowsInitialLocked(state,setupStatus);return UNKNOWN_ERROR;}
        lock.lock();
        mConnections.mThreads.clear();
        for(auto& c:mWindowsRetiredConnections)mWindowsClosingConnections.push_back(std::move(c));
        for(auto& c:mConnections.mOutgoing)mWindowsClosingConnections.push_back(std::move(c));
        mWindowsRetiredConnections.clear();mConnections.mOutgoing.clear();
        if(!mConnections.mIncoming.empty()){snapshotWindowsInitialLocked(state,setupStatus);return UNKNOWN_ERROR;}
        mWindowsInitialState=4;
    }
    lock.unlock();
    for(auto& c:mWindowsClosingConnections){
        if(!c||!c->rpcTransport||c->rpcTransport->ndrDirection()!=RpcTransport::NdrDirection::ClientBound){lock.lock();snapshotWindowsInitialLocked(state,setupStatus);return UNKNOWN_ERROR;}
        status_t e=static_cast<RpcTransportNdr*>(c->rpcTransport.get())->closeInitialChannel();
        if(e!=OK){lock.lock();snapshotWindowsInitialLocked(state,setupStatus);return e;}
    }
    // Every original typed channel has a checked known-close result. Clear only
    // after that, and then ask the facade to release its root with no live refs.
    mWindowsClosingConnections.clear();
    DWORD close=mWindowsClientOwner->close();
    if(close){lock.lock();snapshotWindowsInitialLocked(state,setupStatus);return close==ERROR_BUSY?WOULD_BLOCK:UNKNOWN_ERROR;}
    // Dispatch of death notifications is attempted once. An exception may have
    // already notified a subset; never replay it or report the owner released.
    lock.lock();
    if(mWindowsInitialFinalizeAttempted){snapshotWindowsInitialLocked(state,setupStatus);return mWindowsInitialFinalizeStatus;}
    mWindowsInitialFinalizeAttempted=true;lock.unlock();
    try {(void)this->state()->sendObituaries(*this);mRpcBinderState->clear();}
    catch(...){lock.lock();if(mWindowsInitialSetupStatus==OK)mWindowsInitialSetupStatus=UNKNOWN_ERROR;snapshotWindowsInitialLocked(state,setupStatus);return UNKNOWN_ERROR;}
    lock.lock();mWindowsInitialFinalizeStatus=OK;mWindowsInitialState=5;snapshotWindowsInitialLocked(state,setupStatus);mWindowsInitialCv.notify_all();return OK;
}
}
