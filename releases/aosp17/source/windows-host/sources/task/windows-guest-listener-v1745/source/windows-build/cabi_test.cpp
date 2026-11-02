#include "WindowsBinderRpc.h"
#include "WindowsLocalRpcBridge.h"
#include <binder_rpc_unstable.hpp>
#include <android/binder_libbinder.h>
#include <android/binder_ibinder.h>
#include <binder/Binder.h>
#include <utils/Errors.h>
#include <thread>
#include <stdexcept>
#include <cstdio>
using namespace android;
#define CHECK(x)do{if(!(x)){printf("FAIL line=%d error=%lu\n",__LINE__,GetLastError());return 1;}}while(0)
class ThrowRoot final:public BBinder{public:int failure=0;BBinder*localBinder()override{if(failure==1)throw std::bad_alloc();if(failure==2)throw std::runtime_error("fixture root config");return this;}};
int wmain(){std::thread([]{Sleep(15000);ExitProcess(90);}).detach();auto lib=GetModuleHandleW(L"libbinder_rpc_unstable_host.dll");CHECK(lib&&GetProcAddress(lib,"ARpcServer_newWindowsNdr")&&GetProcAddress(lib,"ARpcSession_newWindowsNdr")&&GetProcAddress(lib,"ARpcSession_setupWindowsNdr")&&GetProcAddress(lib,"ARpcServer_free"));
 auto*initial=reinterpret_cast<WlrAuthenticatedOwner*>(uintptr_t{0x1234});auto*saved=initial;auto*session=reinterpret_cast<ARpcSession*>(uintptr_t{0x9876});CHECK(ARpcSession_newWindowsNdr(&initial,&session)==PERMISSION_DENIED&&initial==saved&&!session);AIBinder*outRoot=reinterpret_cast<AIBinder*>(uintptr_t{1});CHECK(ARpcSession_setupWindowsNdr(reinterpret_cast<ARpcSession*>(uintptr_t{1}),&outRoot)==PERMISSION_DENIED&&!outRoot);CHECK(ARpcSession_newWindowsNdr(nullptr,&session)==BAD_VALUE&&!session);
 std::wstring ep=L"cabi-owner-"+std::to_wstring(GetCurrentProcessId());WlrServerAcceptor*acceptor=nullptr;CHECK(!WlrStartOwnedListener(ep.c_str(),&acceptor));auto original=acceptor;auto root=sp<ThrowRoot>::make();AIBinder*ndk=AIBinder_fromPlatformBinder(root);CHECK(ndk);ARpcServer*server=reinterpret_cast<ARpcServer*>(uintptr_t{1});
 root->failure=1;CHECK(ARpcServer_newWindowsNdr(ndk,&acceptor,&server)==NO_MEMORY&&!server&&acceptor==original&&!WlrAcceptorCheckLive(acceptor));root->failure=2;CHECK(ARpcServer_newWindowsNdr(ndk,&acceptor,&server)==UNKNOWN_ERROR&&!server&&acceptor==original&&!WlrAcceptorCheckLive(acceptor));root->failure=0;CHECK(ARpcServer_newWindowsNdr(ndk,&acceptor,&server)==OK&&server&&!acceptor);CHECK(WlrStop()==ERROR_BUSY);ARpcServer_free(server);server=nullptr;AIBinder_decStrong(ndk);root.clear();LONG ns,nc,nr,nf;WlrCounters(&ns,&nc,&nr,&nf);CHECK(!ns&&!nc&&!nr&&!nf&&!WlrStop());printf("C_exports=PASS client_unimplemented_admission_denied=PASS exception_before_cap_commit=PASS constructor_free_then_explicit_stop=PASS counters0=PASS\n");return 0;}
