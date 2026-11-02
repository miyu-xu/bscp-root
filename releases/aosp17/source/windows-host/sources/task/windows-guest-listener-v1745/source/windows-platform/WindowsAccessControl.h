#pragma once
#include "WindowsCallerIdentity.h"
#include <cstdint>
namespace bscp::windows {
enum class FileAccessKind { ReadImage, WriteInstance, ReadOwnedDirectory };
struct FileAccessEvidence {
    DWORD volumeSerial=0;
    std::uint64_t fileIndex=0;
    DWORD links=0;
    DWORD attributes=0;
    ACCESS_MASK handleGranted=0;
    ACCESS_MASK callerGranted=0;
};
// Only the already-held object is queried. Does not resolve/open a path, change ACLs,
// impersonate, or use server identity. Caller owns HANDLE throughout the check/use.
// Rust must still bind its final path to this object ID and reject ancestor traversal.
// ReadImage permits a readable RDWR handle; WriteInstance requires read+write data.
DWORD checkCurrentCallerFile(HANDLE file,FileAccessKind kind,FileAccessEvidence* out);
}
