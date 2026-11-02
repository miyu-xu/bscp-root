#include "WindowsNdrRecordAdapter.h"
#include <cstdio>
#include <string>
#include <vector>
#include <thread>
using namespace bscp::windows;
#define CHECK(x) do {if(!(x)){std::printf("FAIL line=%d error=%lu\n",__LINE__,GetLastError());return 1;}}while(0)
static LONG records(){LONG s,c,r,f;WlrCounters(&s,&c,&r,&f);return r;}
int wmain(int argc,wchar_t**){
 std::thread([]{Sleep(20000);ExitProcess(90);}).detach();
 std::wstring endpoint=L"wlr-proof-"+std::to_wstring(GetCurrentProcessId());CHECK(WlrStartIsolated(endpoint.c_str())==0);
 void* b=nullptr;void* s=nullptr;void* c=nullptr;uint64_t g=0,id=0;
 HANDLE self=nullptr;CHECK(DuplicateHandle(GetCurrentProcess(),GetCurrentProcess(),GetCurrentProcess(),&self,PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,FALSE,0));
 CHECK(WlrBind(endpoint.c_str(),&b)==0);CHECK(WlrOpen(b,self,&s,&g)==0);CHECK(CloseHandle(self));CHECK(WlrConnect(s,g,&c,&id)==0);
 std::vector<unsigned char> data(614399,0x5a);
 if(argc>1){data.push_back(0x5a);DWORD e=WlrSend(c,g,1,data.data(),static_cast<uint32_t>(data.size()),nullptr,0,nullptr,0);DWORD raw=WlrLastRpcException();CHECK(e==WLR_RPC_AMBIGUOUS&&raw!=0&&records()==0);CHECK(WlrCloseChannel(&c)==0);CHECK(WlrCloseSession(&s)==0);CHECK(WlrFreeBinding(&b)==0);CHECK(WlrStop()==0);std::printf("oversize614400 rejected raw=%lu no_replay=1\n",raw);return 0;}
 CHECK(WlrSend(c,g,1,data.data(),static_cast<uint32_t>(data.size()),nullptr,0,nullptr,0)==0);
 std::shared_ptr<WindowsNdrReceivedRecord> r;CHECK(WindowsNdrReceivedRecord::tryPop(g,id,&r)==0);CHECK(r->byteCount()==614399);
 std::shared_ptr<const WindowsNdrRequestSnapshot> proof;
 CHECK(r->makeSnapshot(&proof)==ERROR_INVALID_DATA);CHECK(r->sealConsumedRecord()==ERROR_INVALID_DATA);
 std::vector<unsigned char> received(data.size());CHECK(r->readExact(std::span(received).first(16))==0);
 CHECK(r->readExact(std::span(received).subspan(16))==0);unsigned char extra=0;CHECK(r->readExact(std::span(&extra,1))==ERROR_INVALID_DATA);
 CHECK(received==data);CHECK(r->sealConsumedRecord()==0);CHECK(r->makeSnapshot(&proof)==0);
 DWORD pid=0;CHECK(proof->checkedCallingPid(&pid)==0&&pid==GetCurrentProcessId());r.reset();CHECK(records()==0);CHECK(proof->checkedCallingPid(&pid)==0);proof.reset();
 // Exact record boundary: a second queued record never fills the first read.
 CHECK(WlrSend(c,g,2,data.data(),3,nullptr,0,nullptr,0)==0);CHECK(WindowsNdrReceivedRecord::tryPop(g,id,&r)==0);
 CHECK(WlrSend(c,g,3,data.data(),3,nullptr,0,nullptr,0)==0);unsigned char four[4]{};CHECK(r->readExact(four)==ERROR_INVALID_DATA);r.reset();
 windowsNdrTestFailControlBlock(1);CHECK(WindowsNdrReceivedRecord::tryPop(g,id,&r)==ERROR_NOT_ENOUGH_MEMORY&&!r);CHECK(records()==0);
 CHECK(WlrSend(c,g,4,data.data(),3,nullptr,0,nullptr,0)==0);CHECK(WindowsNdrReceivedRecord::tryPop(g,id,&r)==0);CHECK(r->readExact(std::span(four,3))==0);CHECK(r->sealConsumedRecord()==0);
 windowsNdrTestFailControlBlock(2);CHECK(r->makeSnapshot(&proof)==ERROR_NOT_ENOUGH_MEMORY&&!proof);CHECK(records()==1);r.reset();CHECK(records()==0);
 // Forged identity fields in public outbound view are ignored, not proof.
 WlrView out{};out.sequence=1;out.data=data.data();out.bytes=3;out.requestToken=GetCurrentProcess();out.requestProcess=GetCurrentProcess();out.requestPid=GetCurrentProcessId();CHECK(WlrServerPush(g,id,&out)==0);
 WlrRecord* reverse=nullptr;CHECK(WlrReceive(c,g,&reverse)==0);HANDLE token=reinterpret_cast<HANDLE>(1),process=token;pid=1;
 CHECK(WlrRecordQueryRequestIdentity(reverse,&token,&process,&pid)==ERROR_ACCESS_DENIED&&!token&&!process&&!pid);WlrRecordRelease(reverse);

 std::wstring file=L"record-owned-"+std::to_wstring(GetCurrentProcessId())+L".tmp";
 HANDLE f=CreateFileW(file.c_str(),GENERIC_READ|GENERIC_WRITE,0,nullptr,CREATE_NEW,FILE_ATTRIBUTE_TEMPORARY|FILE_FLAG_DELETE_ON_CLOSE,nullptr);CHECK(f!=INVALID_HANDLE_VALUE);
 HANDLE pr=nullptr,pw=nullptr;CHECK(CreatePipe(&pr,&pw,nullptr,0));std::vector<HANDLE> fs(253,f);
 CHECK(WlrSend(c,g,5,data.data(),3,fs.data(),253,nullptr,0)==0);CHECK(WindowsNdrReceivedRecord::tryPop(g,id,&r)==0);CHECK(r->files().size()==253);r.reset();CHECK(records()==0);
 CHECK(WlrSend(c,g,6,data.data(),3,fs.data(),253,&pr,1)==ERROR_INVALID_PARAMETER);CHECK(WlrLastRpcException()==0&&records()==0);
 CHECK(WlrSend(c,g,6,data.data(),3,fs.data(),252,&pr,1)==0);CHECK(WindowsNdrReceivedRecord::tryPop(g,id,&r)==0);CHECK(r->files().size()==252&&r->pipes().size()==1);r.reset();CHECK(records()==0);
 std::vector<HANDLE> ps(253,pr);CHECK(WlrSend(c,g,7,data.data(),3,nullptr,0,ps.data(),253)==0);CHECK(WindowsNdrReceivedRecord::tryPop(g,id,&r)==0);CHECK(r->pipes().size()==253);r.reset();
 CHECK(WlrSend(c,g,8,data.data(),3,fs.data(),253,ps.data(),253)==ERROR_INVALID_PARAMETER);CHECK(records()==0&&WlrLastRpcException()==0);
 CHECK(CloseHandle(f)&&CloseHandle(pr)&&CloseHandle(pw));std::printf("typed_files253=PASS typed_pipes253=PASS mixed253=PASS total254_506_manager_reject=PASS\n");
 CHECK(WlrCloseChannel(&c)==0);CHECK(WlrCloseSession(&s)==0);CHECK(WlrFreeBinding(&b)==0);
 LONG ns,nc,nr,nf;WlrCounters(&ns,&nc,&nr,&nf);CHECK(ns==0&&nc==0&&nr==0&&nf==0);CHECK(WlrStop()==0);
 std::printf("max614399=PASS partial_exact=PASS cross_record_reject=PASS role_reject=PASS snapshot_after_release=PASS controlblock_oom1_2=PASS counters=0\n");return 0;
}
