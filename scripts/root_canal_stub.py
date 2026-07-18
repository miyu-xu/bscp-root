#!/usr/bin/env python3
"""Minimal cross-platform Bluetooth HCI controller stub.

This is a control-plane fallback, not a radio simulator.  It implements enough
H4 command handling for Android to enumerate a deterministic LE-only controller.
"""

from __future__ import annotations

import argparse
import signal
import socketserver
import struct
import threading

H4_COMMAND = 0x01
H4_ACL = 0x02
H4_SCO = 0x03
H4_EVENT = 0x04
H4_ISO = 0x05


def command_return_parameters(opcode: int, params: bytes = b"") -> bytes:
    success = b"\x00"
    if opcode == 0x1001:  # Read Local Version Information
        return struct.pack("<BBHBHH", 0, 0x0C, 1, 0x0C, 0x00E0, 1)
    if opcode == 0x1002:  # Read Local Supported Commands
        return success + bytes(64)
    if opcode == 0x1003:  # Read Local Supported Features: LE-only controller
        features = bytearray(8)
        features[4] = 0x60  # BR/EDR not supported, LE supported
        return success + bytes(features)
    if opcode == 0x1004:  # Read Local Extended Features
        page = params[0] if params else 0
        return bytes((0, page, 1)) + bytes(8)
    if opcode == 0x1005:  # Read Buffer Size
        return struct.pack("<BHBHH", 0, 1021, 0, 16, 0)
    if opcode == 0x1009:  # Read BD_ADDR (little-endian on HCI)
        return success + bytes.fromhex("56341200CF00")
    if opcode == 0x100B:  # Read Local Supported Codecs v1
        return success + b"\x00\x00"
    if opcode == 0x2002:  # LE Read Buffer Size v1
        return struct.pack("<BHB", 0, 251, 16)
    if opcode == 0x2003:  # LE Read Local Supported Features
        return success + b"\x01" + bytes(7)  # LE encryption
    if opcode == 0x200F:  # LE Read Filter Accept List Size
        return success + b"\x00"
    if opcode == 0x201C:  # LE Read Supported States
        return success + bytes([0xFF]) * 8
    if opcode == 0x2023:  # LE Read Suggested Default Data Length
        return struct.pack("<BHH", 0, 251, 2120)
    if opcode == 0x202A:  # LE Read Resolving List Size
        return success + b"\x00"
    if opcode == 0x202F:  # LE Read Maximum Data Length
        return struct.pack("<BHHHH", 0, 251, 2120, 251, 2120)
    if opcode == 0x203A:  # LE Read Maximum Advertising Data Length
        return struct.pack("<BH", 0, 1650)
    if opcode == 0x203B:  # LE Read Number of Supported Advertising Sets
        return success + b"\x01"
    if opcode == 0x204A:  # LE Read Periodic Advertiser List Size
        return success + b"\x00"
    if opcode == 0x2060:  # LE Read Buffer Size v2
        return struct.pack("<BHBHB", 0, 251, 16, 0, 0)
    # Configuration commands are acknowledged so the Android stack can finish
    # controller bring-up. No radio procedures or peer connections are created.
    return success


def command_complete(opcode: int, params: bytes = b"") -> bytes:
    result = command_return_parameters(opcode, params)
    payload = b"\x01" + struct.pack("<H", opcode) + result
    return bytes((H4_EVENT, 0x0E, len(payload))) + payload


class HciHandler(socketserver.BaseRequestHandler):
    def handle(self) -> None:
        buffer = bytearray()
        while True:
            data = self.request.recv(4096)
            if not data:
                return
            buffer.extend(data)
            while buffer:
                packet_type = buffer[0]
                if packet_type == H4_COMMAND:
                    if len(buffer) < 4:
                        break
                    payload_size = buffer[3]
                    total = 4 + payload_size
                    if len(buffer) < total:
                        break
                    opcode = struct.unpack_from("<H", buffer, 1)[0]
                    params = bytes(buffer[4:total])
                    del buffer[:total]
                    self.request.sendall(command_complete(opcode, params))
                elif packet_type == H4_ACL:
                    if len(buffer) < 5:
                        break
                    total = 5 + struct.unpack_from("<H", buffer, 3)[0]
                    if len(buffer) < total:
                        break
                    del buffer[:total]
                elif packet_type == H4_SCO:
                    if len(buffer) < 4:
                        break
                    total = 4 + buffer[3]
                    if len(buffer) < total:
                        break
                    del buffer[:total]
                elif packet_type == H4_ISO:
                    if len(buffer) < 5:
                        break
                    total = 5 + (struct.unpack_from("<H", buffer, 3)[0] & 0x3FFF)
                    if len(buffer) < total:
                        break
                    del buffer[:total]
                else:
                    del buffer[0]


class ThreadingTcpServer(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hci-port", "--hci_port", type=int, default=7300)
    parser.add_argument("--test-port", "--test_port", type=int)
    parser.add_argument("--link-port", "--link_port", type=int)
    parser.add_argument("--link-ble-port", "--link_ble_port", type=int)
    args = parser.parse_args()
    server = ThreadingTcpServer(("127.0.0.1", args.hci_port), HciHandler)
    stop = threading.Event()

    def request_stop(_signum: int, _frame: object) -> None:
        stop.set()
        threading.Thread(target=server.shutdown, daemon=True).start()

    signal.signal(signal.SIGINT, request_stop)
    signal.signal(signal.SIGTERM, request_stop)
    print(f"root_canal_stub: H4/TCP 127.0.0.1:{args.hci_port}", flush=True)
    try:
        server.serve_forever(poll_interval=0.2)
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
