#include "WindowsIdentityAbi.h"
#include <cstdio>
int main(){
 DWORD pid=99;
 if(bscp_rpc_current_pid(&pid)!=ERROR_NO_TOKEN || pid!=0)return 1;
 if(bscp_rpc_current_pid(nullptr)!=ERROR_INVALID_PARAMETER)return 2;
 auto m=GetModuleHandleW(L"libbinder_sdk.dll");
 if(!m||!GetProcAddress(m,"bscp_rpc_current_pid"))return 3;
 puts("real DLL PID export; missing scope rejected and output cleared");
 return 0;
}
