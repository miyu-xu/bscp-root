#pragma once
#include <windows.h>
extern "C" {
__declspec(dllexport) int bscp_fd_close(int fd);
__declspec(dllexport) int bscp_fd_dup(int fd, int* out);
// Borrowed input; creates a noninheritable duplicate in the native descriptor table (async pipes retain native HANDLE ownership).
__declspec(dllexport) int bscp_fd_from_handle(HANDLE input, int* out);
// Caller owns the successful noninheritable duplicate. SOCKET is not a CRT handle.
// Explicit async-pipe registration, same original-access duplicate; never CRT pipe I/O.
__declspec(dllexport) int bscp_fd_from_overlapped_pipe_handle(HANDLE input, int* out);
__declspec(dllexport) int bscp_fd_to_handle(int fd, HANDLE* out);
}
