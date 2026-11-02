#include <binder/RpcServer.h>
#include "FdTrigger.h"
#include "WindowsLocalRpcBridge.h"
#include <chrono>
#include <new>
namespace android {
#ifdef BSCP_NDR_LIFECYCLE_TESTING
status_t RpcServer::setWindowsNdrTestHook(std::function<void()> hook) {
    RpcMutexLockGuard lock(mLock);
    if(mWindowsNdrState!=WindowsNdrState::New || mWindowsManaged)return INVALID_OPERATION;
    mWindowsNdrTestHook=std::move(hook);return OK;
}
#endif
void RpcServer::latchWindowsNdrErrorLocked(status_t value) {
    if (value != OK && mWindowsNdrLoopStatus == OK) mWindowsNdrLoopStatus = value;
}
void RpcServer::windowsNdrSnapshotLocked(uint32_t* state, int32_t* error) {
    *state = static_cast<uint32_t>(mWindowsNdrState); *error = mWindowsNdrLoopStatus;
}
bool RpcServer::windowsNdrManaged() { RpcMutexLockGuard lock(mLock); return mWindowsManaged; }
status_t RpcServer::windowsNdrReleaseStatus() {
    RpcMutexLockGuard lock(mLock);
    return mWindowsDesktop && mWindowsManaged && mWindowsNdrState == WindowsNdrState::Joined &&
        !mJoinThread && !mJoinThreadRunning && !mShutdownTrigger &&
        mConnectingThreads.empty() && mSessions.empty() ? OK : INVALID_OPERATION;
}
status_t RpcServer::startWindowsNdr() {
    std::unique_lock<std::mutex> single(mWindowsNdrJoinGate, std::try_to_lock);
    if (!single.owns_lock()) return WOULD_BLOCK;
    RpcMutexLockGuard lock(mLock);
    if (!mWindowsDesktop || mWindowsNdrState != WindowsNdrState::New || mJoinThread ||
        mJoinThreadRunning || mShutdownTrigger) return INVALID_OPERATION;
    mWindowsManaged = true; mWindowsNdrState = WindowsNdrState::Starting;
    try {
        mJoinThread = std::make_unique<RpcMaybeThread>(
            &RpcServer::joinWindowsNdrManaged, sp<RpcServer>::fromExisting(this));
    } catch (const std::bad_alloc&) {
        latchWindowsNdrErrorLocked(NO_MEMORY); mWindowsNdrState = WindowsNdrState::LoopExited;
        mShutdownCv.notify_all(); return NO_MEMORY;
    } catch (...) {
        latchWindowsNdrErrorLocked(UNKNOWN_ERROR); mWindowsNdrState = WindowsNdrState::LoopExited;
        mShutdownCv.notify_all(); return UNKNOWN_ERROR;
    }
    mShutdownCv.notify_all(); return OK;
}
void RpcServer::requestWindowsNdrStopLocked() {
    mWindowsStopRequested = true;
    if (mWindowsNdrState < WindowsNdrState::LoopExited) mWindowsNdrState = WindowsNdrState::Stopping;
    // Use the checked Windows event operation, not FdTrigger's fatal wrapper.
    if (mShutdownTrigger && !mShutdownTrigger->cancellation().trigger())
        latchWindowsNdrErrorLocked(UNKNOWN_ERROR);
    for (auto& [id, session] : mSessions) {
        (void)id;
        RpcMutexLockGuard sessionLock(session->mMutex);
        if (session->mShutdownTrigger && !session->mShutdownTrigger->cancellation().trigger())
            latchWindowsNdrErrorLocked(UNKNOWN_ERROR);
    }
    mShutdownCv.notify_all();
}
void RpcServer::joinWindowsNdrManaged() noexcept {
    status_t result = OK;
    try {
#ifdef BSCP_NDR_LIFECYCLE_TESTING
        if(mWindowsNdrTestHook)mWindowsNdrTestHook();
#endif
        {
            RpcMutexLockGuard lock(mLock);
            mJoinThreadRunning = true;
            mShutdownTrigger = FdTrigger::make();
            if (!mShutdownTrigger) result = NO_MEMORY;
            else if (WlrAcceptorCheckLive(mWindowsAcceptor.get())) result = DEAD_OBJECT;
            if (result == OK && !mWindowsStopRequested) mWindowsNdrState = WindowsNdrState::Ready;
            else requestWindowsNdrStopLocked();
            mShutdownCv.notify_all();
        }
        while (result == OK) {
            {
                RpcMutexLockGuard lock(mLock);
                if (mWindowsStopRequested) break;
                if (mShutdownTrigger->isTriggered()) { result = DEAD_OBJECT; break; }
            }
            result = acceptWindowsNdrConnection();
            if (result == WOULD_BLOCK) { result = OK; Sleep(5); continue; }
            if (result != OK) break; // Preserve real accept errors even if stop races.
        }
    } catch (const std::bad_alloc&) { result = NO_MEMORY; }
      catch (...) { result = UNKNOWN_ERROR; }
    {
        RpcMutexLockGuard lock(mLock);
        latchWindowsNdrErrorLocked(result);
        requestWindowsNdrStopLocked();
        mJoinThreadRunning = false;
        mWindowsNdrState = WindowsNdrState::LoopExited;
        mShutdownCv.notify_all();
    }
}
status_t RpcServer::waitWindowsNdrReady(uint32_t ms, uint32_t* state, int32_t* error) {
    if (!state || !error || ms == UINT32_MAX) return BAD_VALUE;
    RpcMutexUniqueLock lock(mLock);
    if (!mWindowsDesktop) { windowsNdrSnapshotLocked(state,error); return INVALID_OPERATION; }
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(ms);
    while (mWindowsNdrState == WindowsNdrState::New || mWindowsNdrState == WindowsNdrState::Starting) {
        if (mShutdownCv.wait_until(lock, deadline) == RpcCvStatus::timeout &&
            (mWindowsNdrState == WindowsNdrState::New || mWindowsNdrState == WindowsNdrState::Starting)) {
            windowsNdrSnapshotLocked(state,error); return TIMED_OUT;
        }
    }
    windowsNdrSnapshotLocked(state,error);
    if (mWindowsNdrState == WindowsNdrState::Ready) return OK;
    return mWindowsNdrLoopStatus == OK ? DEAD_OBJECT : mWindowsNdrLoopStatus;
}
status_t RpcServer::shutdownJoinWindowsNdr(uint32_t ms, uint32_t* state, int32_t* error) {
    if (!state || !error || ms == UINT32_MAX) return BAD_VALUE;
    std::unique_lock<std::mutex> single(mWindowsNdrJoinGate, std::try_to_lock);
    if (!single.owns_lock()) {
        RpcMutexLockGuard lock(mLock); windowsNdrSnapshotLocked(state,error); return WOULD_BLOCK;
    }
    RpcMutexUniqueLock lock(mLock);
    if (!mWindowsDesktop || (!mWindowsManaged && (mJoinThread || mJoinThreadRunning || mShutdownTrigger))) {
        windowsNdrSnapshotLocked(state,error); return INVALID_OPERATION;
    }
    mWindowsManaged = true;
    if (mWindowsNdrState == WindowsNdrState::Joined) { windowsNdrSnapshotLocked(state,error); return OK; }
    if (mWindowsNdrState == WindowsNdrState::New) {
        mWindowsStopRequested = true; mWindowsNdrState = WindowsNdrState::Joined;
        windowsNdrSnapshotLocked(state,error); mShutdownCv.notify_all(); return OK;
    }
    requestWindowsNdrStopLocked();
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(ms);
    while (mWindowsNdrState != WindowsNdrState::LoopExited || mJoinThreadRunning ||
           !mConnectingThreads.empty() || !mSessions.empty()) {
        if (mShutdownCv.wait_until(lock, deadline) == RpcCvStatus::timeout &&
            (mWindowsNdrState != WindowsNdrState::LoopExited || mJoinThreadRunning ||
             !mConnectingThreads.empty() || !mSessions.empty())) {
            windowsNdrSnapshotLocked(state,error); return TIMED_OUT;
        }
    }
    // Single-join gate remains held, but never the state lock. A failed join
    // retains the exact joinable object; no local joinable destructor runs.
    lock.unlock();
    try { if (mJoinThread) mJoinThread->join(); }
    catch (...) {
        lock.lock(); latchWindowsNdrErrorLocked(UNKNOWN_ERROR);
        windowsNdrSnapshotLocked(state,error); return UNKNOWN_ERROR;
    }
    lock.lock();
    mJoinThread.reset(); mShutdownTrigger.reset(); mWindowsNdrState = WindowsNdrState::Joined;
    windowsNdrSnapshotLocked(state,error); mShutdownCv.notify_all(); return OK;
}
} // namespace android
