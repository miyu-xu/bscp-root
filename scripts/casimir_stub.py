#!/usr/bin/env python3
"""Minimal cross-platform NFC NCI 2.0 controller stub.

The stub supports controller initialization and configuration but intentionally
does not synthesize tags or peer RF traffic.
"""

from __future__ import annotations

import argparse
import signal
import socketserver
import struct
import threading

MT_COMMAND = 1
MT_RESPONSE = 2
MT_NOTIFICATION = 3
GID_CORE = 0
GID_RF = 1
GID_NFCEE = 2


def control_packet(mt: int, gid: int, oid: int, payload: bytes) -> bytes:
    if len(payload) > 255:
        raise ValueError("NCI control payload exceeds one packet")
    return bytes(((mt << 5) | (gid & 0x0F), oid & 0x3F, len(payload))) + payload


def response_payload(gid: int, oid: int, payload: bytes) -> bytes:
    if gid == GID_CORE and oid == 0x00:  # CORE_RESET_RSP (NCI 2.0)
        return b"\x00"
    if gid == GID_CORE and oid == 0x01:  # CORE_INIT_RSP (NCI 2.0)
        return (
            b"\x00"                  # status
            + struct.pack("<I", 0)    # NFCC features
            + b"\x01"                # max logical connections
            + struct.pack("<H", 0)    # routing table size
            + b"\xFF"                # max control payload
            + b"\xFF"                # max data payload
            + b"\x01"                # initial data credits
            + struct.pack("<H", 255)  # max NFC-V frame
            + b"\x01\x00\x00"        # one FRAME RF interface, no extensions
        )
    if gid == GID_CORE and oid == 0x02:  # CORE_SET_CONFIG_RSP
        return b"\x00\x00"
    if gid == GID_CORE and oid == 0x03:  # CORE_GET_CONFIG_RSP
        return b"\x00\x00"
    if gid == GID_CORE and oid == 0x04:  # CORE_CONN_CREATE_RSP
        return b"\x00\xFF\x01\x01"
    if gid == GID_CORE and oid == 0x05:  # CORE_CONN_CLOSE_RSP
        return b"\x00"
    if gid == GID_RF and oid == 0x02:  # RF_GET_LISTEN_MODE_ROUTING_RSP
        return b"\x00\x00"
    if gid == GID_NFCEE and oid == 0x00:  # NFCEE_DISCOVER_RSP
        return b"\x00\x00"
    # RF mapping/routing/discovery/deactivation and vendor initialization are
    # acknowledged. The stub never emits an RF discovery notification.
    return b"\x00"


def replies_for_command(gid: int, oid: int, payload: bytes) -> list[bytes]:
    replies = [control_packet(MT_RESPONSE, gid, oid, response_payload(gid, oid, payload))]
    if gid == GID_CORE and oid == 0x00:
        # reset trigger, config status, NCI version, manufacturer, info length
        reset_ntf = b"\x02\x00\x20\xE0\x00"
        replies.append(control_packet(MT_NOTIFICATION, GID_CORE, 0x00, reset_ntf))
    elif gid == GID_RF and oid == 0x06:
        deactivate_type = payload[:1] or b"\x00"
        replies.append(
            control_packet(MT_NOTIFICATION, GID_RF, 0x06, deactivate_type + b"\x00")
        )
    return replies


class NciHandler(socketserver.BaseRequestHandler):
    def handle(self) -> None:
        buffer = bytearray()
        while True:
            data = self.request.recv(4096)
            if not data:
                return
            buffer.extend(data)
            while len(buffer) >= 3:
                size = buffer[2]
                total = 3 + size
                if len(buffer) < total:
                    break
                header0, header1 = buffer[0], buffer[1]
                payload = bytes(buffer[3:total])
                del buffer[:total]
                mt = (header0 >> 5) & 0x07
                gid = header0 & 0x0F
                oid = header1 & 0x3F
                if mt != MT_COMMAND:
                    continue
                for reply in replies_for_command(gid, oid, payload):
                    self.request.sendall(reply)


class DiscardHandler(socketserver.BaseRequestHandler):
    def handle(self) -> None:
        while self.request.recv(4096):
            pass


class ThreadingTcpServer(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--nci-port", type=int, default=7800)
    parser.add_argument("--rf-port", type=int, default=7900)
    args = parser.parse_args()
    nci = ThreadingTcpServer(("127.0.0.1", args.nci_port), NciHandler)
    rf = ThreadingTcpServer(("127.0.0.1", args.rf_port), DiscardHandler)

    def request_stop(_signum: int, _frame: object) -> None:
        threading.Thread(target=nci.shutdown, daemon=True).start()
        threading.Thread(target=rf.shutdown, daemon=True).start()

    signal.signal(signal.SIGINT, request_stop)
    signal.signal(signal.SIGTERM, request_stop)
    threading.Thread(target=rf.serve_forever, kwargs={"poll_interval": 0.2}, daemon=True).start()
    print(
        f"casimir_stub: NCI/TCP 127.0.0.1:{args.nci_port}, RF/TCP 127.0.0.1:{args.rf_port}",
        flush=True,
    )
    try:
        nci.serve_forever(poll_interval=0.2)
    finally:
        nci.server_close()
        rf.shutdown()
        rf.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
