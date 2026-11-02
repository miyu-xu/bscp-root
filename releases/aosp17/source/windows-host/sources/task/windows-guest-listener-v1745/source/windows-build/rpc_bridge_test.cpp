#include <binder/Binder.h>
#include <binder_rpc_unstable.hpp>
#include <android/binder_ibinder.h>
#include <binder/RpcServer.h>
#include <binder/RpcSession.h>
#include <binder/Parcel.h>
#include <condition_variable>
#include <thread>
#include <cstdio>
#include <cstdlib>
#include <chrono>
using namespace android;
class Echo final:public BBinder {
 status_t onTransact(uint32_t code,const Parcel& data,Parcel* reply,uint32_t) override {
    if(code!=1 || !reply)return BAD_VALUE;
    int32_t input=0;auto r=data.readInt32(&input);if(r!=OK)return r;
    return reply->writeInt32(input+1);
 }
};
int test(){
 auto server=RpcServer::make();if(!server)return 1;
 server->setMaxThreads(2);server->setRootObject(sp<Echo>::make());unsigned int port=0;
 auto setup=server->setupInetServer("127.0.0.1",0,&port);if(setup!=OK || !port){std::fprintf(stderr,"setup status=%d errno=%d wsa=%d port=%u\n",setup,errno,WSAGetLastError(),port);return 2;}
 server->start();int result=0;
 {
 auto session=ARpcSession_new();
 if(!session)result=3;
 else {
  auto root=ARpcSession_setupInet(session,"127.0.0.1",port);
  if(!root)result=4;
  else {if(!AIBinder_isRemote(root) || AIBinder_ping(root)!=STATUS_OK)result=5;AIBinder_decStrong(root);}
  ARpcSession_free(session);
 }

 }
 if(!server->shutdown() && !result)result=8;
 server.clear();if(!result)std::puts("A17_WINDOWS_REAL_NDK_UNSTABLE_BRIDGE_PING_SHUTDOWN_PASS");return result;
}
int main(){
 std::mutex m;std::condition_variable cv;bool done=false;
 std::thread deadline([&]{std::unique_lock<std::mutex> l(m);if(!cv.wait_for(l,std::chrono::seconds(15),[&]{return done;}))std::_Exit(90);});
 int r=test();{std::lock_guard<std::mutex> l(m);done=true;}cv.notify_one();deadline.join();return r;
}
