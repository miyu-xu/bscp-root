#include "WindowsCallerIdentity.h"
#include "WindowsAccessControl.h"
#include <sddl.h>
#include <cstdio>
#include <string>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <chrono>
#include <cstdlib>
using namespace bscp::windows;
bool io(HANDLE pipe,bool write,char* data) {
 OVERLAPPED ov{};ov.hEvent=CreateEventW(nullptr,TRUE,FALSE,nullptr);if(!ov.hEvent)return false;
 DWORD n=0;BOOL ok=write?WriteFile(pipe,data,1,&n,&ov):ReadFile(pipe,data,1,&n,&ov);
 if(!ok && GetLastError()==ERROR_IO_PENDING){DWORD wait=WaitForSingleObject(ov.hEvent,8000);
  if(wait!=WAIT_OBJECT_0){CancelIoEx(pipe,&ov);GetOverlappedResult(pipe,&ov,&n,TRUE);CloseHandle(ov.hEvent);return false;}
  ok=GetOverlappedResult(pipe,&ov,&n,FALSE);
 }
 CloseHandle(ov.hEvent);return ok&&n==1;
}
int child(const wchar_t* name,bool anonymous) {
 HANDLE pipe=CreateFileW(name,GENERIC_READ|GENERIC_WRITE,0,nullptr,OPEN_EXISTING,
   FILE_FLAG_OVERLAPPED|SECURITY_SQOS_PRESENT|SECURITY_CONTEXT_TRACKING|(anonymous?SECURITY_ANONYMOUS:SECURITY_IDENTIFICATION),nullptr);
 if(pipe==INVALID_HANDLE_VALUE)return 21;
 char byte='I';bool good=io(pipe,true,&byte)&&io(pipe,false,&byte)&&byte=='A';
 if(good&&!anonymous){
  if(!ImpersonateAnonymousToken(GetCurrentThread()))good=false;
  else {byte='N';good=io(pipe,true,&byte);if(!RevertToSelf())std::abort();}
  if(good)good=io(pipe,false,&byte)&&byte=='B';
 }
 CloseHandle(pipe);return good?0:22;
}
bool noToken(){HANDLE h=nullptr;if(OpenThreadToken(GetCurrentThread(),TOKEN_QUERY,TRUE,&h)){CloseHandle(h);return false;}return GetLastError()==ERROR_NO_TOKEN;}

int fileAccessTests(const CallerIdentity& captured) {
 wchar_t exe[32768]{};if(!GetModuleFileNameW(nullptr,exe,32768))return 41;
 std::wstring parent=exe;parent.resize(parent.find_last_of(L"\\/"));
 LPWSTR sid=nullptr;if(!ConvertSidToStringSidW(const_cast<unsigned char*>(captured.sid.data()),&sid))return 42;
 std::wstring sddl=L"O:"+std::wstring(sid)+L"D:P(A;OICI;FA;;;"+sid+L")(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)";LocalFree(sid);
 PSECURITY_DESCRIPTOR sd=nullptr;if(!ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl.c_str(),SDDL_REVISION_1,&sd,nullptr))return 43;
 std::wstring directory=parent+L"\\private";
 std::wprintf(L"private_create=%ls sddl=%ls\n",directory.c_str(),sddl.c_str());
 SECURITY_ATTRIBUTES sa{sizeof(sa),sd,FALSE};BOOL created=CreateDirectoryW(directory.c_str(),&sa);DWORD createError=created?0:GetLastError();LocalFree(sd);
 if(!created){std::printf("private create error=%lu\n",createError);return 44;}
 HANDLE file=CreateFileW((directory+L"\\image.bin").c_str(),GENERIC_READ|GENERIC_WRITE,FILE_SHARE_READ|FILE_SHARE_WRITE,nullptr,CREATE_NEW,FILE_ATTRIBUTE_NORMAL,nullptr);
 HANDLE foreign=CreateFileW((parent+L"\\inherited.bin").c_str(),GENERIC_READ|GENERIC_WRITE,FILE_SHARE_READ|FILE_SHARE_WRITE,nullptr,CREATE_NEW,FILE_ATTRIBUTE_NORMAL,nullptr);
 HANDLE dir=CreateFileW(directory.c_str(),GENERIC_READ,FILE_SHARE_READ|FILE_SHARE_WRITE,nullptr,OPEN_EXISTING,FILE_FLAG_BACKUP_SEMANTICS,nullptr);
 HANDLE read=CreateFileW((directory+L"\\image.bin").c_str(),GENERIC_READ,FILE_SHARE_READ|FILE_SHARE_WRITE,nullptr,OPEN_EXISTING,FILE_ATTRIBUTE_NORMAL,nullptr);
 int result=0;FileAccessEvidence evidence{};
 if(file==INVALID_HANDLE_VALUE||foreign==INVALID_HANDLE_VALUE||dir==INVALID_HANDLE_VALUE||read==INVALID_HANDLE_VALUE)result=45;
 if(!result&&checkCurrentCallerFile(file,FileAccessKind::ReadImage,&evidence)!=ERROR_NO_TOKEN)result=46;
 {
 CallerScope scope(&captured);
 DWORD status=checkCurrentCallerFile(file,FileAccessKind::ReadImage,&evidence);std::printf("private rdwr image read status=%lu handle=%lx caller=%lx\n",status,evidence.handleGranted,evidence.callerGranted);
 if(!result&&status)result=47;
 if(!result&&checkCurrentCallerFile(read,FileAccessKind::ReadImage,&evidence))result=48;
 if(!result&&checkCurrentCallerFile(file,FileAccessKind::WriteInstance,&evidence))result=49;
 if(!result&&checkCurrentCallerFile(read,FileAccessKind::WriteInstance,&evidence)!=ERROR_ACCESS_DENIED)result=50;
 if(!result&&checkCurrentCallerFile(dir,FileAccessKind::ReadOwnedDirectory,&evidence))result=51;
 if(!result&&checkCurrentCallerFile(file,FileAccessKind::ReadOwnedDirectory,&evidence)!=ERROR_INVALID_DATA)result=52;
 if(!result&&checkCurrentCallerFile(dir,FileAccessKind::ReadImage,&evidence)!=ERROR_INVALID_DATA)result=53;
 DWORD inherited=checkCurrentCallerFile(foreign,FileAccessKind::ReadImage,&evidence);std::printf("default inherited image status=%lu\n",inherited);
 if(!result&&inherited!=ERROR_ACCESS_DENIED)result=54;
 if(!result&&checkCurrentCallerFile(INVALID_HANDLE_VALUE,FileAccessKind::ReadImage,&evidence)!=ERROR_INVALID_HANDLE)result=55;
 }
 if(read!=INVALID_HANDLE_VALUE)CloseHandle(read);if(file!=INVALID_HANDLE_VALUE)CloseHandle(file);
 if(foreign!=INVALID_HANDLE_VALUE)CloseHandle(foreign);if(dir!=INVALID_HANDLE_VALUE)CloseHandle(dir);
 return result;
}
int runCase(bool anonymous) {
 std::wstring name=L"\\\\.\\pipe\\bscp_identity_v1460_"+std::to_wstring(GetCurrentProcessId())+L"_"+std::to_wstring(GetTickCount64());
 HANDLE pipe=CreateNamedPipeW(name.c_str(),PIPE_ACCESS_DUPLEX|FILE_FLAG_FIRST_PIPE_INSTANCE,
  PIPE_TYPE_MESSAGE|PIPE_READMODE_MESSAGE|PIPE_WAIT|PIPE_REJECT_REMOTE_CLIENTS,1,4096,4096,0,nullptr);
 if(pipe==INVALID_HANDLE_VALUE)return 1;
 wchar_t exe[32768]{};if(!GetModuleFileNameW(nullptr,exe,32768))return 2;
 std::wstring cmd=L"\""+std::wstring(exe)+L"\" --child \""+name+L"\" "+(anonymous?L"anon":L"identify");
 STARTUPINFOW si{};si.cb=sizeof(si);PROCESS_INFORMATION pi{};
 if(!CreateProcessW(exe,cmd.data(),nullptr,nullptr,FALSE,CREATE_NO_WINDOW,nullptr,nullptr,&si,&pi)){CloseHandle(pipe);return 3;}
 CloseHandle(pi.hThread);
 int result=0;bool connected=ConnectNamedPipe(pipe,nullptr)||GetLastError()==ERROR_PIPE_CONNECTED;
 char byte=0;DWORD count=0;
 if(!connected||!ReadFile(pipe,&byte,1,&count,nullptr)||count!=1||byte!='I')result=4;
 CallerIdentity allowed,request;
 if(!result&&CallerIdentity::currentProcess(&allowed))result=5;
 std::shared_ptr<const BoundPipePeer> peer;
 if(!result&&BoundPipePeer::bind(pipe,pi.hProcess,allowed,&peer))result=6;
 if(!result){DWORD status=authenticatePipeRequest(pipe,allowed,peer.get(),&request);
  if(anonymous){if(status==ERROR_SUCCESS)result=7;}else if(status || !request.sameAs(allowed))result=8;
 }
 if(!result&&!noToken())result=9;
 if(!result&&!anonymous)result=fileAccessTests(request);
 if(!result&&!anonymous){
  CallerIdentity wrong=allowed;wrong.logon.LowPart^=1;
  if(authenticatePipeRequest(pipe,wrong,peer.get(),&request)!=ERROR_ACCESS_DENIED||!request.sid.empty()||!noToken())result=10;
  std::shared_ptr<const BoundPipePeer> wrongPeer;
  HANDLE self=nullptr;
  if(!DuplicateHandle(GetCurrentProcess(),GetCurrentProcess(),GetCurrentProcess(),&self,SYNCHRONIZE|PROCESS_QUERY_LIMITED_INFORMATION,FALSE,0))result=18;
  else {if(BoundPipePeer::bind(pipe,self,allowed,&wrongPeer)!=ERROR_ACCESS_DENIED||wrongPeer)result=11;CloseHandle(self);}
  if(authenticatePipeRequest(INVALID_HANDLE_VALUE,allowed,nullptr,&request)==ERROR_SUCCESS||!noToken())result=12;
  DWORD flags=0;if(!GetHandleInformation(peer->processHandle(),&flags)||(flags&HANDLE_FLAG_INHERIT))result=13;
  CallerIdentity malformed=allowed;malformed.sid.resize(1);if(malformed.sameAs(allowed))result=19;
  // Reject an ambient impersonation context and leave it untouched for its owner.
  if(!ImpersonateSelf(SecurityIdentification))result=23;
  else {
   if(authenticatePipeRequest(pipe,allowed,peer.get(),&request)!=ERROR_BAD_IMPERSONATION_LEVEL || noToken())result=24;
   if(!RevertToSelf())std::abort();if(!noToken())result=25;
  }
 }
 if(!result&&!anonymous){
  if(CallerScope::current())result=26;
  {CallerScope outer(&allowed);
   if(!CallerScope::current()||!CallerScope::current()->sameAs(allowed))result=27;
   {CallerScope clear(nullptr);if(CallerScope::current())result=28;}
   if(!CallerScope::current()||!CallerScope::current()->sameAs(allowed))result=29;
   CallerIdentity inner=allowed;inner.logon.LowPart^=1;
   {CallerScope nested(&inner);if(!CallerScope::current()->sameAs(inner))result=30;}
   if(!CallerScope::current()->sameAs(allowed))result=31;
  }
  if(CallerScope::current())result=32;
 }
 byte='A';if(!WriteFile(pipe,&byte,1,&count,nullptr)&&!result)result=14;
 if(!anonymous){
  if(!ReadFile(pipe,&byte,1,&count,nullptr)||count!=1||byte!='N'){if(!result)result=33;}
  else if(authenticatePipeRequest(pipe,allowed,peer.get(),&request)==ERROR_SUCCESS || !request.sid.empty() || !noToken()){if(!result)result=34;}
  byte='B';if(!WriteFile(pipe,&byte,1,&count,nullptr)&&!result)result=35;
 }
 if(WaitForSingleObject(pi.hProcess,10000)!=WAIT_OBJECT_0){if(!result)result=15;}
 DWORD exit=255;if(!GetExitCodeProcess(pi.hProcess,&exit)||exit!=0){if(!result)result=16;}
 if(peer&&peer->validate(pipe,allowed)!=ERROR_PROCESS_ABORTED&&!result)result=17;
 peer.reset();CloseHandle(pi.hProcess);DisconnectNamedPipe(pipe);CloseHandle(pipe);return result;
}
int wmain(int argc,wchar_t** argv){
 if(argc==4&&std::wstring(argv[1])==L"--child")return child(argv[2],std::wstring(argv[3])==L"anon");
 std::mutex mutex;std::condition_variable cv;bool done=false;
 std::thread deadline([&]{std::unique_lock<std::mutex> lock(mutex);if(!cv.wait_for(lock,std::chrono::seconds(25),[&]{return done;}))std::_Exit(90);});
 int result=runCase(false);if(!result)result=runCase(true);
 {std::lock_guard<std::mutex> lock(mutex);done=true;}cv.notify_one();deadline.join();
 std::printf("identity real child requests: %s exit=%d\n",result?"FAIL":"PASS",result);return result;
}
