#pragma once
#include <stdint.h>
#ifdef _WIN32
#define BSCP_DESKTOP_RPC_EXPORT __declspec(dllexport)
#else
#define BSCP_DESKTOP_RPC_EXPORT
#endif
#ifdef __cplusplus
extern "C" {
#endif
struct AIBinder;struct ARpcServer;struct ARpcSession;
struct WlrServerAcceptor;struct WlrAuthenticatedOwner;
/* No listener/start side effect. The caller owns final WlrStop after server
 * shutdown, join and free. The acceptor transfers only on successful creation. */
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcServer_newWindowsNdr(
    struct AIBinder* root,struct WlrServerAcceptor** owned,struct ARpcServer** out);
/* Managed NDR lifecycle only. READY means join loop initialized, not authenticated.
 * Finite budgets observe progress, not a hard OS deadline. On timeout/error retain
 * owner. shutdownJoin returns operation status, with sticky loop failure separate.
 * state: 0 New, 1 Starting, 2 Ready, 3 Stopping, 4 LoopExited, 5 Joined.
 * Release requires a unique, externally quiescent owner and Joined; no concurrent
 * API borrower may remain. Caller still owns final WlrStop. */
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcServer_startWindowsNdr(struct ARpcServer* server);
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcServer_waitWindowsNdrReady(struct ARpcServer* server,
    uint32_t milliseconds,uint32_t* state,int32_t* loopStatus);
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcServer_shutdownJoinWindowsNdr(struct ARpcServer* server,
    uint32_t milliseconds,uint32_t* state,int32_t* loopStatus);
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcServer_releaseWindowsNdr(struct ARpcServer** owned);
/* Major3 configured initial association. Construction is pending, not trusted
 * admission. Every setup connection must independently pass InitialConnect.
 * Failure before construction commit leaves *owned unchanged. Setup is one-shot.
 * Client state: 0 Configured,1 SettingUp,2 Ready,3 Stopping,4 ThreadsJoined,5 Closed.
 * Close success is separate from sticky setup failure; wait timeout retains owner.
 * release requires Closed and externally unique/quiescent ownership. */
struct InitialAssociation;
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcSession_newWindowsInitial(struct InitialAssociation** owned,struct ARpcSession** out);
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcSession_setupWindowsInitial(struct ARpcSession*,struct AIBinder** out);
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcSession_shutdownJoinWindowsInitial(struct ARpcSession*,uint32_t ms,uint32_t* state,int32_t* setupStatus);
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcSession_releaseWindowsInitial(struct ARpcSession** owned);
/* Production admission is intentionally closed pending actual RPC-peer-to-launch
 * proof. Returns PERMISSION_DENIED, out=NULL and leaves owned untouched.
 * No environment/feature switch, fallback or successful unauthenticated object. */
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcSession_newWindowsNdr(
    struct WlrAuthenticatedOwner** owned,struct ARpcSession** out);
BSCP_DESKTOP_RPC_EXPORT int32_t ARpcSession_setupWindowsNdr(
    struct ARpcSession* session,struct AIBinder** out);
#ifdef __cplusplus
}
#endif
