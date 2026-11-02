#include <binder/Binder.h>
#include <binder/RpcServer.h>
#include <binder/RpcSession.h>
#include <binder/Parcel.h>
#include <thread>
#include <condition_variable>
#include <atomic>
#include <cstdio>
#include <cstdlib>
#include <cstring>
using namespace android;
struct Gate {std::mutex m;std::condition_variable cv;bool entered=false,release=false;std::atomic<int> calls{0};};
class Slow final:public BBinder {Gate& g;public:explicit Slow(Gate& x):g(x){}
 status_t onTransact(uint32_t code,const Parcel& data,Parcel* reply,uint32_t) override {
  if(code!=1||!reply)return BAD_VALUE;int32_t n=0;if(data.readInt32(&n)!=OK||n!=118)return BAD_VALUE;
  ++g.calls;std::unique_lock<std::mutex> l(g.m);g.entered=true;g.cv.notify_all();
  if(!g.cv.wait_for(l,std::chrono::seconds(3),[&]{return g.release;}))return TIMED_OUT;
  return reply->writeInt32(n+1);
 }};
int run(bool disconnect){
 Gate g;auto server=RpcServer::make();if(!server)return 1;server->setMaxThreads(2);server->setRootObject(sp<Slow>::make(g));unsigned port=0;
 if(server->setupInetServer("127.0.0.1",0,&port)!=OK||!port)return 2;server->start();auto session=RpcSession::make();
 if(!session||session->setupInetClient("127.0.0.1",port)!=OK)return 3;auto root=session->getRootObject();if(!root)return 4;
 std::mutex m;std::condition_variable cv;bool done=false;status_t status=OK;size_t bytes=999;
 std::thread call([&]{Parcel in,out;in.markForRpc(session);if(in.writeInt32(118)!=OK){status=BAD_VALUE;}else status=root->transact(1,in,&out,0);bytes=out.dataSize();{std::lock_guard<std::mutex> l(m);done=true;}cv.notify_all();});
 {std::unique_lock<std::mutex> l(g.m);if(!g.cv.wait_for(l,std::chrono::seconds(2),[&]{return g.entered;}))std::_Exit(21);}
 bool premature;{std::unique_lock<std::mutex> l(m);premature=cv.wait_for(l,std::chrono::milliseconds(150),[&]{return done;});}
 bool shutdownResult=false;
 std::thread shutdown([&]{shutdownResult=disconnect?server->shutdown():session->shutdownAndWait(true);});
 bool beforeRelease;{std::unique_lock<std::mutex> l(m);beforeRelease=cv.wait_for(l,std::chrono::milliseconds(600),[&]{return done;});}
 {std::lock_guard<std::mutex> l(g.m);g.release=true;}g.cv.notify_all();call.join();shutdown.join();
 root.clear();bool clientFinal=session->shutdownAndWait(true);bool serverFinal=disconnect?true:server->shutdown();session.clear();server.clear();
 std::printf("mode=%s entered=%d premature=%d caller_before_release=%d status=%d reply_bytes=%zu shutdown=%d final_client=%d final_server=%d calls=%d\n",disconnect?"disconnect":"client_deadline",g.entered,premature,beforeRelease,status,bytes,shutdownResult,clientFinal,serverFinal,g.calls.load());
 if(premature||!beforeRelease||status==OK||bytes!=0||g.calls!=1||!shutdownResult||!serverFinal)return 5;
 return 0;
}
int main(int argc,char** argv){std::mutex m;std::condition_variable cv;bool done=false;std::thread deadline([&]{std::unique_lock<std::mutex> l(m);if(!cv.wait_for(l,std::chrono::seconds(10),[&]{return done;}))std::_Exit(90);});int r=run(argc==2&&!std::strcmp(argv[1],"disconnect"));{std::lock_guard<std::mutex> l(m);done=true;}cv.notify_all();deadline.join();return r;}
