#include <binder/Binder.h>
#include <binder/RpcServer.h>
#include <binder/RpcSession.h>
#include <binder/Parcel.h>
#include <thread>
#include <cstdio>
#include <cstdlib>
#include <cstring>
using namespace android;
class ExitPeer final:public BBinder{status_t onTransact(uint32_t code,const Parcel& data,Parcel*,uint32_t)override{int32_t n=0;if(code!=1||data.readInt32(&n)!=OK||n!=118)return BAD_VALUE;std::puts("ONLY_CALL 118");std::fflush(stdout);std::this_thread::sleep_for(std::chrono::milliseconds(150));std::_Exit(73);}};
int main(int argc,char** argv){std::thread([]{std::this_thread::sleep_for(std::chrono::seconds(10));std::_Exit(90);}).detach();
 if(argc==2&&!std::strcmp(argv[1],"server")){auto server=RpcServer::make();server->setMaxThreads(1);server->setRootObject(sp<ExitPeer>::make());unsigned port=0;if(server->setupInetServer("127.0.0.1",0,&port)!=OK)return 1;std::printf("PORT %u\n",port);std::fflush(stdout);server->join();return 2;}
 if(argc!=3)return 3;auto session=RpcSession::make();if(session->setupInetClient("127.0.0.1",unsigned(std::strtoul(argv[2],nullptr,10)))!=OK)return 4;auto root=session->getRootObject();if(!root)return 5;
 Parcel in,out;in.markForRpc(session);if(in.writeInt32(118)!=OK)return 6;auto status=root->transact(1,in,&out,0);std::printf("status=%d reply_bytes=%zu\n",status,out.dataSize());root.clear();bool closed=session->shutdownAndWait(true);std::printf("shutdown=%d\n",closed);return status!=OK&&out.dataSize()==0&&closed?0:7;}
