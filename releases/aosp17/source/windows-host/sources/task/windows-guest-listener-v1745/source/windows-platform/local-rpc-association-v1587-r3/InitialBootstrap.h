#pragma once
#include "WindowsLocalRpcBridge.h"
#ifdef __cplusplus
extern "C" {
#endif
typedef struct InitialAssociation InitialAssociation;
typedef struct InitialChannel InitialChannel;
/* Root is configured only. No business authority until each Connect succeeds.
   Same DLL major3 only. No raw binding/session/channel export.
   Root Create/Connect/Release externally serialized. Channel calls internally
   reject concurrency; Shutdown may mark terminal during a call but cannot
   cancel it. Close requires quiescent unique ownership; BUSY keeps *owner.
   All ChannelClose callers must also drain CheckLive, Generation and Shutdown
   borrowers; the operation CAS only covers Send/Receive and is not a general
   lifetime guard. CheckConfigured membership validation does not permit races
   with root Release; external root serialization remains mandatory.
   Other close failure consumes pointer into retained UNKNOWN ownership.
   Release root with live channels returns BUSY and leaves *owner intact. */
WLR_API DWORD InitialCheckConfigured(const InitialAssociation*);
WLR_API DWORD InitialCreate(const wchar_t*,HANDLE,InitialAssociation**);
WLR_API DWORD InitialConnect(InitialAssociation*,InitialChannel**);
WLR_API DWORD InitialRelease(InitialAssociation**);
WLR_API DWORD InitialChannelCheckLive(InitialChannel*);
WLR_API DWORD InitialChannelGeneration(InitialChannel*,uint64_t*);
WLR_API DWORD InitialTrySend(InitialChannel*,const WlrView*);
WLR_API DWORD InitialTryReceive(InitialChannel*,WlrRecord**);
WLR_API DWORD InitialChannelShutdown(InitialChannel*);
WLR_API DWORD InitialChannelClose(InitialChannel**);
#ifdef BSCP_WLR_TEST_HOOKS
WLR_API void InitialTestBehavior(DWORD);
WLR_API DWORD InitialTestChannelInfo(InitialChannel*,uint64_t*,uint64_t*,DWORD*);
WLR_API void InitialLastResult(DWORD*,DWORD*,DWORD*,LONG*);
#endif

#ifdef __cplusplus
}
#endif
