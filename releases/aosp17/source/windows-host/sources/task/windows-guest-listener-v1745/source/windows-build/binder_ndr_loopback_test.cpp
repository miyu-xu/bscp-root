#include <binder/Binder.h>
#include <binder/RpcServer.h>
#include <binder/RpcSession.h>
#include <binder/Parcel.h>
#include "WindowsLocalRpcBridge.h"
#include "WindowsCallerIdentity.h"
#include "WindowsFdAbi.h"
#include <atomic>
#include <thread>
#include <cstdio>
using namespace android;using namespace bscp::windows;
#define CHECK(x) do{if(!(x)){printf("FAIL line=%d error=%lu\n",__LINE__,GetLastError());return 1;}}while(0)
static std::atomic<unsigned> calls{0},factoryCalls{0},filterCalls{0},callbacks{0};
static bool actualCaller(){const auto*p=CallerScope::current();DWORD pid=0;return p&&p->queryTokenForAccessCheck()&&!p->checkedCallingPid(&pid)&&pid==GetCurrentProcessId();}
class Callback final:public BBinder {status_t onTransact(uint32_t code,const Parcel&,Parcel* reply,uint32_t)override{if(code!=1||CallerScope::current()||!reply)return PERMISSION_DENIED;++callbacks;return reply->writeInt32(42);}};
class Echo final:public BBinder {status_t onTransact(uint32_t code,const Parcel& data,Parcel* reply,uint32_t)override{
 if(!actualCaller())return PERMISSION_DENIED;
 if(code==1){int32_t n=0;if(!reply||data.readInt32(&n))return BAD_VALUE;++calls;return reply->writeInt32(n+1);}
 if(code==2){auto cb=data.readStrongBinder();if(!cb||!reply)return BAD_VALUE;Parcel input,output;input.markForBinder(cb);status_t r=cb->transact(1,input,&output,0);int32_t value=0;if(r||output.readInt32(&value)||value!=42||!actualCaller())return BAD_VALUE;return reply->writeInt32(value);}
 if(code==3){++calls;return OK;}
 if(code==4){int fd=data.readFileDescriptor();HANDLE h=nullptr;if(fd<0||bscp_fd_to_handle(fd,&h))return BAD_VALUE;DWORD type=GetFileType(h);if(!CloseHandle(h)||type!=FILE_TYPE_DISK||!reply)return BAD_VALUE;return reply->writeInt32(1);}
 return BAD_VALUE;
}};
int wmain(){std::thread([]{Sleep(30000);ExitProcess(90);}).detach();std::wstring endpoint=L"binder-owned-"+std::to_wstring(GetCurrentProcessId());WlrServerAcceptor*acceptor=nullptr;CHECK(!WlrStartOwnedListener(endpoint.c_str(),&acceptor));auto server=RpcServer::makeWindowsDesktop(&acceptor);CHECK(server&&!acceptor);server->setMaxThreads(2);server->setSupportedFileDescriptorTransportModes({RpcSession::FileDescriptorTransportMode::UNIX});server->setConnectionFilter([](const void*addr,size_t n){if(addr||n||!actualCaller())return false;++filterCalls;return true;});server->setPerSessionRootObject([](wp<RpcSession>,const void*addr,size_t n)->sp<IBinder>{if(addr||n||!actualCaller())return nullptr;++factoryCalls;return sp<Echo>::make();});server->start();
 HANDLE self=nullptr;CHECK(DuplicateHandle(GetCurrentProcess(),GetCurrentProcess(),GetCurrentProcess(),&self,PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,FALSE,0));CallerIdentity allowed;CHECK(!CallerIdentity::currentProcess(&allowed));WlrAuthenticatedOwner*owner=nullptr;CHECK(!WlrCreateInitialAuthenticated(endpoint.c_str(),allowed.sid.data(),static_cast<DWORD>(allowed.sid.size()),self,&owner)&&CloseHandle(self));auto client=RpcSession::makeWindowsDesktop(&owner);CHECK(client&&!owner);client->setMaxIncomingThreads(1);client->setMaxOutgoingConnections(2);client->setFileDescriptorTransportMode(RpcSession::FileDescriptorTransportMode::UNIX);CHECK(client->setupWindowsDesktopClient()==OK);auto root=client->getRootObject();CHECK(root);
 {Parcel in,out;in.markForRpc(client);CHECK(in.writeInt32(118)==OK&&root->transact(1,in,&out,0)==OK);int32_t v=0;CHECK(out.readInt32(&v)==OK&&v==119);}
 {Parcel in,out;in.markForRpc(client);CHECK(in.writeStrongBinder(sp<Callback>::make())==OK&&root->transact(2,in,&out,0)==OK);int32_t v=0;CHECK(out.readInt32(&v)==OK&&v==42&&callbacks==1);}
 for(int i=0;i<4;++i){Parcel in;in.markForRpc(client);CHECK(root->transact(3,in,nullptr,IBinder::FLAG_ONEWAY)==OK);}ULONGLONG deadline=GetTickCount64()+3000;while(calls<5&&GetTickCount64()<deadline)Sleep(1);CHECK(calls==5);
 {std::wstring path=L"binder-typed-"+std::to_wstring(GetCurrentProcessId());HANDLE h=CreateFileW(path.c_str(),GENERIC_READ|GENERIC_WRITE,0,nullptr,CREATE_NEW,FILE_FLAG_DELETE_ON_CLOSE,nullptr);CHECK(h!=INVALID_HANDLE_VALUE);int fd=-1;CHECK(!bscp_fd_from_handle(h,&fd)&&CloseHandle(h));Parcel in,out;in.markForRpc(client);CHECK(in.writeFileDescriptor(fd,false)==OK&&root->transact(4,in,&out,0)==OK);int32_t v=0;CHECK(out.readInt32(&v)==OK&&v==1&&!bscp_fd_close(fd));}
 root.clear();CHECK(client->shutdownAndWait(true));client.clear();CHECK(server->shutdown());server.clear();LONG ns,nc,nr,nf;WlrCounters(&ns,&nc,&nr,&nf);CHECK(!ns&&!nc&&!nr&&!nf&&!WlrStop());CHECK(factoryCalls==1&&filterCalls>=2&&!CallerScope::current());printf("real_Binder_echo_nested_callback_scope_oneway_FD_shutdown=PASS factory=%u filter=%u\n",factoryCalls.load(),filterCalls.load());return 0;}

