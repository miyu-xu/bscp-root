/*
 * Copyright (C) 2021 The Android Open Source Project
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

#define LOG_TAG "libbinder.RpcRawTransport"
#include <log/log.h>

#include <poll.h>
#include <stddef.h>
#include <sys/socket.h>

#include <binder/RpcTransportRaw.h>

#include "FdTrigger.h"
#include "OS.h"
#include "RpcState.h"
#include "RpcTransportUtils.h"
#ifdef _WIN32
#include "WindowsOverlappedPipeFd.h"
#include "WindowsPipeTransport.h"
#include "WindowsPipeFrame.h"
#endif

namespace android {
#ifdef _WIN32
using bscp::windows::socket;
using bscp::windows::socketpair;
using bscp::windows::connect;
using bscp::windows::bind;
using bscp::windows::listen;
using bscp::windows::accept4;
using bscp::windows::setsockopt;
using bscp::windows::getsockopt;
#endif


using namespace android::binder::impl;
using android::binder::borrowed_fd;
using android::binder::unique_fd;

// RpcTransport with TLS disabled.
class RpcTransportRaw : public RpcTransport {
public:
    explicit RpcTransportRaw(android::RpcTransportFd socket) : mSocket(std::move(socket)) {}
    status_t pollRead(void) override {
        uint8_t buf;
        ssize_t ret = TEMP_FAILURE_RETRY(
                
#ifdef _WIN32
                bscp::windows::recv(mSocket.fd.get(), &buf, sizeof(buf), MSG_PEEK | MSG_DONTWAIT)
#else
                ::recv(mSocket.fd.get(), &buf, sizeof(buf), MSG_PEEK | MSG_DONTWAIT)
#endif
        );
        if (ret < 0) {
            int savedErrno = errno;
            if (savedErrno == EAGAIN || savedErrno == EWOULDBLOCK) {
                return WOULD_BLOCK;
            }

            LOG_RPC_DETAIL("RpcTransport poll(): %s", strerror(savedErrno));
            return -savedErrno;
        } else if (ret == 0) {
            return DEAD_OBJECT;
        }

        return OK;
    }

    status_t interruptableWriteFully(
            FdTrigger* fdTrigger, iovec* iovs, int niovs,
            const std::optional<SmallFunction<status_t()>>& altPoll,
            const std::vector<std::variant<unique_fd, borrowed_fd>>* ancillaryFds) override {
        bool sentFds = false;
        auto send = [&](iovec* iovs, int niovs) -> ssize_t {
            ssize_t ret = binder::os::sendMessageOnSocket(mSocket, iovs, niovs,
                                                          sentFds ? nullptr : ancillaryFds);
            sentFds |= ret > 0;
            return ret;
        };
        return interruptableReadOrWrite(mSocket, fdTrigger, iovs, niovs, send, "sendmsg", POLLOUT,
                                        altPoll);
    }

    status_t interruptableReadFully(
            FdTrigger* fdTrigger, iovec* iovs, int niovs,
            const std::optional<SmallFunction<status_t()>>& altPoll,
            std::vector<std::variant<unique_fd, borrowed_fd>>* ancillaryFds) override {
        auto recv = [&](iovec* iovs, int niovs) -> ssize_t {
            return binder::os::receiveMessageFromSocket(mSocket, iovs, niovs, ancillaryFds);
        };
        return interruptableReadOrWrite(mSocket, fdTrigger, iovs, niovs, recv, "recvmsg", POLLIN,
                                        altPoll);
    }

    bool isWaiting() override { return mSocket.isInPollingState(); }

private:
    android::RpcTransportFd mSocket;
};

// RpcTransportCtx with TLS disabled.
class RpcTransportCtxRaw : public RpcTransportCtx {
    bool mFramed=false;
public:
    explicit RpcTransportCtxRaw(bool framed=false):mFramed(framed){}
    std::unique_ptr<RpcTransport> newTransport(android::RpcTransportFd socket,
                                               FdTrigger*) const override {
#ifdef _WIN32
        if(auto pipe=bscp::windows::acquireOverlappedPipe(socket.fd.get())) {
            // The consumed registry descriptor is closed after this factory;
            // transport's lease keeps the original HANDLE and opaque slot alive.
            auto bytes=RpcTransportWindowsPipe::fromRegistered(std::move(pipe));
            if(!bytes)return nullptr;
            if(mFramed)return std::make_unique<RpcTransportWindowsPipeFrame>(std::move(bytes));
            return bytes;
        }
        if(mFramed || !bscp::windows::acquireSocket(socket.fd.get()))return nullptr;
#endif
        return std::make_unique<RpcTransportRaw>(std::move(socket));
    }
    std::vector<uint8_t> getCertificate(RpcCertificateFormat) const override { return {}; }
};

std::unique_ptr<RpcTransportCtx> RpcTransportCtxFactoryRaw::newServerCtx() const {
    return std::make_unique<RpcTransportCtxRaw>(mFramed);
}

std::unique_ptr<RpcTransportCtx> RpcTransportCtxFactoryRaw::newClientCtx() const {
    return std::make_unique<RpcTransportCtxRaw>(mFramed);
}

const char *RpcTransportCtxFactoryRaw::toCString() const {
    return mFramed?"windows-guest-framed":"raw";
}

std::unique_ptr<RpcTransportCtxFactory> RpcTransportCtxFactoryRaw::make() {
    return std::unique_ptr<RpcTransportCtxFactoryRaw>(new RpcTransportCtxFactoryRaw());
}

#ifdef _WIN32
std::unique_ptr<RpcTransportCtxFactory> RpcTransportCtxFactoryRaw::makeWindowsGuestFramed(){
    return std::unique_ptr<RpcTransportCtxFactoryRaw>(new RpcTransportCtxFactoryRaw(true));
}
#endif

} // namespace android
