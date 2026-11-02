#include "WindowsLocalRpcBridge.c"
#include "BootstrapCallback.h"
#include "InitialBootstrap.h"
enum {P_CONFIGURED,P_AWAITING,P_PROOF_READY,P_PUBLISHED,P_UNKNOWN,P_CLOSING};
struct InitialAssociation {RPC_BINDING_HANDLE binding;void* session;uint64_t generation;Identity identity;PSECURITY_DESCRIPTOR sd;RPC_WSTR bindingText;LPWSTR sidText;HANDLE self,launch;volatile LONG refs,uncertain;struct InitialAssociation* next;struct InitialAssociation* activeNext;};
struct InitialChannel {InitialAssociation* root;volatile LONG operation;RPC_BINDING_HANDLE binding;void* session;void* channel;uint64_t generation,id;Identity identity;PSECURITY_DESCRIPTOR sd;RPC_WSTR bindingText;LPWSTR sidText;HANDLE self,launch,serviceProof,clientProof,tokenProof;BYTE nonce[32];wchar_t callback[128];DWORD state,reason,proofCount,preSend;int registered,uncertain;struct InitialChannel* next;};
static SRWLOCK pendingLock=SRWLOCK_INIT;
static InitialChannel* pending;
static LONG callbackBorrows,handshakeBusy,unknownCount;
static InitialAssociation* retainedRoots;
static InitialAssociation* activeRoots;
static InitialChannel* unknownOwners;
static DWORD lastReason,lastProofCount,lastPreSend;
#ifdef BSCP_WLR_TEST_HOOKS
static DWORD testBehavior;
static LONG testAssociationCount;
#else
#define testBehavior 0u
#endif
#ifdef BSCP_WLR_TEST_HOOKS
void InitialTestBehavior(DWORD mode){testBehavior=mode;}
#endif

static DWORD releaseHandle(HANDLE* h){DWORD e;if(!*h)return 0;if(!CloseHandle(*h)){e=GetLastError();return e;}*h=NULL;return 0;}
static DWORD duplicateProcess(HANDLE source,HANDLE* out){*out=NULL;return DuplicateHandle(GetCurrentProcess(),source,GetCurrentProcess(),out,0x101000,FALSE,0)?0:GetLastError();}
static DWORD getIdentity(HANDLE process,Identity* id){HANDLE t=NULL;DWORD e,closed;if(!OpenProcessToken(process,TOKEN_QUERY,&t))return GetLastError();e=identity(t,id);closed=closeProofToken(t);return closed?closed:e;}
/* Call only on the actual callback RPC thread. Inputs remain stub-owned. */
static DWORD actualSender(HANDLE input,InitialChannel* p,HANDLE* process,HANDLE* snapshot){RPC_CALL_ATTRIBUTES_V2_W a={0};HANDLE t=NULL,ambient=NULL;Identity id;DWORD e,flags=0,n=0;ULONG_PTR pid;TOKEN_TYPE type;SECURITY_IMPERSONATION_LEVEL level;
 *process=*snapshot=NULL;
 if(OpenThreadToken(GetCurrentThread(),TOKEN_QUERY,TRUE,&ambient)){closeProofToken(ambient);return ERROR_ACCESS_DENIED;}if(GetLastError()!=ERROR_NO_TOKEN)return ERROR_ACCESS_DENIED;
 e=duplicateProcess(input,process);if(e)return e;
 if(!GetHandleInformation(*process,&flags)||(flags&HANDLE_FLAG_INHERIT)||WaitForSingleObject(*process,0)!=WAIT_TIMEOUT)return ERROR_ACCESS_DENIED;
 a.Version=2;a.Flags=RPC_QUERY_CLIENT_PID;e=RpcServerInqCallAttributesW(NULL,&a);if(e)return e;pid=(ULONG_PTR)a.ClientPID;
 if(a.ProtocolSequence!=RPC_PROTSEQ_LRPC||a.NullSession||a.AuthenticationLevel<RPC_C_AUTHN_LEVEL_PKT_INTEGRITY||(a.CallStatus&(RPC_CALL_STATUS_CANCELLED|RPC_CALL_STATUS_DISCONNECTED))||!pid||pid>MAXDWORD||GetProcessId(*process)!=(DWORD)pid)return ERROR_ACCESS_DENIED;
 e=RpcImpersonateClient(NULL);if(e)return e;
 if(!OpenThreadToken(GetCurrentThread(),TOKEN_QUERY|TOKEN_DUPLICATE,TRUE,&t))e=GetLastError();
 if(!e)e=identity(t,&id);if(!e&&!equalIdentity(&id,&p->identity))e=ERROR_ACCESS_DENIED;
 if(!e&&!DuplicateTokenEx(t,TOKEN_QUERY,NULL,SecurityIdentification,TokenImpersonation,snapshot))e=GetLastError();
 if(!e&&(!GetTokenInformation(*snapshot,TokenType,&type,sizeof(type),&n)||type!=TokenImpersonation||!GetTokenInformation(*snapshot,TokenImpersonationLevel,&level,sizeof(level),&n)||level<SecurityIdentification))e=ERROR_ACCESS_DENIED;
 if(RpcRevertToSelf()!=RPC_S_OK)RaiseFailFastException(NULL,NULL,0);
 if(OpenThreadToken(GetCurrentThread(),TOKEN_QUERY,TRUE,&ambient)){closeProofToken(ambient);RaiseFailFastException(NULL,NULL,0);}if(GetLastError()!=ERROR_NO_TOKEN)RaiseFailFastException(NULL,NULL,0);
 {DWORD closed=closeProofToken(t);if(closed)e=closed;}
 if(!e&&(WaitForSingleObject(*process,0)!=WAIT_TIMEOUT||WaitForSingleObject(p->launch,0)!=WAIT_TIMEOUT||!compareObjects()||!compareObjects()(*process,p->launch)))e=ERROR_ACCESS_DENIED;
 return e;
}
unsigned long CbServer_Offer(handle_t binding,HANDLE sender,HANDLE actualClient,BYTE nonce[32],__int64 generation,__int64 id,ULONG protocol){InitialChannel* p;HANDLE process=NULL,client=NULL,token=NULL;DWORD e=0,reason=0;int accepted=0;(void)binding;
 AcquireSRWLockExclusive(&pendingLock);p=pending;if(!p||p->state!=P_AWAITING){ReleaseSRWLockExclusive(&pendingLock);return ERROR_ACCESS_DENIED;}callbackBorrows++;ReleaseSRWLockExclusive(&pendingLock);
 e=actualSender(sender,p,&process,&token);if(e){reason=1;goto done;}
 e=duplicateProcess(actualClient,&client);if(e){reason=2;goto done;}
 if(WaitForSingleObject(client,0)!=WAIT_TIMEOUT||!compareObjects()(client,p->self)){e=ERROR_ACCESS_DENIED;reason=2;goto done;}
 if(protocol!=1||memcmp(nonce,p->nonce,32)||p->generation!=(uint64_t)generation||p->id!=(uint64_t)id){e=ERROR_ACCESS_DENIED;reason=3;goto done;}
 AcquireSRWLockExclusive(&pendingLock);
 if(pending!=p||p->state!=P_AWAITING){e=ERROR_ACCESS_DENIED;reason=4;}
 else if(WaitForSingleObject(process,0)!=WAIT_TIMEOUT||WaitForSingleObject(p->launch,0)!=WAIT_TIMEOUT||WaitForSingleObject(client,0)!=WAIT_TIMEOUT){e=ERROR_ACCESS_DENIED;reason=1;}
 else{p->serviceProof=process;process=NULL;p->clientProof=client;client=NULL;p->tokenProof=token;token=NULL;p->state=P_PROOF_READY;p->proofCount++;accepted=1;}
 ReleaseSRWLockExclusive(&pendingLock);
done:
 /* Known receiver duplicates only; never close NDR input handles. */
 {DWORD a=closeProofToken(process),b=closeProofToken(client),c=closeProofToken(token);if(a||b||c){e=ERROR_GEN_FAILURE;reason=5;}}
 AcquireSRWLockExclusive(&pendingLock);if(reason)p->reason=reason;callbackBorrows--;ReleaseSRWLockExclusive(&pendingLock);
 
#ifdef BSCP_WLR_TEST_HOOKS
printf("CALLBACK status=%lu reason=%lu accepted=%d\n",e,reason,accepted);fflush(stdout);
#else
(void)accepted;
#endif
return e;
}
static INIT_ONCE callbackEndpointOnce=INIT_ONCE_STATIC_INIT;
static wchar_t processCallbackEndpoint[128];
static DWORD callbackEndpointError;
static BOOL CALLBACK createCallbackEndpoint(PINIT_ONCE once,PVOID param,PVOID* context){DWORD e;BYTE random[16];wchar_t hex[33];unsigned i;(void)once;(void)param;(void)context;
 if(BCryptGenRandom(NULL,random,sizeof(random),BCRYPT_USE_SYSTEM_PREFERRED_RNG)<0){callbackEndpointError=ERROR_GEN_FAILURE;return TRUE;}
 for(i=0;i<16;i++)swprintf_s(hex+2*i,33-2*i,L"%02x",random[i]);
 swprintf_s(processCallbackEndpoint,128,L"bscp-initial-channel-%lu-%s",GetCurrentProcessId(),hex);
 e=ensureAuth();if(!e)e=RpcServerUseProtseqEpW((RPC_WSTR)L"ncalrpc",8,(RPC_WSTR)processCallbackEndpoint,NULL);
 callbackEndpointError=e;return TRUE;
}
static DWORD startCallback(InitialChannel* p){RPC_STATUS e;
 if(BCryptGenRandom(NULL,p->nonce,32,BCRYPT_USE_SYSTEM_PREFERRED_RNG)<0)return ERROR_GEN_FAILURE;
 if(!InitOnceExecuteOnce(&callbackEndpointOnce,createCallbackEndpoint,NULL,NULL))return GetLastError();if(callbackEndpointError)return callbackEndpointError;
 wcscpy_s(p->callback,128,processCallbackEndpoint);
 AcquireSRWLockExclusive(&pendingLock);if(pending){ReleaseSRWLockExclusive(&pendingLock);return ERROR_BUSY;}pending=p;p->state=P_AWAITING;ReleaseSRWLockExclusive(&pendingLock);
 e=RpcServerRegisterIf3(CbServer_BootstrapCallback_v1_0_s_ifspec,NULL,NULL,RPC_IF_AUTOLISTEN|RPC_IF_ALLOW_LOCAL_ONLY|RPC_IF_SEC_NO_CACHE,8,0,security,NULL);
 if(e){AcquireSRWLockExclusive(&pendingLock);pending=NULL;ReleaseSRWLockExclusive(&pendingLock);return e;}p->registered=1;return 0;
}
static DWORD retireCallback(InitialChannel* p){DWORD e;if(!p->registered)return 0;
 AcquireSRWLockExclusive(&pendingLock);p->state=P_CLOSING;ReleaseSRWLockExclusive(&pendingLock);
 /* Never hold pendingLock while waiting for callback calls to drain. */
 e=RpcServerUnregisterIf(CbServer_BootstrapCallback_v1_0_s_ifspec,NULL,TRUE);if(e)return e;
 AcquireSRWLockExclusive(&pendingLock);if(callbackBorrows){ReleaseSRWLockExclusive(&pendingLock);return ERROR_BUSY;}if(pending==p)pending=NULL;p->registered=0;ReleaseSRWLockExclusive(&pendingLock);return 0;
}
static DWORD quarantine(InitialChannel* p,DWORD error){p->uncertain=1;if(p->root)InterlockedExchange(&p->root->uncertain,1);AcquireSRWLockExclusive(&pendingLock);p->next=unknownOwners;unknownOwners=p;InterlockedIncrement(&unknownCount);ReleaseSRWLockExclusive(&pendingLock);return error?error:WLR_RPC_AMBIGUOUS;}
static DWORD destroyInitial(InitialChannel* p){DWORD e;e=retireCallback(p);if(e)return quarantine(p,e);if(p->uncertain)return quarantine(p,WLR_RPC_AMBIGUOUS);
 if(p->channel){e=WlrCloseChannel(&p->channel);if(e)return quarantine(p,e);}
 e=releaseHandle(&p->serviceProof);if(e)return quarantine(p,e);e=releaseHandle(&p->clientProof);if(e)return quarantine(p,e);e=releaseHandle(&p->tokenProof);if(e)return quarantine(p,e);e=releaseHandle(&p->self);if(e)return quarantine(p,e);e=releaseHandle(&p->launch);if(e)return quarantine(p,e);
 if(p->root)InterlockedDecrement(&p->root->refs);free(p);return 0;
}

static DWORD associateRaw(void* channel,uint64_t generation,const wchar_t* ep,const BYTE* nonce){CALL_RPC(Client_AssociateInitial(channel,(__int64)generation,(wchar_t*)ep,(BYTE*)nonce),0);}
typedef struct CallbackResources {RPC_WSTR text;RPC_BINDING_HANDLE binding;struct CallbackResources* next;} CallbackResources;
static CallbackResources* retainedCallbacks;
static DWORD callbackToClient(const wchar_t* endpoint,HANDLE self,HANDLE actualClient,BYTE nonce[32],uint64_t generation,uint64_t id){CallbackResources* r=(CallbackResources*)calloc(1,sizeof(*r));RPC_SECURITY_QOS q={0};DWORD e=0,exception=0;if(!r)return ERROR_NOT_ENOUGH_MEMORY;
 e=RpcStringBindingComposeW(NULL,(RPC_WSTR)L"ncalrpc",NULL,(RPC_WSTR)endpoint,NULL,&r->text);if(e)goto done;e=RpcBindingFromStringBindingW(r->text,&r->binding);if(e)goto done;e=RpcStringFreeW(&r->text);if(e)goto done;
 q.Version=1;q.IdentityTracking=RPC_C_QOS_IDENTITY_DYNAMIC;q.ImpersonationType=RPC_C_IMP_LEVEL_IDENTIFY;e=RpcBindingSetAuthInfoExW(r->binding,NULL,RPC_C_AUTHN_LEVEL_PKT_PRIVACY,RPC_C_AUTHN_WINNT,NULL,RPC_C_AUTHZ_NONE,&q);if(e)goto done;e=RpcBindingSetOption(r->binding,RPC_C_OPT_CALL_TIMEOUT,1200);if(e)goto done;
 RpcTryExcept{e=CbClient_Offer(r->binding,self,actualClient,nonce,(__int64)generation,(__int64)id,1);
#ifdef BSCP_WLR_TEST_HOOKS
if(!e&&testBehavior==3){DWORD second=CbClient_Offer(r->binding,self,actualClient,nonce,(__int64)generation,(__int64)id,1);printf("DUPLICATE_CALLBACK=%lu\n",second);if(!second)e=ERROR_INVALID_DATA;}
#endif
}RpcExcept(1){exception=RpcExceptionCode();e=WLR_RPC_AMBIGUOUS;}RpcEndExcept
 
#ifdef BSCP_WLR_TEST_HOOKS
printf("SERVICE_CALLBACK status=%lu exception=%lu\n",e,exception);fflush(stdout);
#else
(void)exception;
#endif

done:{DWORD cleanup=0;if(r->text)cleanup=RpcStringFreeW(&r->text);if(r->binding){DWORD freed=RpcBindingFree(&r->binding);if(freed)cleanup=freed;}if(cleanup){AcquireSRWLockExclusive(&pendingLock);r->next=retainedCallbacks;retainedCallbacks=r;InterlockedIncrement(&closeFailures);ReleaseSRWLockExclusive(&pendingLock);return cleanup;}free(r);return e;}
}
unsigned long Server_AssociateInitial(WLR_CHANNEL raw,__int64 generation,wchar_t* endpoint,BYTE nonce[32]){Channel* c=(Channel*)raw;HANDLE token=NULL,self=NULL,actualClient=NULL;Identity actual;DWORD e,closed;uint64_t g=0,id=0;BYTE challenge[32];int held=0;
 if(!c||c->tag!=CHANNEL_TAG||!endpoint||wcslen(endpoint)>127)return ERROR_INVALID_PARAMETER;
 e=capture(c->session,NULL,&token,&actual,NULL);closed=closeProofToken(token);if(closed)return closed;if(e)return e;
#ifdef BSCP_WLR_TEST_HOOKS
 if(testBehavior==6){HANDLE rd=NULL,wr=NULL;WlrView view={0};BYTE value=9;DWORD pushed,a,b;
  if(!CreatePipe(&rd,&wr,NULL,0))return GetLastError();view.sequence=1;view.data=&value;view.bytes=1;view.pipes=&rd;view.pipeCount=1;
  pushed=pushChannel(c,&view);a=closeProofToken(rd);b=closeProofToken(wr);printf("PROVISIONAL_SERVER_DESCRIPTOR=%lu\n",pushed);fflush(stdout);if(a||b)return a?a:b;if(pushed!=ERROR_ACCESS_DENIED)return ERROR_INVALID_DATA;
 }
#endif

 AcquireSRWLockExclusive(&c->lock);
 if(c->closing||c->session->closing||c->session->generation!=(uint64_t)generation||c->association!=0)e=ERROR_ACCESS_DENIED;
 else{c->association=1;InterlockedIncrement(&c->refs);held=1;g=c->session->generation;id=c->id;e=duplicateProcess(c->session->owner,&actualClient);}
 ReleaseSRWLockExclusive(&c->lock);if(e)goto done;
 e=duplicateProcess(GetCurrentProcess(),&self);if(e)goto done;memcpy(challenge,nonce,32);
#ifdef BSCP_WLR_TEST_HOOKS
if(testBehavior==1)challenge[0]^=1;if(testBehavior==2)id^=1;if(testBehavior==7&&InterlockedIncrement(&testAssociationCount)>1)challenge[0]^=1;
#endif
 /* No registry/channel/pending lock or client impersonation spans reverse RPC. */
 e=callbackToClient(endpoint,self,actualClient,challenge,g,id);
#ifdef BSCP_WLR_TEST_HOOKS
 if(!e&&testBehavior==4)e=ERROR_ACCESS_DENIED;
 if(!e&&testBehavior==5)e=RPC_S_CALL_FAILED; /* Raise only after owned resources and context references are released. */
#endif
done:
 closed=closeProofToken(self);if(closed)e=closed;closed=closeProofToken(actualClient);if(closed)e=closed;
 if(held){AcquireSRWLockExclusive(&c->lock);if(!e&&(c->closing||c->session->closing||c->association!=1))e=ERROR_OPERATION_ABORTED;c->association=e?3:2;ReleaseSRWLockExclusive(&c->lock);releaseChannel(c);}
#ifdef BSCP_WLR_TEST_HOOKS
if(testBehavior==5&&e==RPC_S_CALL_FAILED)RpcRaiseException(RPC_S_CALL_FAILED);
#endif
return e;
}

static DWORD retainRoot(InitialAssociation* r,DWORD e){InterlockedExchange(&r->uncertain,1);AcquireSRWLockExclusive(&pendingLock);r->next=retainedRoots;retainedRoots=r;InterlockedIncrement(&unknownCount);ReleaseSRWLockExclusive(&pendingLock);return e?e:WLR_RPC_AMBIGUOUS;}
static DWORD destroyRoot(InitialAssociation* r){DWORD e;if(r->uncertain)return retainRoot(r,WLR_RPC_AMBIGUOUS);
 if(r->session){e=WlrCloseSession(&r->session);if(e)return retainRoot(r,e);}if(r->binding){e=RpcBindingFree(&r->binding);if(e)return retainRoot(r,e);}
 if(r->bindingText){e=RpcStringFreeW(&r->bindingText);if(e)return retainRoot(r,e);}if(r->sidText){if(LocalFree(r->sidText))return retainRoot(r,ERROR_GEN_FAILURE);r->sidText=NULL;}if(r->sd){if(LocalFree(r->sd))return retainRoot(r,ERROR_GEN_FAILURE);r->sd=NULL;}
 e=releaseHandle(&r->self);if(e)return retainRoot(r,e);e=releaseHandle(&r->launch);if(e)return retainRoot(r,e);free(r);return 0;
}
DWORD InitialCreate(const wchar_t* endpoint,HANDLE launch,InitialAssociation** out){InitialAssociation* r;DWORD e,cleanup;Identity service;wchar_t sd[1024];RPC_SECURITY_QOS_V5_W q={0};
 if(!out)return ERROR_INVALID_PARAMETER;*out=NULL;if(!endpoint||!*endpoint||!compareObjects()||!launch||WaitForSingleObject(launch,0)!=WAIT_TIMEOUT)return ERROR_INVALID_PARAMETER;
 r=(InitialAssociation*)calloc(1,sizeof(*r));if(!r)return ERROR_NOT_ENOUGH_MEMORY;r->refs=1;
 e=duplicateProcess(launch,&r->launch);if(!e)e=duplicateProcess(GetCurrentProcess(),&r->self);if(!e)e=getIdentity(r->self,&r->identity);if(!e)e=getIdentity(r->launch,&service);if(!e&&!equalIdentity(&r->identity,&service))e=ERROR_ACCESS_DENIED;if(e)goto fail;
 if(!ConvertSidToStringSidW(r->identity.sid,&r->sidText)){e=GetLastError();goto fail;}swprintf_s(sd,1024,L"D:(A;;GA;;;%s)",r->sidText);if(LocalFree(r->sidText)){e=ERROR_GEN_FAILURE;goto fail;}r->sidText=NULL;
 if(!ConvertStringSecurityDescriptorToSecurityDescriptorW(sd,SDDL_REVISION_1,&r->sd,NULL)){e=GetLastError();goto fail;}
 e=RpcStringBindingComposeW(NULL,(RPC_WSTR)L"ncalrpc",NULL,(RPC_WSTR)endpoint,NULL,&r->bindingText);if(e)goto fail;e=RpcBindingFromStringBindingW(r->bindingText,&r->binding);if(e)goto fail;e=RpcStringFreeW(&r->bindingText);if(e)goto fail;
 q.Version=5;q.Capabilities=RPC_C_QOS_CAPABILITIES_MUTUAL_AUTH;q.IdentityTracking=RPC_C_QOS_IDENTITY_DYNAMIC;q.ImpersonationType=RPC_C_IMP_LEVEL_IDENTIFY;q.Sid=r->identity.sid;q.ServerSecurityDescriptor=r->sd;
 e=RpcBindingSetAuthInfoExW(r->binding,NULL,RPC_C_AUTHN_LEVEL_PKT_PRIVACY,RPC_C_AUTHN_WINNT,NULL,RPC_C_AUTHZ_NONE,(RPC_SECURITY_QOS*)&q);if(e)goto fail;e=RpcBindingSetOption(r->binding,RPC_C_OPT_CALL_TIMEOUT,2000);if(e)goto fail;
 e=WlrOpen(r->binding,r->self,&r->session,&r->generation);if(e)goto fail;
 if(WaitForSingleObject(r->launch,0)!=WAIT_TIMEOUT){e=ERROR_ACCESS_DENIED;goto fail;}AcquireSRWLockExclusive(&pendingLock);r->activeNext=activeRoots;activeRoots=r;ReleaseSRWLockExclusive(&pendingLock);*out=r;return 0;
fail:if(e==WLR_RPC_AMBIGUOUS)r->uncertain=1;cleanup=destroyRoot(r);return e?e:cleanup;
}
DWORD InitialCheckConfigured(const InitialAssociation* r){InitialAssociation* item;DWORD e=ERROR_INVALID_HANDLE;AcquireSRWLockShared(&pendingLock);for(item=activeRoots;item;item=item->activeNext)if(item==r){e=(item->uncertain||WaitForSingleObject(item->launch,0)!=WAIT_TIMEOUT)?ERROR_ACCESS_DENIED:0;break;}ReleaseSRWLockShared(&pendingLock);return e;}
DWORD InitialRelease(InitialAssociation** owner){InitialAssociation* r;InitialAssociation** link;if(!owner||!*owner)return ERROR_INVALID_PARAMETER;r=*owner;AcquireSRWLockExclusive(&pendingLock);for(link=&activeRoots;*link&&*link!=r;link=&(*link)->activeNext){}if(!*link){ReleaseSRWLockExclusive(&pendingLock);return ERROR_INVALID_HANDLE;}if(InterlockedCompareExchange(&r->refs,0,0)!=1){ReleaseSRWLockExclusive(&pendingLock);return ERROR_BUSY;}*link=r->activeNext;ReleaseSRWLockExclusive(&pendingLock);*owner=NULL;return destroyRoot(r);}
static void snapshotDiagnostic(InitialChannel* p){AcquireSRWLockShared(&pendingLock);lastReason=p->reason;lastProofCount=p->proofCount;lastPreSend=p->preSend;ReleaseSRWLockShared(&pendingLock);}
DWORD InitialConnect(InitialAssociation* r,InitialChannel** out){InitialChannel* p;DWORD e=0,cleanup;if(!out)return ERROR_INVALID_PARAMETER;*out=NULL;e=InitialCheckConfigured(r);if(e)return e;
 if(InterlockedCompareExchange(&handshakeBusy,1,0))return ERROR_BUSY;
 p=(InitialChannel*)calloc(1,sizeof(*p));if(!p){InterlockedExchange(&handshakeBusy,0);return ERROR_NOT_ENOUGH_MEMORY;}p->root=r;InterlockedIncrement(&r->refs);p->generation=r->generation;p->identity=r->identity;
 e=duplicateProcess(r->self,&p->self);if(!e)e=duplicateProcess(r->launch,&p->launch);if(e)goto fail;
 e=WlrConnect(r->session,r->generation,&p->channel,&p->id);if(e)goto fail;
#ifdef BSCP_WLR_TEST_HOOKS
 {HANDLE rd=NULL,wr=NULL;BYTE byte=42;DWORD a,b;if(!CreatePipe(&rd,&wr,NULL,0)){e=GetLastError();goto fail;}p->preSend=WlrSend(p->channel,p->generation,1,&byte,1,NULL,0,&rd,1);a=closeProofToken(rd);b=closeProofToken(wr);if(a||b){e=a?a:b;goto fail;}if(p->preSend!=ERROR_ACCESS_DENIED){e=ERROR_INVALID_DATA;goto fail;}}
#endif
 e=startCallback(p);if(e)goto fail;e=associateRaw(p->channel,p->generation,p->callback,p->nonce);if(e)goto fail;
 AcquireSRWLockShared(&pendingLock);if(p->state!=P_PROOF_READY||p->proofCount!=1)e=ERROR_ACCESS_DENIED;ReleaseSRWLockShared(&pendingLock);if(e)goto fail;
 snapshotDiagnostic(p);e=retireCallback(p);if(e)goto fail;
 if(WaitForSingleObject(p->launch,0)!=WAIT_TIMEOUT||WaitForSingleObject(p->serviceProof,0)!=WAIT_TIMEOUT||WaitForSingleObject(p->self,0)!=WAIT_TIMEOUT||WaitForSingleObject(p->clientProof,0)!=WAIT_TIMEOUT||!compareObjects()(p->launch,p->serviceProof)||!compareObjects()(p->self,p->clientProof)){e=ERROR_ACCESS_DENIED;goto fail;}
 InterlockedExchange((volatile LONG*)&p->state,P_PUBLISHED);*out=p;InterlockedExchange(&handshakeBusy,0);return 0;
fail:snapshotDiagnostic(p);if(e==WLR_RPC_AMBIGUOUS)p->uncertain=1;cleanup=destroyInitial(p);
 AcquireSRWLockShared(&pendingLock);if(!pending)InterlockedExchange(&handshakeBusy,0);ReleaseSRWLockShared(&pendingLock);return e?e:cleanup;
}
DWORD InitialChannelCheckLive(InitialChannel* p){if(!p||InterlockedCompareExchange((volatile LONG*)&p->state,0,0)!=P_PUBLISHED||p->root->uncertain)return ERROR_ACCESS_DENIED;
 if(WaitForSingleObject(p->launch,0)!=WAIT_TIMEOUT||WaitForSingleObject(p->serviceProof,0)!=WAIT_TIMEOUT||!compareObjects()(p->launch,p->serviceProof)){InterlockedExchange((volatile LONG*)&p->state,P_CLOSING);return ERROR_ACCESS_DENIED;}return 0;
}
DWORD InitialChannelGeneration(InitialChannel* p,uint64_t* generation){DWORD e;if(!generation)return ERROR_INVALID_PARAMETER;*generation=0;e=InitialChannelCheckLive(p);if(!e)*generation=p->generation;return e;}
static DWORD enterChannel(InitialChannel* p){DWORD e;if(!p)return ERROR_INVALID_HANDLE;if(InterlockedCompareExchange(&p->operation,1,0))return ERROR_BUSY;e=InitialChannelCheckLive(p);if(e)InterlockedExchange(&p->operation,0);return e;}
static DWORD leaveChannel(InitialChannel* p,DWORD e,int receive){if(e&&e!=ERROR_BUSY&&!(receive&&e==ERROR_NO_MORE_ITEMS)){InterlockedExchange((volatile LONG*)&p->state,P_CLOSING);if(e==WLR_RPC_AMBIGUOUS){p->uncertain=1;InterlockedExchange(&p->root->uncertain,1);}}InterlockedExchange(&p->operation,0);return e;}
DWORD InitialTrySend(InitialChannel* p,const WlrView* v){DWORD e;if(!v)return ERROR_INVALID_PARAMETER;e=enterChannel(p);if(e)return e;e=WlrSend(p->channel,p->generation,v->sequence,v->data,v->bytes,v->files,v->fileCount,v->pipes,v->pipeCount);return leaveChannel(p,e,0);}
DWORD InitialTryReceive(InitialChannel* p,WlrRecord** out){DWORD e;if(!out)return ERROR_INVALID_PARAMETER;*out=NULL;e=enterChannel(p);if(e)return e;e=WlrReceive(p->channel,p->generation,out);return leaveChannel(p,e,1);}
DWORD InitialChannelShutdown(InitialChannel* p){if(!p)return ERROR_INVALID_HANDLE;InterlockedExchange((volatile LONG*)&p->state,P_CLOSING);return 0;}
DWORD InitialChannelClose(InitialChannel** owner){InitialChannel* p;if(!owner||!*owner)return ERROR_INVALID_PARAMETER;p=*owner;if(InterlockedCompareExchange(&p->operation,1,0))return ERROR_BUSY;*owner=NULL;InterlockedExchange((volatile LONG*)&p->state,P_CLOSING);return destroyInitial(p);}
#ifdef BSCP_WLR_TEST_HOOKS
void InitialLastResult(DWORD* reason,DWORD* proofs,DWORD* pre,LONG* unknown){*reason=lastReason;*proofs=lastProofCount;*pre=lastPreSend;*unknown=unknownCount;}
DWORD InitialTestChannelInfo(InitialChannel* p,uint64_t* generation,uint64_t* id,DWORD* proofs){if(!p)return ERROR_INVALID_PARAMETER;*generation=p->generation;*id=p->id;*proofs=p->proofCount;return 0;}
#endif
