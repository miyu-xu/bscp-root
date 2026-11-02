#pragma once
#include <windows.h>
#include <stdint.h>
// All returns are Win32 DWORD status: 0 success; no scope is ERROR_NO_TOKEN.
// These APIs observe only the current authenticated dispatch snapshot.
// Consume outputs only on status 0. Identity metadata is reset on failure, but
// SID bytes are not overwritten. An invalid evidence size is not written.
struct BscpFileEvidence {
 uint32_t version, volume_serial;
 uint64_t file_index;
 uint32_t links, attributes, handle_granted, caller_granted;
};
extern "C" {
__declspec(dllexport) DWORD bscp_rpc_check_owner();
// Actual NDR request original-process proof only; no scope/proof returns an error.
__declspec(dllexport) DWORD bscp_rpc_current_pid(DWORD* out);
__declspec(dllexport) DWORD bscp_rpc_current_identity(unsigned char* sid,uint32_t capacity,
 uint32_t* required,uint32_t* logon_low,int32_t* logon_high);
// kind: 0 ReadImage, 1 WriteInstance, 2 ReadOwnedDirectory. No path open or ACL mutation.
__declspec(dllexport) DWORD bscp_rpc_check_file(HANDLE file,uint32_t kind,
 BscpFileEvidence* evidence,uint32_t evidence_size);
}
