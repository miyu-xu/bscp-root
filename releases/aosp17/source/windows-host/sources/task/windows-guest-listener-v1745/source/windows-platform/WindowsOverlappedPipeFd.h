#pragma once
#include <windows.h>
#include <memory>
namespace bscp::windows {
struct PipeSnapshot {DWORD access=0,mode=0,pipeFlags=0,pipeState=0;};
// Read-only native query, same classes/status checks as native_snapshot.rs.
// No object ACL, access increase, pipe-mode change, or synchronous I/O fallback.
DWORD inspectNativePipe(HANDLE,PipeSnapshot*);
struct OverlappedPipeOwner;
using OverlappedPipeLease=std::shared_ptr<OverlappedPipeOwner>;
OverlappedPipeLease acquireOverlappedPipe(int fd);
bool isOverlappedPipeSlot(int fd); // active or retired reserved slot, never CRT fallback
HANDLE borrowedOverlappedPipe(const OverlappedPipeLease&);
int registerOverlappedPipe(HANDLE borrowed,int* out); // duplicate, original unchanged
int duplicateOverlappedPipe(const OverlappedPipeLease&,int* out);
int copyOverlappedPipe(const OverlappedPipeLease&,HANDLE* out);
bool closeOverlappedPipe(int fd,int* result); // true means registered role was consumed
DWORD validateRawOverlappedPipe(const OverlappedPipeLease&);
DWORD validateRawOverlappedPipeHandle(HANDLE);
// Takes an exact owned handle; on failed close retains it in preallocated owner.
DWORD closeOrRetainPipe(HANDLE);
void releaseOrRetainPipe(HANDLE);
}
extern "C" __declspec(dllexport) int bscp_fd_from_overlapped_pipe_handle(HANDLE borrowed,int* out);
