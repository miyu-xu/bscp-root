/*
 * Copyright (C) 2020 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#define LOG_TAG "libbinder.RpcServer"

#include <inttypes.h>
#include <netinet/tcp.h>
#include <poll.h>
#include <sys/socket.h>
#include <sys/un.h>

#include <vector>

#include <binder/Functional.h>
#include <binder/Parcel.h>
#include <binder/RpcServer.h>
#ifdef _WIN32
#include "RpcTransportNdr.h"
#include "WindowsNdrRecordAdapter.h"
#include "WindowsCallerIdentity.h"
#include "WindowsGuestListener.h"
#endif
#include <binder/RpcThreads.h>
#include <binder/RpcTransportRaw.h>
#include <log/log.h>

#include "BuildFlags.h"
#include "FdTrigger.h"
#include "OS.h"
#include "RpcSocketAddress.h"
#include "RpcState.h"
#include "RpcTransportUtils.h"
#include "RpcWireFormat.h"
#include "Utils.h"

namespace android {
#ifdef _WIN32
using bscp::windows::socket;
using bscp::windows::getsockname;
using bscp::windows::getpeername;
using bscp::windows::socketpair;
using bscp::windows::connect;
using bscp::windows::bind;
using bscp::windows::listen;
using bscp::windows::accept4;
using bscp::windows::setsockopt;
using bscp::windows::getsockopt;
#endif


constexpr size_t kSessionIdBytes = 32;

using namespace android::binder::impl;
using android::binder::borrowed_fd;
using android::binder::unique_fd;

RpcServer::RpcServer(std::unique_ptr<RpcTransportCtx> ctx
#ifdef _WIN32
                       , bool windowsDesktop, std::shared_ptr<WlrServerAcceptor> acceptor
#endif
                       ) :
#ifdef _WIN32
                         mWindowsDesktop(windowsDesktop), mWindowsAcceptor(std::move(acceptor)),
#endif
                         mCtx(std::move(ctx)) {}
#ifdef _WIN32
sp<RpcServer> RpcServer::makeWindowsDesktop(WlrServerAcceptor** owned) {
    return makeWindowsDesktopWithRoot(owned, nullptr);
}
sp<RpcServer> RpcServer::makeWindowsDesktopWithRoot(WlrServerAcceptor** owned, const sp<IBinder>& root) {
    if (!owned || !*owned || WlrAcceptorCheckLive(*owned)) return nullptr;
    struct Holder { WlrServerAcceptor* cap = nullptr; ~Holder() {
        if (cap && WlrReleaseAcceptor(&cap)) ALOGE("NDR acceptor release UNKNOWN"); } };
    auto holder = std::make_shared<Holder>();
    auto factory = binder::os::makeDefaultRpcTransportCtxFactory();
    auto ctx = factory->newServerCtx();
    if (!ctx) return nullptr;
    auto capability = std::shared_ptr<WlrServerAcceptor>(holder, *owned);
    auto server = sp<RpcServer>::make(std::move(ctx), true, std::move(capability));
    if (root) server->setRootObject(root);
    holder->cap = *owned; *owned = nullptr;
    return server;
}
#endif
RpcServer::~RpcServer() {
    RpcMutexUniqueLock _l(mLock);
    LOG_ALWAYS_FATAL_IF(mShutdownTrigger != nullptr, "Must call shutdown() before destructor");
}

sp<RpcServer> RpcServer::make(std::unique_ptr<RpcTransportCtxFactory> rpcTransportCtxFactory) {
    // Default is without TLS.
    if (rpcTransportCtxFactory == nullptr)
        rpcTransportCtxFactory = binder::os::makeDefaultRpcTransportCtxFactory();
    auto ctx = rpcTransportCtxFactory->newServerCtx();
    if (ctx == nullptr) return nullptr;
    return sp<RpcServer>::make(std::move(ctx));
}

status_t RpcServer::setupUnixDomainSocketBootstrapServer(unique_fd bootstrapFd) {
    return setupExternalServer(std::move(bootstrapFd), &RpcServer::recvmsgSocketConnection);
}

status_t RpcServer::setupUnixDomainServer(const char* path) {
    return setupSocketServer(UnixSocketAddress(path));
}

status_t RpcServer::setupVsockServer(unsigned bindCid, unsigned port, unsigned* assignedPort) {
    auto status = setupSocketServer(VsockSocketAddress(bindCid, port));
    if (status != OK) return status;

    if (assignedPort == nullptr) return OK;
    sockaddr_vm addr;
    socklen_t len = sizeof(addr);
    if (0 != getsockname(mServer.fd.get(), reinterpret_cast<sockaddr*>(&addr), &len)) {
        status = -errno;
        ALOGE("setupVsockServer: Failed to getsockname: %s", strerror(-status));
        return status;
    }

    LOG_ALWAYS_FATAL_IF(len != sizeof(addr), "Wrong socket type: len %zu vs len %zu",
                        static_cast<size_t>(len), sizeof(addr));
    *assignedPort = addr.svm_port;
    return OK;
}

status_t RpcServer::setupInetServer(const char* address, unsigned int port,
                                    unsigned int* assignedPort) {
    if (assignedPort != nullptr) *assignedPort = 0;
    auto aiStart = InetSocketAddress::getAddrInfo(address, port);
    if (aiStart == nullptr) return UNKNOWN_ERROR;
    for (auto ai = aiStart.get(); ai != nullptr; ai = ai->ai_next) {
        if (ai->ai_addr == nullptr) continue;
        InetSocketAddress socketAddress(ai->ai_addr, ai->ai_addrlen, address, port);
        if (status_t status = setupSocketServer(socketAddress); status != OK) {
            continue;
        }

        LOG_ALWAYS_FATAL_IF(socketAddress.addr()->sa_family != AF_INET, "expecting inet");
        sockaddr_in addr{};
        socklen_t len = sizeof(addr);
        if (0 != getsockname(mServer.fd.get(), reinterpret_cast<sockaddr*>(&addr), &len)) {
            int savedErrno = errno;
            ALOGE("Could not getsockname at %s: %s", socketAddress.toString().c_str(),
                  strerror(savedErrno));
            return -savedErrno;
        }
        LOG_ALWAYS_FATAL_IF(len != sizeof(addr), "Wrong socket type: len %zu vs len %zu",
                            static_cast<size_t>(len), sizeof(addr));
        unsigned int realPort = ntohs(addr.sin_port);
        LOG_ALWAYS_FATAL_IF(port != 0 && realPort != port,
                            "Requesting inet server on %s but it is set up on %u.",
                            socketAddress.toString().c_str(), realPort);

        if (assignedPort != nullptr) {
            *assignedPort = realPort;
        }

        return OK;
    }
    ALOGE("None of the socket address resolved for %s:%u can be set up as inet server.", address,
          port);
    return UNKNOWN_ERROR;
}

void RpcServer::setMaxThreads(size_t threads) {
#ifdef BINDER_RPC_SINGLE_THREADED
    LOG_ALWAYS_FATAL_IF(threads > 1, "Cannot set max threads > 1 in single-threaded mode");
#endif // BINDER_RPC_SINGLE_THREADED
    LOG_ALWAYS_FATAL_IF(threads <= 0, "RpcServer is useless without threads");
    LOG_ALWAYS_FATAL_IF(mJoinThreadRunning, "Cannot set max threads while running");
    if (sp<IBinder> root = getRootObject(); root != nullptr) {
        LOG_ALWAYS_FATAL_IF(threads < root->getMinRpcThreads(),
                            "RpcServer can not satisfy the root object's minimum "
                            "thread requirements of %d with the requested max "
                            "thread count of %zu",
                            root->getMinRpcThreads(), threads);
    }
    mMaxThreads = threads;
}

size_t RpcServer::getMaxThreads() {
    return mMaxThreads;
}

bool RpcServer::setProtocolVersion(uint32_t version) {
    if (!RpcState::validateProtocolVersion(version)) {
        return false;
    }

    mProtocolVersion = version;
    return true;
}

void RpcServer::setSupportedFileDescriptorTransportModes(
        const std::vector<RpcSession::FileDescriptorTransportMode>& modes) {
    mSupportedFileDescriptorTransportModes.reset();
    for (RpcSession::FileDescriptorTransportMode mode : modes) {
        mSupportedFileDescriptorTransportModes.set(static_cast<size_t>(mode));
    }
}

static void logMaxThreadOverwrite(size_t max, uint16_t min, bool isPerSession) {
    ALOGW("Overriding this RpcServer's max threads (%zu) based on the min threads required "
          "(%d) by the %s object that is being set.",
          max, min, isPerSession ? "per-session root" : "root");
}

void RpcServer::maybeOverwriteMaxThreads(const sp<IBinder>& binder) {
    uint16_t minThreads = binder->getMinRpcThreads();
    if (mMaxThreads < minThreads) {
        logMaxThreadOverwrite(mMaxThreads, minThreads, false);
        mMaxThreads = binder->getMinRpcThreads();
    }
}

void RpcServer::setRootObject(const sp<IBinder>& binder) {
    RpcMutexLockGuard _l(mLock);
    LOG_ALWAYS_FATAL_IF(mJoinThreadRunning, "Cannot set root object while running");
    mRootObjectFactory = nullptr;
    mRootObjectWeak = mRootObject = binder;
    maybeOverwriteMaxThreads(binder);
}

void RpcServer::setRootObjectWeak(const wp<IBinder>& binder) {
    RpcMutexLockGuard _l(mLock);
    LOG_ALWAYS_FATAL_IF(mJoinThreadRunning, "Cannot set root object while running");
    mRootObject.clear();
    mRootObjectFactory = nullptr;
    mRootObjectWeak = binder;

    sp<IBinder> s = binder.promote();
    if (s == nullptr) return;
    maybeOverwriteMaxThreads(s);
}
void RpcServer::setPerSessionRootObject(
        std::function<sp<IBinder>(wp<RpcSession> session, const void*, size_t)>&& makeObject) {
    RpcMutexLockGuard _l(mLock);
    mRootObject.clear();
    mRootObjectWeak.clear();
    mRootObjectFactory = std::move(makeObject);
}

void RpcServer::setConnectionFilter(std::function<bool(const void*, size_t)>&& filter) {
    RpcMutexLockGuard _l(mLock);
    LOG_ALWAYS_FATAL_IF(mShutdownTrigger != nullptr, "Already joined");
    mConnectionFilter = std::move(filter);
}

void RpcServer::setServerSocketModifier(std::function<void(borrowed_fd)>&& modifier) {
    RpcMutexLockGuard _l(mLock);
    LOG_ALWAYS_FATAL_IF(mServer.fd.ok(), "Already started");
    mServerSocketModifier = std::move(modifier);
}

sp<IBinder> RpcServer::getRootObject() {
    RpcMutexLockGuard _l(mLock);
    bool hasWeak = mRootObjectWeak.unsafe_get();
    sp<IBinder> ret = mRootObjectWeak.promote();
    ALOGW_IF(hasWeak && ret == nullptr, "RpcServer root object is freed, returning nullptr");
    return ret;
}

std::vector<uint8_t> RpcServer::getCertificate(RpcCertificateFormat format) {
    RpcMutexLockGuard _l(mLock);
    return mCtx->getCertificate(format);
}

static void joinRpcServer(sp<RpcServer>&& thiz) {
    thiz->join();
}

void RpcServer::start() {
    RpcMutexLockGuard _l(mLock);
#ifdef _WIN32
    if (mWindowsManaged) { ALOGE("Legacy start rejected for managed NDR server"); return; }
#endif
    LOG_ALWAYS_FATAL_IF(mJoinThread.get() != nullptr, "Already started!");
    mJoinThread =
            std::make_unique<RpcMaybeThread>(&joinRpcServer, sp<RpcServer>::fromExisting(this));
    rpcJoinIfSingleThreaded(*mJoinThread);
}

status_t RpcServer::acceptSocketConnection(const RpcServer& server, RpcTransportFd* out) {
    RpcTransportFd clientSocket(unique_fd(TEMP_FAILURE_RETRY(
            accept4(server.mServer.fd.get(), nullptr, nullptr, SOCK_CLOEXEC | SOCK_NONBLOCK))));
    if (!clientSocket.fd.ok()) {
        int savedErrno = errno;
        ALOGE("Could not accept4 socket: %s", strerror(savedErrno));
        return -savedErrno;
    }

    *out = std::move(clientSocket);
    return OK;
}

status_t RpcServer::recvmsgSocketConnection(const RpcServer& server, RpcTransportFd* out) {
    int zero = 0;
    iovec iov{&zero, sizeof(zero)};
    std::vector<std::variant<unique_fd, borrowed_fd>> fds;

    // TODO: this should delegate to RpcState::rpcRec to handle transaction errors
    // consistently
    ssize_t num_bytes = binder::os::receiveMessageFromSocket(server.mServer, &iov, 1, &fds);
    if (num_bytes < 0) {
        int savedErrno = errno;
        ALOGE("Failed recvmsg: %s", strerror(savedErrno));

        if (savedErrno == ECONNRESET) {
            return DEAD_OBJECT;
        }

        return -savedErrno;
    }
    if (num_bytes == 0) {
        return DEAD_OBJECT;
    }
    if (fds.size() != 1) {
        ALOGE("Expected exactly one fd from recvmsg, got %zu", fds.size());
        return -EINVAL;
    }

    unique_fd fd(std::move(std::get<unique_fd>(fds.back())));
    if (status_t res = binder::os::setNonBlocking(fd); res != OK) return res;

    *out = RpcTransportFd(std::move(fd));
    return OK;
}

void RpcServer::join() {
    {
        RpcMutexLockGuard _l(mLock);
#ifdef _WIN32
        if (mWindowsManaged) { ALOGE("Legacy join rejected for managed NDR server"); return; }
#endif
        #ifdef _WIN32
        if (!mWindowsDesktop && !mWindowsGuest) {
#endif
        LOG_ALWAYS_FATAL_IF(!mServer.fd.ok(), "RpcServer must be setup to join.");
        LOG_ALWAYS_FATAL_IF(mAcceptFn == nullptr, "RpcServer must have an accept() function");
#ifdef _WIN32
        }
#endif
        LOG_ALWAYS_FATAL_IF(mShutdownTrigger != nullptr, "Already joined");
        mJoinThreadRunning = true;
        mShutdownTrigger = FdTrigger::make();
        LOG_ALWAYS_FATAL_IF(mShutdownTrigger == nullptr, "Cannot create join signaler");
    }

    status_t status = OK;
    while (
#ifdef _WIN32
           (mWindowsDesktop || mWindowsGuest) ? !mShutdownTrigger->isTriggered() :
#endif
           (status = mShutdownTrigger->triggerablePoll(mServer, POLLIN)) == OK) {
#ifdef _WIN32
        if(mWindowsGuest){
            status=acceptWindowsGuestConnection();
            if(status==PERMISSION_DENIED)continue;
            if(status!=OK)break;
            continue;
        }
        if (mWindowsDesktop) {
            status = acceptWindowsNdrConnection();
            if (status == WOULD_BLOCK) { Sleep(5); continue; }
            if (status != OK) break;
            continue;
        }
#endif
        status = acceptConnection(RpcSession::join);
        if (status == DEAD_OBJECT) {
            break;
        }
    }
    LOG_RPC_DETAIL("RpcServer::join exiting with %s", statusToString(status).c_str());

    if constexpr (kEnableRpcThreads) {
        RpcMutexLockGuard _l(mLock);
        mJoinThreadRunning = false;
    } else {
        // Multi-threaded builds clear this in shutdown(), but we need it valid
        // so the loop above exits cleanly
        mShutdownTrigger = nullptr;
    }
    mShutdownCv.notify_all();
}

status_t RpcServer::acceptConnection(
        std::function<void(sp<RpcSession>&&, RpcSession::PreJoinSetupResult&&)>&& joinFn) {
    RpcTransportFd clientFd;
    std::array<uint8_t, kRpcAddressSize> addr;
    static_assert(addr.size() >= sizeof(sockaddr_storage), "kRpcAddressSize is too small");
    socklen_t addrLen = addr.size();

    status_t status;
    if ((status = mAcceptFn(*this, &clientFd)) != OK) {
        if (status != DEAD_OBJECT) {
            ALOGE("Accept returned error %s", statusToString(status).c_str());
        }
        return status;
    }

    LOG_RPC_DETAIL("accept on fd %d yields fd %d", mServer.fd.get(), clientFd.fd.get());

    if (getpeername(clientFd.fd.get(), reinterpret_cast<sockaddr*>(addr.data()), &addrLen)) {
        ALOGE("Could not getpeername socket: %s", strerror(errno));
        return EINVAL;
    }

    if (mConnectionFilter != nullptr && !mConnectionFilter(addr.data(), addrLen)) {
        ALOGE("Dropped client connection fd %d", clientFd.fd.get());
        return EINVAL;
    }

    {
        RpcMutexLockGuard _l(mLock);
        RpcMaybeThread thread =
                RpcMaybeThread(&RpcServer::establishConnection, sp<RpcServer>::fromExisting(this),
                               std::move(clientFd), addr, addrLen, std::move(joinFn), nullptr);
        auto& threadRef = mConnectingThreads[thread.get_id()];
        threadRef = std::move(thread);
        rpcJoinIfSingleThreaded(threadRef);
    }

    return OK;
}

#ifdef _WIN32
status_t RpcServer::setupWindowsGuestServer(uint32_t cid,uint32_t port){
    RpcMutexLockGuard lock(mLock);
    if(mWindowsDesktop||mWindowsGuest||mServer.fd.ok()||mShutdownTrigger)return INVALID_OPERATION;
    return WindowsGuestListener::create(cid,port,&mWindowsGuest);
}
status_t RpcServer::bindWindowsGuestProcess(void* process){
    RpcMutexLockGuard lock(mLock);
    if(!mWindowsGuest||mShutdownTrigger||mJoinThreadRunning)return INVALID_OPERATION;
    return mWindowsGuest->bindOriginalProcess(process);
}
status_t RpcServer::acceptWindowsGuestConnection(){
    std::unique_ptr<RpcTransport> transport;
    status_t s=mWindowsGuest->accept(mShutdownTrigger.get(),&transport);
    if(s!=OK)return s;
    if(!transport||!transport->guestProof()||transport->guestProof()->check()!=OK)return PERMISSION_DENIED;
    RpcMutexLockGuard lock(mLock);
    if(mShutdownTrigger->isTriggered())return DEAD_OBJECT;
    decltype(mConnectingThreads) staging;
    auto empty=staging.try_emplace(RpcMaybeThread::id{});auto node=staging.extract(empty.first);
    RpcMaybeThread thread(&RpcServer::establishConnection,sp<RpcServer>::fromExisting(this),
        RpcTransportFd(),std::array<uint8_t,kRpcAddressSize>{},size_t{0},
        std::function<void(sp<RpcSession>&&,RpcSession::PreJoinSetupResult&&)>(RpcSession::join),std::move(transport));
    node.key()=thread.get_id();node.mapped()=std::move(thread);
    auto inserted=mConnectingThreads.insert(std::move(node));
    LOG_ALWAYS_FATAL_IF(!inserted.inserted,"Live Guest connection thread duplicated");
    rpcJoinIfSingleThreaded(inserted.position->second);return OK;
}
status_t RpcServer::acceptWindowsNdrConnection() {
    std::unique_ptr<RpcTransportNdr> accepted;
    status_t status = RpcTransportNdr::tryAccept(mWindowsAcceptor.get(), &accepted);
    if (status != OK) return status;
    RpcMutexLockGuard lock(mLock);
    if (mShutdownTrigger->isTriggered()) return mWindowsManaged ? WOULD_BLOCK : DEAD_OBJECT;
    std::unique_ptr<RpcTransport> transport(std::move(accepted));
    // Allocate the registry node before starting a thread. Otherwise allocation
    // failure after std::thread construction destroys a joinable local thread.
    decltype(mConnectingThreads) staging;
    auto empty = staging.try_emplace(RpcMaybeThread::id{});
    auto node = staging.extract(empty.first);
    RpcMaybeThread thread(&RpcServer::establishConnection, sp<RpcServer>::fromExisting(this),
        RpcTransportFd(), std::array<uint8_t, kRpcAddressSize>{}, size_t{0},
        std::function<void(sp<RpcSession>&&, RpcSession::PreJoinSetupResult&&)>(RpcSession::join),
        std::move(transport));
    node.key() = thread.get_id();
    node.mapped() = std::move(thread);
    auto inserted = mConnectingThreads.insert(std::move(node)); // same allocator: no allocation
    LOG_ALWAYS_FATAL_IF(!inserted.inserted, "Live Windows connection thread ID duplicated");
    rpcJoinIfSingleThreaded(inserted.position->second);
    return OK;
}
#endif

bool RpcServer::shutdown() {
    RpcMutexUniqueLock _l(mLock);
#ifdef _WIN32
    if (mWindowsManaged) { ALOGE("Legacy shutdown rejected for managed NDR server"); return false; }
#endif
    if (mShutdownTrigger == nullptr) {
        LOG_RPC_DETAIL("Cannot shutdown. No shutdown trigger installed (already shutdown, or not "
                       "joined yet?)");
        return false;
    }

    mShutdownTrigger->trigger();

    for (auto& [id, session] : mSessions) {
        (void)id;
        // server lock is a more general lock
        RpcMutexLockGuard _lSession(session->mMutex);
        session->mShutdownTrigger->trigger();
    }

    if constexpr (!kEnableRpcThreads) {
        // In single-threaded mode we're done here, everything else that
        // needs to happen should be at the end of RpcServer::join()
        return true;
    }

    while (mJoinThreadRunning || !mConnectingThreads.empty() || !mSessions.empty()) {
        if (mShutdownCv.wait_for(_l, std::chrono::seconds(1)) == RpcCvStatus::timeout) {
            ALOGE("Waiting for RpcServer to shut down (1s w/o progress). Join thread running: %d, "
                  "Connecting threads: "
                  "%zu, Sessions: %zu. Is your server deadlocked?",
                  mJoinThreadRunning, mConnectingThreads.size(), mSessions.size());
        }
    }

    // At this point, we know join() is about to exit, but the thread that calls
    // join() may not have exited yet.
    // If RpcServer owns the join thread (aka start() is called), make sure the thread exits;
    // otherwise ~thread() may call std::terminate(), which may crash the process.
    // If RpcServer does not own the join thread (aka join() is called directly),
    // then the owner of RpcServer is responsible for cleaning up that thread.
    if (mJoinThread.get()) {
        mJoinThread->join();
        mJoinThread.reset();
    }

    mServer = RpcTransportFd();
#ifdef _WIN32
    mWindowsGuest.reset();
#endif

    LOG_RPC_DETAIL("Finished waiting on shutdown.");

    mShutdownTrigger = nullptr;
    return true;
}

std::vector<sp<RpcSession>> RpcServer::listSessions() {
    RpcMutexLockGuard _l(mLock);
    std::vector<sp<RpcSession>> sessions;
    for (auto& [id, session] : mSessions) {
        (void)id;
        sessions.push_back(session);
    }
    return sessions;
}

size_t RpcServer::numUninitializedSessions() {
    RpcMutexLockGuard _l(mLock);
    return mConnectingThreads.size();
}

void RpcServer::establishConnection(
        sp<RpcServer>&& server, RpcTransportFd clientFd, std::array<uint8_t, kRpcAddressSize> addr,
        size_t addrLen,
        std::function<void(sp<RpcSession>&&, RpcSession::PreJoinSetupResult&&)>&& joinFn,
        std::unique_ptr<RpcTransport> accepted) {
    // mShutdownTrigger can only be cleared once connection threads have joined.
    // It must be set before this thread is started
    LOG_ALWAYS_FATAL_IF(server->mShutdownTrigger == nullptr);
    LOG_ALWAYS_FATAL_IF(server->mCtx == nullptr);

    status_t status = OK;

    int clientFdForLog = clientFd.fd.get();
#ifdef BSCP_DESKTOP_RPC_AUTH
    std::optional<desktop_rpc::Peer> authenticatedPeer;
    if (server->mDesktopOwner) {
        desktop_rpc::Peer peer;
        const auto [owner, group] = *server->mDesktopOwner;
        if (desktop_rpc::capture(clientFd.fd.get(), owner, group, &peer) != 0)
            status = PERMISSION_DENIED;
        else authenticatedPeer = std::move(peer);
    }
#endif

    std::optional<uid_t> uid = std::nullopt;
    if (status == OK && !accepted) status = RpcSocketAddress::getPeerUid(clientFd.fd,
                                          reinterpret_cast<const sockaddr*>(addr.data()), addrLen,
                                          &uid);
    if (status != OK) {
        ALOGE("Failed to get peer credentials: %s", statusToString(status).c_str());
    }

    auto client = accepted ? std::move(accepted) : server->mCtx->newTransport(std::move(clientFd), server->mShutdownTrigger.get());
    if (client == nullptr) {
        ALOGE("Dropping accept4()-ed socket because sslAccept fails");
        status = DEAD_OBJECT;
        // still need to cleanup before we can return
    } else {
        LOG_RPC_DETAIL("Created RpcTransport %p for client fd %d", client.get(), clientFdForLog);
    }

#ifdef _WIN32
    const bool required = server->mWindowsDesktop;
    // Only this private path accepts a transport from the concrete factory.
    const uint64_t ndrGeneration = required && client &&
        client->ndrDirection() == RpcTransport::NdrDirection::ServerAccepted
        ? static_cast<RpcTransportNdr*>(client.get())->authenticatedSessionGeneration() : 0;
    std::shared_ptr<const bscp::windows::WindowsNdrRequestSnapshot> admission;
    if (client && ((required && client->ndrDirection() != RpcTransport::NdrDirection::ServerAccepted) ||
                   (!required && client->ndrDirection() != RpcTransport::NdrDirection::NotNdr)))
        status = PERMISSION_DENIED;
#endif
    RpcConnectionHeader header{};
    if (status == OK) {
#ifdef _WIN32
        if (required) status = client->beginNdrRecord(RpcTransport::NdrPhase::ConnectionHeader);
#endif
        iovec iov{&header, sizeof(header)};
        if (status == OK) status = client->interruptableReadFully(server->mShutdownTrigger.get(), &iov, 1,
                                                std::nullopt, /*ancillaryFds=*/nullptr);
        if (status != OK) {
            ALOGE("Failed to read ID for client connecting to RPC server: %s",
                  statusToString(status).c_str());
            // still need to cleanup before we can return
        }
    }

#ifdef _WIN32
    if (status == OK && required) {
        if ((header.options & ~RPC_CONNECTION_OPTION_INCOMING) ||
            (header.sessionIdSize && header.sessionIdSize != kSessionIdBytes) ||
            header.fileDescriptorTransportMode >= server->mSupportedFileDescriptorTransportModes.size() ||
            !server->mSupportedFileDescriptorTransportModes.test(header.fileDescriptorTransportMode))
            status = BAD_VALUE;
        for (auto byte : header.reservered) if (byte) status = BAD_VALUE;
        if (status == OK) status = client->finishNdrRecord(&admission);
        if (status == OK && !admission) status = PERMISSION_DENIED;
        if (status == OK && server->mConnectionFilter) {
            bscp::windows::CallerIdentity allowed, caller;
            if (bscp::windows::CallerIdentity::currentProcess(&allowed) ||
                bscp::windows::importAuthenticatedWindowsNdrSnapshot(admission, allowed, &caller))
                status = PERMISSION_DENIED;
            else {
                bscp::windows::CallerScope scope(&caller);
                // NDR has no sockaddr. The callback's documented size check remains mandatory.
                if (!server->mConnectionFilter(nullptr, 0)) status = PERMISSION_DENIED;
            }
        }
    }
#endif
    std::vector<uint8_t> sessionId;
    if (status == OK) {
        if (header.sessionIdSize > 0) {
            if (header.sessionIdSize == kSessionIdBytes) {
                sessionId.resize(header.sessionIdSize);
                iovec iov{sessionId.data(), sessionId.size()};
#ifdef _WIN32
                if (required) status = client->beginNdrRecord(RpcTransport::NdrPhase::SessionId);
#endif
                if (status == OK) status = client->interruptableReadFully(server->mShutdownTrigger.get(), &iov, 1,
                                                        std::nullopt, /*ancillaryFds=*/nullptr);
#ifdef _WIN32
                if (status == OK && required) {
                    std::shared_ptr<const bscp::windows::WindowsNdrRequestSnapshot> part;
                    status = client->finishNdrRecord(&part);
                    bool same = false;
                    if (status == OK && (!part || admission->matchesOriginalSnapshot(*part, &same) || !same))
                        status = PERMISSION_DENIED;
                }
#endif
                if (status != OK) {
                    ALOGE("Failed to read session ID for client connecting to RPC server: %s",
                          statusToString(status).c_str());
                    // still need to cleanup before we can return
                }
            } else {
                ALOGE("Malformed session ID. Expecting session ID of size %zu but got %" PRIu16,
                      kSessionIdBytes, header.sessionIdSize);
                status = BAD_VALUE;
            }
        }
    }

    bool incoming = false;
    uint32_t protocolVersion = 0;
    bool requestingNewSession = false;

    if (status == OK) {
        incoming = header.options & RPC_CONNECTION_OPTION_INCOMING;
        protocolVersion = std::min(header.version,
                                   server->mProtocolVersion.value_or(RPC_WIRE_PROTOCOL_VERSION));
        requestingNewSession = sessionId.empty();

        if (requestingNewSession) {
            RpcNewSessionResponse response{
                    .version = protocolVersion,
            };

            iovec iov{&response, sizeof(response)};
            status = client->interruptableWriteFully(server->mShutdownTrigger.get(), &iov, 1,
                                                     std::nullopt, nullptr);
            if (status != OK) {
                ALOGE("Failed to send new session response: %s", statusToString(status).c_str());
                // still need to cleanup before we can return
            }
        }
    }

    RpcMaybeThread thisThread;
    sp<RpcSession> session;
    {
        RpcMutexUniqueLock _l(server->mLock);

        auto threadId = server->mConnectingThreads.find(rpc_this_thread::get_id());
        LOG_ALWAYS_FATAL_IF(threadId == server->mConnectingThreads.end(),
                            "Must establish connection on owned thread");
        thisThread = std::move(threadId->second);
        auto detachGuardLambda = [&]() {
            thisThread.detach();
            _l.unlock();
            server->mShutdownCv.notify_all();
        };
        auto detachGuard = make_scope_guard(std::ref(detachGuardLambda));
        server->mConnectingThreads.erase(threadId);

        if (status != OK || server->mShutdownTrigger->isTriggered()) {
            return;
        }

        if (requestingNewSession) {
            if (incoming) {
                ALOGE("Cannot create a new session with an incoming connection, would leak");
                return;
            }

            // Uniquely identify session at the application layer. Even if a
            // client/server use the same certificates, if they create multiple
            // sessions, we still want to distinguish between them.
            sessionId.resize(kSessionIdBytes);
            size_t tries = 0;
            do {
                // don't block if there is some entropy issue
                if (tries++ > 5) {
                    ALOGE("Cannot find new address: %s",
                          HexString(sessionId.data(), sessionId.size()).c_str());
                    return;
                }

                auto result = binder::os::getRandomBytes(sessionId.data(), sessionId.size());
                if (result != OK) {
                    ALOGE("Failed to read random session ID: %s", statusToString(result).c_str());
                    return;
                }
            } while (server->mSessions.end() != server->mSessions.find(sessionId));

#ifdef _WIN32
            session = sp<RpcSession>::make(nullptr, uid,
                required ? RpcSession::WindowsPolicy::DesktopServer : RpcSession::WindowsPolicy::RawGuest);
            session->mWindowsAdmission = admission;
            session->mWindowsNdrGeneration = ndrGeneration;
#else
            session = sp<RpcSession>::make(nullptr, uid);
#endif
#ifdef BSCP_DESKTOP_RPC_AUTH
            session->mDesktopPeer = authenticatedPeer;
#endif

            if (!session->setProtocolVersion(protocolVersion)) return;

            if (header.fileDescriptorTransportMode <
                        server->mSupportedFileDescriptorTransportModes.size() &&
                server->mSupportedFileDescriptorTransportModes.test(
                        header.fileDescriptorTransportMode)) {
                session->setFileDescriptorTransportMode(
                        static_cast<RpcSession::FileDescriptorTransportMode>(
                                header.fileDescriptorTransportMode));
            } else {
                ALOGE("Rejecting connection: FileDescriptorTransportMode is not supported: %hhu",
                      header.fileDescriptorTransportMode);
                return;
            }

            // if null, falls back to server root
            sp<IBinder> sessionSpecificRoot;
            if (server->mRootObjectFactory != nullptr) {
#ifdef _WIN32
                if (required) {
                    bscp::windows::CallerIdentity allowed, caller;
                    if (bscp::windows::CallerIdentity::currentProcess(&allowed) ||
                        bscp::windows::importAuthenticatedWindowsNdrSnapshot(admission, allowed, &caller))
                        return;
                    bscp::windows::CallerScope scope(&caller);
                    sessionSpecificRoot = server->mRootObjectFactory(wp<RpcSession>(session), nullptr, 0);
                } else
#endif
                {
                    sessionSpecificRoot = server->mRootObjectFactory(wp<RpcSession>(session), addr.data(), addrLen);
                }
                if (sessionSpecificRoot == nullptr) {
                    ALOGE("Warning: server returned null from root object factory");
                } else {
                    server->maybeOverwriteMaxThreads(sessionSpecificRoot);
                }
            }
            session->setMaxIncomingThreads(server->mMaxThreads);

            if (!session->setForServer(server, server, sessionId, sessionSpecificRoot)) {
                ALOGE("Failed to attach server to session");
                return;
            }

            server->mSessions[sessionId] = session;
        } else {
            auto it = server->mSessions.find(sessionId);
            if (it == server->mSessions.end()) {
                ALOGE("Cannot add thread, no record of session with ID %s",
                      HexString(sessionId.data(), sessionId.size()).c_str());
                return;
            }
            session = it->second;
#ifdef _WIN32
            const auto expectedPolicy = required ? RpcSession::WindowsPolicy::DesktopServer : RpcSession::WindowsPolicy::RawGuest;
            if (session->mWindowsPolicy != expectedPolicy) return;
            if (required) {
                bool same = false;
                if (session->mWindowsNdrGeneration != ndrGeneration || !session->mWindowsAdmission || !admission ||
                    session->mWindowsAdmission->matchesOriginalSnapshot(*admission, &same) || !same) return;
            }
#endif
#ifdef BSCP_DESKTOP_RPC_AUTH
            if (session->mDesktopPeer != authenticatedPeer) {
                ALOGE("Rejecting RPC attachment: original authenticated peer differs or exited");
                return;
            }
#endif
        }

        if (incoming) {
            LOG_ALWAYS_FATAL_IF(OK != session->addOutgoingConnection(std::move(client), true),
                                "server state must already be initialized");
            return;
        }

        detachGuard.release();
        session->preJoinThreadOwnership(std::move(thisThread));
    }

    auto setupResult = session->preJoinSetup(std::move(client));

    // avoid strong cycle
    server = nullptr;

    joinFn(std::move(session), std::move(setupResult));
}

status_t RpcServer::setupSocketServer(const RpcSocketAddress& addr) {
    LOG_RPC_DETAIL("Setting up socket server %s", addr.toString().c_str());
    LOG_ALWAYS_FATAL_IF(hasServer(), "Each RpcServer can only have one server.");

    unique_fd socket_fd(TEMP_FAILURE_RETRY(
            socket(addr.addr()->sa_family, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0)));
    if (!socket_fd.ok()) {
        int savedErrno = errno;
        ALOGE("Could not create socket at %s: %s", addr.toString().c_str(), strerror(savedErrno));
        return -savedErrno;
    }

    if (addr.addr()->sa_family == AF_INET || addr.addr()->sa_family == AF_INET6) {
        int noDelay = 1;
        int result =
                setsockopt(socket_fd.get(), IPPROTO_TCP, TCP_NODELAY, &noDelay, sizeof(noDelay));
        if (result < 0) {
            int savedErrno = errno;
            ALOGE("Could not set TCP_NODELAY on  %s", strerror(savedErrno));
            return -savedErrno;
        }
    }

    {
        RpcMutexLockGuard _l(mLock);
        if (mServerSocketModifier != nullptr) {
            mServerSocketModifier(socket_fd);
        }
    }

    if (0 != TEMP_FAILURE_RETRY(bind(socket_fd.get(), addr.addr(), addr.addrSize()))) {
        int savedErrno = errno;
        ALOGE("Could not bind socket at %s: %s", addr.toString().c_str(), strerror(savedErrno));
        return -savedErrno;
    }

    return setupRawSocketServer(std::move(socket_fd));
}

status_t RpcServer::setupRawSocketServer(unique_fd socket_fd) {
    LOG_ALWAYS_FATAL_IF(!socket_fd.ok(), "Socket must be setup to listen.");

    // Right now, we create all threads at once, making accept4 slow. To avoid hanging the client,
    // the backlog is increased to a large number.
    // TODO(b/189955605): Once we create threads dynamically & lazily, the backlog can be reduced
    //  to 1.
    if (0 != TEMP_FAILURE_RETRY(listen(socket_fd.get(), 50 /*backlog*/))) {
        int savedErrno = errno;
        ALOGE("Could not listen initialized Unix socket: %s", strerror(savedErrno));
        return -savedErrno;
    }
    if (status_t status = setupExternalServer(std::move(socket_fd)); status != OK) {
        ALOGE("Another thread has set up server while calling setupSocketServer. Race?");
        return status;
    }
    return OK;
}

void RpcServer::onSessionAllIncomingThreadsEnded(const sp<RpcSession>& session) {
    const std::vector<uint8_t>& id = session->mId;
    LOG_ALWAYS_FATAL_IF(id.empty(), "Server sessions must be initialized with ID");
    LOG_RPC_DETAIL("Dropping session with address %s", HexString(id.data(), id.size()).c_str());

    RpcMutexLockGuard _l(mLock);
    auto it = mSessions.find(id);
    LOG_ALWAYS_FATAL_IF(it == mSessions.end(), "Bad state, unknown session id %s",
                        HexString(id.data(), id.size()).c_str());
    LOG_ALWAYS_FATAL_IF(it->second != session, "Bad state, session has id mismatch %s",
                        HexString(id.data(), id.size()).c_str());
    (void)mSessions.erase(it);
}

void RpcServer::onSessionIncomingThreadEnded() {
    mShutdownCv.notify_all();
}

bool RpcServer::hasServer() {
    RpcMutexLockGuard _l(mLock);
    return mServer.fd.ok();
}

unique_fd RpcServer::releaseServer() {
    RpcMutexLockGuard _l(mLock);
    return std::move(mServer.fd);
}

status_t RpcServer::setupExternalServer(
        unique_fd serverFd, std::function<status_t(const RpcServer&, RpcTransportFd*)>&& acceptFn) {
    if (status_t res = binder::os::setNonBlocking(serverFd); res != OK) return res;

    RpcMutexLockGuard _l(mLock);
    if (mServer.fd.ok()) {
        ALOGE("Each RpcServer can only have one server.");
        return INVALID_OPERATION;
    }
    mServer = std::move(serverFd);
    mAcceptFn = std::move(acceptFn);
    return OK;
}

status_t RpcServer::setupExternalServer(unique_fd serverFd) {
    return setupExternalServer(std::move(serverFd), &RpcServer::acceptSocketConnection);
}

bool RpcServer::hasActiveRequests() {
    RpcMutexLockGuard _l(mLock);
    for (const auto& [_, session] : mSessions) {
        if (session->hasActiveRequests()) {
            return true;
        }
    }
    return !mServer.isInPollingState();
}

#ifdef BSCP_DESKTOP_RPC_AUTH
status_t RpcServer::setDesktopAuthentication(uid_t uid, gid_t gid) {
    RpcMutexLockGuard lock(mLock);
    if (mShutdownTrigger != nullptr || !mSessions.empty()) return INVALID_OPERATION;
    if (uid==0 || uid!=getuid() || uid!=geteuid() || gid!=getgid() || gid!=getegid()) return PERMISSION_DENIED;
    mDesktopOwner=std::make_pair(uid,gid);
    return OK;
}
#endif
} // namespace android
