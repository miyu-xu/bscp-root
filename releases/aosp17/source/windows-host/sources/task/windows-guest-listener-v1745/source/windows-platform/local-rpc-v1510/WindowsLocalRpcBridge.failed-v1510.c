#ifndef BSCP_WLR_ISOLATED_COMPONENT
#error Production admission is disabled until trusted service bootstrap is integrated.
#endif
#define _WIN32_WINNT 0x0A00
#define WIN32_LEAN_AND_MEAN
#define WLR_BUILD
#include <windows.h>
#include <rpc.h>
#include <rpcasync.h>
#include <bcrypt.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "WindowsLocalRpc.h"
#include "WindowsLocalRpcBridge.h"
#define SESSION_TAG 0x73524c57u
#define CHANNEL_TAG 0x63524c57u
#define MAX_BYTES 65536u
#define MAX_HANDLES 8u
#define MAX_SESSIONS 32
#define MAX_CHANNELS 128
typedef struct Identity { unsigned char sid[SECURITY_MAX_SID_SIZE]; LUID logon; } Identity;
typedef struct Session { ULONG tag; LONG refs; volatile LONG closing; HANDLE owner; Identity identity; uint64_t generation; } Session;
typedef struct Channel { ULONG tag; LONG refs; SRWLOCK lock; Session* session; int closing; uint64_t id,nextIn,nextOut; struct WlrRecord* in; struct WlrRecord* out; struct Channel* next; } Channel;
struct WlrRecord { uint64_t sequence; unsigned char* data; ULONG bytes; HANDLE* files; ULONG fileCount; HANDLE* pipes; ULONG pipeCount; HANDLE token; };
static SRWLOCK registryLock=SRWLOCK_INIT;
static Channel* registry;
static Identity allowed;
static volatile LONG sessionCount,channelCount,recordCount,closeFailures,started;
static volatile LONG64 idCounter;
void* __RPC_USER midl_user_allocate(size_t n){return malloc(n);}
void __RPC_USER midl_user_free(void* p){free(p);}
static void closeOwned(HANDLE h){if(h&&h!=INVALID_HANDLE_VALUE&&!CloseHandle(h))InterlockedIncrement(&closeFailures);}
static DWORD identity(HANDLE token,Identity* out){unsigned char b[sizeof(TOKEN_USER)+SECURITY_MAX_SID_SIZE];TOKEN_STATISTICS st;DWORD n=0;
 if(!GetTokenInformation(token,TokenUser,b,sizeof(b),&n)||!CopySid(sizeof(out->sid),out->sid,((TOKEN_USER*)b)->User.Sid)||!GetTokenInformation(token,TokenStatistics,&st,sizeof(st),&n))return GetLastError();out->logon=st.AuthenticationId;return 0;}
static int equalIdentity(const Identity* a,const Identity* b){return EqualSid((PSID)a->sid,(PSID)b->sid)&&a->logon.LowPart==b->logon.LowPart&&a->logon.HighPart==b->logon.HighPart;}
static DWORD capture(Session* s,HANDLE supplied,HANDLE* snapshot,Identity* actual){RPC_CALL_ATTRIBUTES_V2_W a={0};HANDLE t=NULL;DWORD e=0;Identity processIdentity;HANDLE pt=NULL;HANDLE owner=s?s->owner:supplied;*snapshot=NULL;
 a.Version=2;a.Flags=RPC_QUERY_CLIENT_PID;
 if(RpcServerInqCallAttributesW(NULL,&a)!=RPC_S_OK||a.ProtocolSequence!=RPC_PROTSEQ_LRPC||a.NullSession||a.AuthenticationLevel<RPC_C_AUTHN_LEVEL_PKT_INTEGRITY)return ERROR_ACCESS_DENIED;
 if(!owner||WaitForSingleObject(owner,0)!=WAIT_TIMEOUT||GetProcessId(owner)!=(DWORD)(ULONG_PTR)a.ClientPID)return ERROR_ACCESS_DENIED;
 if(RpcImpersonateClient(NULL)!=RPC_S_OK)return ERROR_ACCESS_DENIED;
 if(!OpenThreadToken(GetCurrentThread(),TOKEN_QUERY|TOKEN_DUPLICATE,TRUE,&t))e=GetLastError();
 if(!e)e=identity(t,actual);
 if(!e&&!DuplicateTokenEx(t,TOKEN_QUERY|TOKEN_IMPERSONATE,NULL,SecurityIdentification,TokenImpersonation,snapshot))e=GetLastError();
 closeOwned(t);
 /* Revert failure cannot return this thread to the RPC pool with caller authority. */
 if(RpcRevertToSelf()!=RPC_S_OK){closeOwned(*snapshot);*snapshot=NULL;RpcRaiseException(RPC_S_ACCESS_DENIED);return ERROR_ACCESS_DENIED;}
 if(!e&&!equalIdentity(actual,&allowed))e=ERROR_ACCESS_DENIED;
 if(!e&&s&&!equalIdentity(actual,&s->identity))e=ERROR_ACCESS_DENIED;
 if(!e){if(!OpenProcessToken(owner,TOKEN_QUERY,&pt))e=GetLastError();else{e=identity(pt,&processIdentity);closeOwned(pt);if(!e&&!equalIdentity(actual,&processIdentity))e=ERROR_ACCESS_DENIED;}}
 if(!e&&WaitForSingleObject(owner,0)!=WAIT_TIMEOUT)e=ERROR_ACCESS_DENIED;
 if(e){closeOwned(*snapshot);*snapshot=NULL;}return e;
}
static RPC_STATUS RPC_ENTRY security(RPC_IF_HANDLE unused,void* call){RPC_CALL_ATTRIBUTES_V2_W a={0};(void)unused;a.Version=2;
 if(RpcServerInqCallAttributesW(call,&a)!=RPC_S_OK||a.ProtocolSequence!=RPC_PROTSEQ_LRPC||a.NullSession||a.AuthenticationLevel<RPC_C_AUTHN_LEVEL_PKT_INTEGRITY)return RPC_S_ACCESS_DENIED;return RPC_S_OK;}
static void releaseSession(Session* s){if(InterlockedDecrement(&s->refs)==0){closeOwned(s->owner);s->tag=0;free(s);InterlockedDecrement(&sessionCount);}}
void WlrRecordRelease(WlrRecord* r){ULONG i;if(!r)return;for(i=0;i<r->fileCount;i++)closeOwned(r->files[i]);for(i=0;i<r->pipeCount;i++)closeOwned(r->pipes[i]);closeOwned(r->token);free(r->files);free(r->pipes);free(r->data);free(r);InterlockedDecrement(&recordCount);}
static void releaseChannel(Channel* c){if(InterlockedDecrement(&c->refs)==0){WlrRecordRelease(c->in);WlrRecordRelease(c->out);releaseSession(c->session);c->tag=0;free(c);InterlockedDecrement(&channelCount);}}
static Channel* lookup(uint64_t g,uint64_t id){Channel* c;AcquireSRWLockShared(&registryLock);for(c=registry;c;c=c->next)if(c->id==id&&c->session->generation==g){InterlockedIncrement(&c->refs);break;}ReleaseSRWLockShared(&registryLock);return c;}
static DWORD copyHandle(HANDLE source,HANDLE* out,int pipe){DWORD kind=GetFileType(source);*out=NULL;if(pipe){if(kind!=FILE_TYPE_PIPE||!GetNamedPipeInfo(source,NULL,NULL,NULL,NULL))return ERROR_INVALID_HANDLE;}else if(kind!=FILE_TYPE_DISK)return ERROR_INVALID_HANDLE;
 if(!DuplicateHandle(GetCurrentProcess(),source,GetCurrentProcess(),out,0,FALSE,DUPLICATE_SAME_ACCESS))return GetLastError();return 0;}
static DWORD makeRecord(uint64_t sequence,const unsigned char* data,ULONG bytes,const HANDLE* files,ULONG nf,const HANDLE* pipes,ULONG np,HANDLE token,WlrRecord** out){WlrRecord* r;ULONG i;DWORD e=0;*out=NULL;
 if(bytes>MAX_BYTES||nf>MAX_HANDLES||np>MAX_HANDLES||nf+np>MAX_HANDLES||(!data&&bytes)||(!files&&nf)||(!pipes&&np))return ERROR_INVALID_PARAMETER;
 r=(WlrRecord*)calloc(1,sizeof(*r));if(!r)return ERROR_NOT_ENOUGH_MEMORY;InterlockedIncrement(&recordCount);r->sequence=sequence;r->bytes=bytes;
 if(bytes){r->data=(unsigned char*)malloc(bytes);if(!r->data)e=ERROR_NOT_ENOUGH_MEMORY;else memcpy(r->data,data,bytes);}
 if(!e&&nf){r->files=(HANDLE*)calloc(nf,sizeof(HANDLE));if(!r->files)e=ERROR_NOT_ENOUGH_MEMORY;}
 if(!e&&np){r->pipes=(HANDLE*)calloc(np,sizeof(HANDLE));if(!r->pipes)e=ERROR_NOT_ENOUGH_MEMORY;}
 for(i=0;!e&&i<nf;i++){e=copyHandle(files[i],&r->files[i],0);if(!e)r->fileCount++;}
 for(i=0;!e&&i<np;i++){e=copyHandle(pipes[i],&r->pipes[i],1);if(!e)r->pipeCount++;}
 if(!e&&token&&!DuplicateHandle(GetCurrentProcess(),token,GetCurrentProcess(),&r->token,0,FALSE,DUPLICATE_SAME_ACCESS))e=GetLastError();
 if(e){WlrRecordRelease(r);return e;}*out=r;return 0;
}
DWORD WlrRecordView(const WlrRecord* r,WlrView* v){if(!r||!v)return ERROR_INVALID_PARAMETER;v->sequence=r->sequence;v->data=r->data;v->bytes=r->bytes;v->files=r->files;v->fileCount=r->fileCount;v->pipes=r->pipes;v->pipeCount=r->pipeCount;v->requestToken=r->token;return 0;}
void __RPC_USER WLR_CHANNEL_rundown(WLR_CHANNEL raw){Channel* c=(Channel*)raw;Channel** p;WlrRecord* in;WlrRecord* out;if(!c||c->tag!=CHANNEL_TAG)return;
 AcquireSRWLockExclusive(&registryLock);for(p=&registry;*p&&*p!=c;p=&(*p)->next){}if(*p)*p=c->next;ReleaseSRWLockExclusive(&registryLock);
 AcquireSRWLockExclusive(&c->lock);c->closing=1;in=c->in;out=c->out;c->in=c->out=NULL;ReleaseSRWLockExclusive(&c->lock);WlrRecordRelease(in);WlrRecordRelease(out);releaseChannel(c);
}
void __RPC_USER WLR_SESSION_rundown(WLR_SESSION raw){Session* s=(Session*)raw;Channel* c;if(!s||s->tag!=SESSION_TAG)return;InterlockedExchange(&s->closing,1);
 AcquireSRWLockShared(&registryLock);for(c=registry;c;c=c->next)if(c->session==s){WlrRecord* in;WlrRecord* out;AcquireSRWLockExclusive(&c->lock);c->closing=1;in=c->in;out=c->out;c->in=c->out=NULL;ReleaseSRWLockExclusive(&c->lock);WlrRecordRelease(in);WlrRecordRelease(out);}ReleaseSRWLockShared(&registryLock);releaseSession(s);
}
unsigned long Server_OpenSession(handle_t binding,HANDLE owner,WLR_SESSION* out,__int64* generation){Session* s;HANDLE token=NULL;Identity id;DWORD e;(void)binding;*out=NULL;*generation=0;e=capture(NULL,owner,&token,&id);closeOwned(token);if(e)return e;
 if(InterlockedIncrement(&sessionCount)>MAX_SESSIONS){InterlockedDecrement(&sessionCount);return ERROR_BUSY;}s=(Session*)calloc(1,sizeof(*s));if(!s){InterlockedDecrement(&sessionCount);return ERROR_NOT_ENOUGH_MEMORY;}s->tag=SESSION_TAG;s->refs=1;s->identity=id;
 if(!DuplicateHandle(GetCurrentProcess(),owner,GetCurrentProcess(),&s->owner,PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,FALSE,0)){e=GetLastError();releaseSession(s);return e;}
 if(BCryptGenRandom(NULL,(PUCHAR)&s->generation,sizeof(s->generation),BCRYPT_USE_SYSTEM_PREFERRED_RNG)<0||!s->generation){releaseSession(s);return ERROR_GEN_FAILURE;}*out=s;*generation=(__int64)s->generation;return 0;
}
unsigned long Server_OpenChannel(WLR_SESSION raw,__int64 generation,WLR_CHANNEL* out,__int64* id){Session* s=(Session*)raw;Channel* c;HANDLE token=NULL;Identity actual;DWORD e;*out=NULL;*id=0;if(!s||s->tag!=SESSION_TAG)return ERROR_INVALID_HANDLE;e=capture(s,NULL,&token,&actual);closeOwned(token);if(e)return e;if(s->closing||s->generation!=(uint64_t)generation)return ERROR_ACCESS_DENIED;
 if(InterlockedIncrement(&channelCount)>MAX_CHANNELS){InterlockedDecrement(&channelCount);return ERROR_BUSY;}c=(Channel*)calloc(1,sizeof(*c));if(!c){InterlockedDecrement(&channelCount);return ERROR_NOT_ENOUGH_MEMORY;}c->tag=CHANNEL_TAG;c->refs=1;c->session=s;InterlockedIncrement(&s->refs);c->id=(uint64_t)InterlockedIncrement64(&idCounter);c->nextIn=c->nextOut=1;
 AcquireSRWLockExclusive(&registryLock);c->next=registry;registry=c;ReleaseSRWLockExclusive(&registryLock);*out=c;*id=(__int64)c->id;return 0;
}
unsigned long Server_TrySend(WLR_CHANNEL raw,__int64 generation,__int64 sequence,ULONG bytes,unsigned char* data,ULONG nf,HANDLE* files,ULONG np,HANDLE* pipes){Channel* c=(Channel*)raw;HANDLE token=NULL;Identity actual;WlrRecord* r=NULL;DWORD e;if(!c||c->tag!=CHANNEL_TAG)return ERROR_INVALID_HANDLE;e=capture(c->session,NULL,&token,&actual);if(e)return e;
 AcquireSRWLockExclusive(&c->lock);if(c->closing||c->session->closing)e=ERROR_OPERATION_ABORTED;else if(c->session->generation!=(uint64_t)generation)e=ERROR_ACCESS_DENIED;else if((uint64_t)sequence!=c->nextIn)e=ERROR_INVALID_DATA;else if(c->in)e=ERROR_BUSY;else{e=makeRecord((uint64_t)sequence,data,bytes,files,nf,pipes,np,token,&r);if(!e){c->in=r;c->nextIn++;}}ReleaseSRWLockExclusive(&c->lock);closeOwned(token);return e;
}
unsigned long Server_TryReceive(WLR_CHANNEL raw,__int64 generation,__int64* sequence,ULONG* bytes,unsigned char** data,ULONG* nf,HANDLE** files,ULONG* np,HANDLE** pipes){Channel* c=(Channel*)raw;HANDLE token=NULL;Identity actual;WlrRecord* r=NULL;DWORD e;*sequence=0;*bytes=*nf=*np=0;*data=NULL;*files=*pipes=NULL;if(!c||c->tag!=CHANNEL_TAG)return ERROR_INVALID_HANDLE;e=capture(c->session,NULL,&token,&actual);closeOwned(token);if(e)return e;
 AcquireSRWLockExclusive(&c->lock);if(c->closing||c->session->closing)e=ERROR_OPERATION_ABORTED;else if(c->session->generation!=(uint64_t)generation)e=ERROR_ACCESS_DENIED;else if(!c->out)e=ERROR_NO_MORE_ITEMS;else{r=c->out;c->out=NULL;e=0;}ReleaseSRWLockExclusive(&c->lock);if(e)return e;
 *sequence=(__int64)r->sequence;*bytes=r->bytes;*data=r->data;*nf=r->fileCount;*files=r->files;*np=r->pipeCount;*pipes=r->pipes;r->data=NULL;r->files=r->pipes=NULL;r->fileCount=r->pipeCount=0;WlrRecordRelease(r);return 0;
}
unsigned long Server_CloseChannel(WLR_CHANNEL* raw){Channel* c=(Channel*)*raw;HANDLE token=NULL;Identity actual;DWORD e;if(!c||c->tag!=CHANNEL_TAG)return ERROR_INVALID_HANDLE;e=capture(c->session,NULL,&token,&actual);closeOwned(token);if(e)return e;*raw=NULL;WLR_CHANNEL_rundown(c);return 0;}
unsigned long Server_CloseSession(WLR_SESSION* raw){Session* s=(Session*)*raw;HANDLE token=NULL;Identity actual;DWORD e;if(!s||s->tag!=SESSION_TAG)return ERROR_INVALID_HANDLE;e=capture(s,NULL,&token,&actual);closeOwned(token);if(e)return e;*raw=NULL;WLR_SESSION_rundown(s);return 0;}
DWORD WlrServerPop(uint64_t generation,uint64_t id,WlrRecord** out){Channel* c;DWORD e=0;*out=NULL;c=lookup(generation,id);if(!c)return ERROR_INVALID_HANDLE;AcquireSRWLockExclusive(&c->lock);if(c->closing||c->session->closing)e=ERROR_OPERATION_ABORTED;else if(!c->in)e=ERROR_NO_MORE_ITEMS;else{*out=c->in;c->in=NULL;}ReleaseSRWLockExclusive(&c->lock);releaseChannel(c);return e;}
DWORD WlrServerPush(uint64_t generation,uint64_t id,const WlrView* v){Channel* c;WlrRecord* r=NULL;DWORD e;if(!v)return ERROR_INVALID_PARAMETER;c=lookup(generation,id);if(!c)return ERROR_INVALID_HANDLE;AcquireSRWLockExclusive(&c->lock);if(c->closing||c->session->closing)e=ERROR_OPERATION_ABORTED;else if(c->out)e=ERROR_BUSY;else if(v->sequence!=c->nextOut)e=ERROR_INVALID_DATA;else{e=makeRecord(v->sequence,v->data,v->bytes,v->files,v->fileCount,v->pipes,v->pipeCount,NULL,&r);if(!e){c->out=r;c->nextOut++;}}ReleaseSRWLockExclusive(&c->lock);releaseChannel(c);return e;}
void WlrCounters(LONG* s,LONG* c,LONG* r,LONG* f){*s=sessionCount;*c=channelCount;*r=recordCount;*f=closeFailures;}
DWORD WlrStartIsolated(const wchar_t* endpoint){HANDLE token=NULL;DWORD e;RPC_STATUS r;if(!endpoint||InterlockedCompareExchange(&started,1,0))return ERROR_INVALID_PARAMETER;if(!OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&token))return GetLastError();e=identity(token,&allowed);closeOwned(token);if(e)return e;
 r=RpcServerUseProtseqEpW((RPC_WSTR)L"ncalrpc",16,(RPC_WSTR)endpoint,NULL);if(r)return r;r=RpcServerRegisterAuthInfoW(NULL,RPC_C_AUTHN_WINNT,NULL,NULL);if(r)return r;r=RpcServerRegisterIf3(Server_WindowsLocalRpc_v1_0_s_ifspec,NULL,NULL,RPC_IF_ALLOW_LOCAL_ONLY|RPC_IF_SEC_NO_CACHE,16,0,security,NULL);if(r)return r;return RpcServerListen(1,16,TRUE);}
DWORD WlrStop(void){RPC_STATUS a=RpcMgmtStopServerListening(NULL),b=RpcMgmtWaitServerListen(),c=RpcServerUnregisterIf(Server_WindowsLocalRpc_v1_0_s_ifspec,NULL,TRUE);return a?a:b?b:c?c:(sessionCount||channelCount||recordCount||closeFailures)?ERROR_BUSY:0;}
DWORD WlrBind(const wchar_t* endpoint,void** out){RPC_WSTR text=NULL;RPC_STATUS r;RPC_SECURITY_QOS q={0};*out=NULL;r=RpcStringBindingComposeW(NULL,(RPC_WSTR)L"ncalrpc",NULL,(RPC_WSTR)endpoint,(RPC_WSTR)L"Security=identification dynamic true",&text);if(r)return r;r=RpcBindingFromStringBindingW(text,out);RpcStringFreeW(&text);if(r)return r;q.Version=1;q.IdentityTracking=RPC_C_QOS_IDENTITY_DYNAMIC;q.ImpersonationType=RPC_C_IMP_LEVEL_IDENTIFY;r=RpcBindingSetAuthInfoExW(*out,NULL,RPC_C_AUTHN_LEVEL_PKT_PRIVACY,RPC_C_AUTHN_WINNT,NULL,RPC_C_AUTHZ_NONE,&q);if(r)RpcBindingFree(out);return r;}
DWORD WlrFreeBinding(void** b){return RpcBindingFree(b);}
#define CALL_RPC(expr) DWORD e=0;RpcTryExcept{e=(expr);}RpcExcept(1){e=RpcExceptionCode();}RpcEndExcept return e
DWORD WlrOpen(void* b,HANDLE p,void** s,uint64_t* g){CALL_RPC(Client_OpenSession(b,p,s,(__int64*)g));}
DWORD WlrConnect(void* s,uint64_t g,void** c,uint64_t* id){CALL_RPC(Client_OpenChannel(s,(__int64)g,c,(__int64*)id));}
DWORD WlrSend(void* c,uint64_t g,uint64_t seq,const unsigned char* d,uint32_t n,const HANDLE* f,uint32_t nf,const HANDLE* p,uint32_t np){CALL_RPC(Client_TrySend(c,(__int64)g,(__int64)seq,n,(unsigned char*)d,nf,(HANDLE*)f,np,(HANDLE*)p));}
DWORD WlrCloseChannel(void** c){CALL_RPC(Client_CloseChannel(c));}
DWORD WlrCloseSession(void** s){CALL_RPC(Client_CloseSession(s));}
DWORD WlrReceive(void* c,uint64_t g,WlrRecord** out){WlrRecord* r=(WlrRecord*)calloc(1,sizeof(*r));DWORD e=0;*out=NULL;if(!r)return ERROR_NOT_ENOUGH_MEMORY;InterlockedIncrement(&recordCount);
 RpcTryExcept{e=Client_TryReceive(c,(__int64)g,(__int64*)&r->sequence,&r->bytes,&r->data,&r->fileCount,&r->files,&r->pipeCount,&r->pipes);}RpcExcept(1){e=RpcExceptionCode();}RpcEndExcept
 if(e){WlrRecordRelease(r);return e;}*out=r;return 0;
}
