#pragma once
#include "WindowsLocalRpcBridge.h"
#include <span>
#include <vector>
namespace bscp::windows {
inline constexpr size_t kNdrBinderPacketMax=614399;
inline constexpr size_t kNdrDescriptorMax=253;
inline constexpr size_t kNdrEnvelopeMax=16+2*kNdrDescriptorMax+kNdrBinderPacketMax;
// Holds bytes and borrowed typed handles. The caller must keep all input HANDLEs
// alive through the one synchronous WlrSend; this object never closes them.
struct WindowsNdrEncodedPacket {
 std::vector<unsigned char> bytes;
 std::vector<HANDLE> files,pipes;
};
struct WindowsNdrDecodedPacket {
 std::span<const unsigned char> binderBytes;
 std::vector<HANDLE> orderedHandles; // borrowed from the original record owner
};
DWORD encodeWindowsNdrPacket(std::span<const unsigned char> binderBytes,
                            std::span<const HANDLE> orderedHandles,
                            WindowsNdrEncodedPacket*);
// No identity/authorization claim: caller must separately retain the authenticated
// record. On rejection, its owning record must be closed and must not dispatch.
DWORD decodeWindowsNdrPacket(const WlrView&,WindowsNdrDecodedPacket*);
}
