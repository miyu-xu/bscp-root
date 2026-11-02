#include "WindowsBinderRpc.h"
#include "WindowsLocalRpcBridge.h"
#include <android/binder_libbinder.h>
#include <android/binder_ibinder.h>
#include <binder/Binder.h>
#include <binder/RpcServer.h>
#include <binder_rpc_unstable.hpp>
#include <cstdio>
#include <thread>
#include <new>
using namespace android;
#define REQUIRE(x) do { if(!(x)){printf("FAIL line=%d winerr=%lu\n",__LINE__,GetLastError());return 1;} }while(0)
int wmain(int argc,wchar_t**argv) {
 std::thread([]{Sleep(20000);fprintf(stderr,"TEST_SELF_DEADLINE_FAIL\n");fflush(nullptr);ExitProcess(90);}).detach();
 const std::wstring mode=argc>1?argv[1]:L"normal";
 uint32_t state=99;int32_t loop=99;
 REQUIRE(ARpcServer_waitWindowsNdrReady(nullptr,0,&state,&loop)==BAD_VALUE&&state==0&&loop==0);
 auto raw=RpcServer::make();REQUIRE(raw->startWindowsNdr()==INVALID_OPERATION);raw.clear();
 std::wstring endpoint=L"managed-loop-"+std::to_wstring(GetCurrentProcessId());
 WlrServerAcceptor* cap=nullptr;REQUIRE(!WlrStartOwnedListener(endpoint.c_str(),&cap));
 auto binder=sp<BBinder>::make();AIBinder* root=AIBinder_fromPlatformBinder(binder);REQUIRE(root);
 ARpcServer* server=nullptr;REQUIRE(!ARpcServer_newWindowsNdr(root,&cap,&server)&&server&&!cap);AIBinder_decStrong(root);
 REQUIRE(ARpcServer_waitWindowsNdrReady(server,5,&state,&loop)==TIMED_OUT&&state==0&&loop==0);
 REQUIRE(ARpcServer_releaseWindowsNdr(&server)==INVALID_OPERATION&&server);
 if(mode==L"before") {
  REQUIRE(!ARpcServer_shutdownJoinWindowsNdr(server,0,&state,&loop)&&state==5&&loop==0);
  REQUIRE(ARpcServer_startWindowsNdr(server)==INVALID_OPERATION);
 } else {
#ifdef BSCP_NDR_LIFECYCLE_TESTING
  HANDLE entered=nullptr,proceed=nullptr;
  if(mode==L"early") {
   entered=CreateEventW(nullptr,TRUE,FALSE,nullptr);proceed=CreateEventW(nullptr,TRUE,FALSE,nullptr);REQUIRE(entered&&proceed);
   REQUIRE(!reinterpret_cast<RpcServer*>(server)->setWindowsNdrTestHook([=]{SetEvent(entered);if(WaitForSingleObject(proceed,5000)!=WAIT_OBJECT_0)throw std::bad_alloc();}));
  }else if(mode==L"throw")REQUIRE(!reinterpret_cast<RpcServer*>(server)->setWindowsNdrTestHook([]{throw std::bad_alloc();}));
#endif
  REQUIRE(!ARpcServer_startWindowsNdr(server));
#ifdef BSCP_NDR_LIFECYCLE_TESTING
  if(mode==L"early") {
   REQUIRE(WaitForSingleObject(entered,3000)==WAIT_OBJECT_0);
   REQUIRE(ARpcServer_waitWindowsNdrReady(server,5,&state,&loop)==TIMED_OUT&&state==1);
   REQUIRE(ARpcServer_shutdownJoinWindowsNdr(server,5,&state,&loop)==TIMED_OUT&&state==3);
   auto original=server;ARpcServer_free(server);REQUIRE(server==original);
   REQUIRE(ARpcServer_releaseWindowsNdr(&server)==INVALID_OPERATION&&server==original);
   REQUIRE(ARpcServer_waitWindowsNdrReady(server,0,&state,&loop)==DEAD_OBJECT&&state==3);
   REQUIRE(SetEvent(proceed));
  }else if(mode==L"throw") {
   REQUIRE(ARpcServer_waitWindowsNdrReady(server,3000,&state,&loop)==NO_MEMORY&&state==4&&loop==NO_MEMORY);
   REQUIRE(ARpcServer_releaseWindowsNdr(&server)==INVALID_OPERATION&&server);
  }else
#endif
  { REQUIRE(!ARpcServer_waitWindowsNdrReady(server,3000,&state,&loop)&&state==2&&loop==0); }
  REQUIRE(ARpcServer_startWindowsNdr(server)==INVALID_OPERATION);
  REQUIRE(!ARpcServer_shutdown(server)); // Actual legacy mixed-mode refusal.
  REQUIRE(!ARpcServer_shutdownJoinWindowsNdr(server,3000,&state,&loop)&&state==5);
  REQUIRE(loop==(mode==L"throw"?NO_MEMORY:OK));
#ifdef BSCP_NDR_LIFECYCLE_TESTING
  if(entered)REQUIRE(CloseHandle(entered));if(proceed)REQUIRE(CloseHandle(proceed));
#endif
 }
 REQUIRE(!ARpcServer_shutdownJoinWindowsNdr(server,0,&state,&loop)&&state==5);
 REQUIRE(ARpcServer_waitWindowsNdrReady(server,0,&state,&loop)==(mode==L"throw"?NO_MEMORY:DEAD_OBJECT)&&state==5);
 REQUIRE(!ARpcServer_releaseWindowsNdr(&server)&&!server);
 LONG s,c,r,f;WlrCounters(&s,&c,&r,&f);REQUIRE(!s&&!c&&!r&&!f&&!WlrStop());
 printf("managed_lifecycle mode=%ls PASS state=%u loopStatus=%d actual_join_complete=1 explicit_listener_stop=1 counters=0\n",mode.c_str(),state,loop);return 0;
}
