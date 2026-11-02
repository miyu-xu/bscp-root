#pragma once
#include <windows.h>
#include <stdint.h>
#ifdef WLR_BUILD
#define WLR_API __declspec(dllexport)
#else
#define WLR_API __declspec(dllimport)
#endif
#ifdef __cplusplus
extern "C" {
#endif
/* All allocations and token/record ownership remain inside the C bridge. */
typedef struct WlrRecord WlrRecord;
#define WLR_RPC_AMBIGUOUS ((DWORD)0xe0570001u)
/* Transport exception != manager rejection. Never replay an ambiguous call.
   First exception poisons new client business calls; cleanup Close remains
   permitted, but is never automatically retried. Restart requires a new process. */
WLR_API DWORD WlrLastRpcException(void);
#ifdef BSCP_WLR_ISOLATED_COMPONENT
WLR_API void WlrTestFailNdrAllocation(LONG ordinal);
#endif
typedef struct WlrView { uint64_t sequence; const unsigned char* data; uint32_t bytes;
 const HANDLE* files; uint32_t fileCount; const HANDLE* pipes; uint32_t pipeCount;
 HANDLE requestToken; HANDLE requestProcess; DWORD requestPid; } WlrView;
WLR_API DWORD WlrStartIsolated(const wchar_t* endpoint);
WLR_API DWORD WlrStop(void);
WLR_API DWORD WlrBind(const wchar_t* endpoint,void** binding);
WLR_API DWORD WlrFreeBinding(void** binding);
WLR_API DWORD WlrOpen(void* binding,HANDLE owner,void** session,uint64_t* generation);
WLR_API DWORD WlrConnect(void* session,uint64_t generation,void** channel,uint64_t* channelId);
WLR_API DWORD WlrSend(void* channel,uint64_t generation,uint64_t sequence,const unsigned char*,uint32_t,const HANDLE*,uint32_t,const HANDLE*,uint32_t);
WLR_API DWORD WlrReceive(void* channel,uint64_t generation,WlrRecord** record);
WLR_API DWORD WlrCloseChannel(void** channel);
WLR_API DWORD WlrCloseSession(void** session);
/* Server-local APIs require exact generation/id; no wire numeric HANDLE adoption. */
/* Server acceptance uses an opaque original channel owner, never client IDs.
   OwnerProof is admission identity only, not a per-message dispatch identity.
   Hold an external shared owner for every operation. Shutdown may race Pop/Push;
   Close consumes the final unique owner only after all borrowed calls finish. */
typedef struct WlrAcceptedChannel WlrAcceptedChannel;
WLR_API DWORD WlrTryAccept(WlrAcceptedChannel**);
WLR_API DWORD WlrAcceptedOwnerProof(const WlrAcceptedChannel*,HANDLE*,HANDLE*,DWORD*,uint64_t*);
WLR_API DWORD WlrAcceptedPop(WlrAcceptedChannel*,WlrRecord**);
WLR_API DWORD WlrAcceptedPush(WlrAcceptedChannel*,const WlrView*);
WLR_API DWORD WlrAcceptedShutdown(WlrAcceptedChannel*);
WLR_API DWORD WlrAcceptedClose(WlrAcceptedChannel**);
#ifdef BSCP_WLR_ISOLATED_COMPONENT
WLR_API void WlrTestFailAcceptAllocation(void);
#endif
WLR_API DWORD WlrServerPop(uint64_t generation,uint64_t channelId,WlrRecord** record);
WLR_API DWORD WlrServerPush(uint64_t generation,uint64_t channelId,const WlrView* record);
WLR_API DWORD WlrRecordView(const WlrRecord*,WlrView*);
/* Borrowed proof only for server-pop inbound records; valid through record lifetime. */
WLR_API DWORD WlrRecordQueryRequestIdentity(const WlrRecord*,HANDLE*,HANDLE*,DWORD*);
WLR_API void WlrRecordRelease(WlrRecord*);
WLR_API void WlrCounters(LONG* sessions,LONG* channels,LONG* records,LONG* closeFailures);
#ifdef __cplusplus
}
#endif
