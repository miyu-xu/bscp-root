#include "WindowsBinderRpc.h"
#include <android/binder_libbinder.h>
#include <binder/RpcServer.h>
#include <binder/RpcSession.h>
#include <utils/Errors.h>
#include <new>
using namespace android;
extern "C" int32_t ARpcServer_newWindowsNdr(AIBinder* root,WlrServerAcceptor** owned,ARpcServer** out) {
    if (!out) return BAD_VALUE;
    *out = nullptr;
    if (!root || !owned || !*owned) return BAD_VALUE;
    try {
        // Perform root conversion before transferring the original acceptor.
        auto platformRoot = AIBinder_toPlatformBinder(root);
        if (!platformRoot) return BAD_VALUE;
        auto server = RpcServer::makeWindowsDesktopWithRoot(owned, platformRoot);
        if (!server) return PERMISSION_DENIED;
        auto* raw = server.get();
        raw->incStrong(raw);
        *out = reinterpret_cast<ARpcServer*>(raw);
        return OK;
    } catch (const std::bad_alloc&) { return NO_MEMORY; }
      catch (...) { return UNKNOWN_ERROR; }
}
extern "C" int32_t ARpcSession_newWindowsNdr(WlrAuthenticatedOwner** owned,ARpcSession** out) {
    if (!out) return BAD_VALUE;
    *out = nullptr;
    if (!owned || !*owned) return BAD_VALUE;
    // Do not dereference/consume the weaker initial-owner capability. Its returned
    // process object is not OS authentication of the actual RPC handler process.
    return PERMISSION_DENIED;
}
extern "C" int32_t ARpcSession_setupWindowsNdr(ARpcSession* session,AIBinder** out) {
    if (!out) return BAD_VALUE;
    *out = nullptr;
    if (!session) return BAD_VALUE;
    // Explicit admission failure, not a no-op successful setup or raw fallback.
    return PERMISSION_DENIED;
}

extern "C" int32_t ARpcServer_startWindowsNdr(ARpcServer* server) {
    if(!server)return BAD_VALUE;
    try {return reinterpret_cast<RpcServer*>(server)->startWindowsNdr();}
    catch(const std::bad_alloc&){return NO_MEMORY;}catch(...){return UNKNOWN_ERROR;}
}
extern "C" int32_t ARpcServer_waitWindowsNdrReady(ARpcServer* server,uint32_t ms,uint32_t* state,int32_t* loopStatus) {
    if(state)*state=0;if(loopStatus)*loopStatus=OK;
    if(!server||!state||!loopStatus||ms==UINT32_MAX)return BAD_VALUE;
    try{return reinterpret_cast<RpcServer*>(server)->waitWindowsNdrReady(ms,state,loopStatus);}
    catch(const std::bad_alloc&){return NO_MEMORY;}catch(...){return UNKNOWN_ERROR;}
}
extern "C" int32_t ARpcServer_shutdownJoinWindowsNdr(ARpcServer* server,uint32_t ms,uint32_t* state,int32_t* loopStatus) {
    if(state)*state=0;if(loopStatus)*loopStatus=OK;
    if(!server||!state||!loopStatus||ms==UINT32_MAX)return BAD_VALUE;
    try{return reinterpret_cast<RpcServer*>(server)->shutdownJoinWindowsNdr(ms,state,loopStatus);}
    catch(const std::bad_alloc&){return NO_MEMORY;}catch(...){return UNKNOWN_ERROR;}
}
extern "C" int32_t ARpcServer_releaseWindowsNdr(ARpcServer** owned) {
    if(!owned||!*owned)return BAD_VALUE;
    auto* raw=reinterpret_cast<RpcServer*>(*owned);
    try {status_t status=raw->windowsNdrReleaseStatus();if(status!=OK)return status;
        *owned=nullptr;raw->decStrong(raw);return OK;
    }catch(...){return UNKNOWN_ERROR;}
}

extern "C" int32_t ARpcSession_newWindowsInitial(InitialAssociation** owned,ARpcSession** out) {
    if(!out)return BAD_VALUE;*out=nullptr;if(!owned||!*owned)return BAD_VALUE;
    try {auto session=RpcSession::makeWindowsInitial(owned);if(!session)return PERMISSION_DENIED;
        auto*raw=session.get();raw->incStrong(raw);*out=reinterpret_cast<ARpcSession*>(raw);return OK;
    }catch(const std::bad_alloc&){return NO_MEMORY;}catch(...){return UNKNOWN_ERROR;}
}
extern "C" int32_t ARpcSession_setupWindowsInitial(ARpcSession* opaque,AIBinder** out) {
    if(!out)return BAD_VALUE;*out=nullptr;if(!opaque)return BAD_VALUE;
    auto* session=reinterpret_cast<RpcSession*>(opaque);
    if(!session->isWindowsInitial())return PERMISSION_DENIED;
    try {status_t status=session->setupWindowsDesktopClient();if(status!=OK)return status;
        auto root=session->getRootObject();if(!root)return DEAD_OBJECT;
        *out=AIBinder_fromPlatformBinder(root);return *out?OK:NO_MEMORY;
    }catch(const std::bad_alloc&){return NO_MEMORY;}catch(...){return UNKNOWN_ERROR;}
}
extern "C" int32_t ARpcSession_shutdownJoinWindowsInitial(ARpcSession* opaque,uint32_t ms,uint32_t* state,int32_t* setupStatus) {
    if(state)*state=0;if(setupStatus)*setupStatus=OK;
    if(!opaque||!state||!setupStatus||ms==UINT32_MAX)return BAD_VALUE;
    try{return reinterpret_cast<RpcSession*>(opaque)->shutdownJoinWindowsInitial(ms,state,setupStatus);}
    catch(const std::bad_alloc&){return NO_MEMORY;}catch(...){return UNKNOWN_ERROR;}
}
extern "C" int32_t ARpcSession_releaseWindowsInitial(ARpcSession** owned) {
    if(!owned||!*owned)return BAD_VALUE;auto*raw=reinterpret_cast<RpcSession*>(*owned);
    try {status_t s=raw->releaseWindowsInitialStatus();if(s!=OK)return s;
        *owned=nullptr;raw->decStrong(raw);return OK;
    }catch(...){return UNKNOWN_ERROR;}
}
