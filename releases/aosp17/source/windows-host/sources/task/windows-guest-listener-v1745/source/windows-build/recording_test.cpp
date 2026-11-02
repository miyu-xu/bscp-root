#include <binder/RecordedTransaction.h>
#include <binder/Parcel.h>
#include <utils/String16.h>
#include <io.h>
#include <fcntl.h>
#include <cstdio>
#include "mapping_fault_inject.h"
using android::binder::debug::RecordedTransaction;
int main(){
 HANDLE h=CreateFileW(L"recording-test-owned.tmp",GENERIC_READ|GENERIC_WRITE,0,nullptr,CREATE_NEW,FILE_ATTRIBUTE_TEMPORARY|FILE_FLAG_DELETE_ON_CLOSE,nullptr);
 if(h==INVALID_HANDLE_VALUE)return 1;
 int raw=_open_osfhandle(reinterpret_cast<intptr_t>(h),_O_BINARY|_O_NOINHERIT);if(raw<0){CloseHandle(h);return 2;}
 android::binder::unique_fd fd(raw);
 // Prefix makes the first mapped payload have a nonzero page-relative offset.
 uint64_t prefix=0;if(_write(fd.get(),&prefix,sizeof(prefix))!=sizeof(prefix))return 3;
 android::Parcel data,reply;
 if(data.writeInt32(118)!=android::OK || reply.writeInt32(119)!=android::OK)return 4;
 auto record=RecordedTransaction::fromDetails(android::String16(u"bscp.a17.record"),7,0,{123,456},data,reply,android::OK);
 if(!record || record->dumpToFile(fd)!=android::OK)return 5;
 auto end=_lseek(fd.get(),0,SEEK_END);if(end<=8)return 6;
 _lseek(fd.get(),8,SEEK_SET);auto read=RecordedTransaction::fromFile(fd);
 if(!read || read->getCode()!=7 || read->getTimestamp().tv_sec!=123 || read->getTimestamp().tv_nsec!=456)return 7;
 _lseek(fd.get(),8,SEEK_SET);mappingFailure=true;auto failed=RecordedTransaction::fromFile(fd);mappingFailure=false;
 if(failed)return 8;
 if(_chsize_s(fd.get(),end-1))return 9;
 _lseek(fd.get(),8,SEEK_SET);if(RecordedTransaction::fromFile(fd))return 10;
 std::puts("A17_REAL_RECORDING_ROUNDTRIP_NONZERO_OFFSET_MAP_FAILURE_TRUNCATION_PASS");return 0;
}
