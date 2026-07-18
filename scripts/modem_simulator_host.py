#!/usr/bin/env python3
"""Portable Cuttlefish AT modem using Linux vsock, macOS UDS, or Windows pipe."""

from __future__ import annotations

import argparse
import os
import signal
import socket
import sys
import threading
import time

from modem_simulator_windows import (
    AtModem,
    ModemState,
    cf_modem_vsock_port,
    pack_framed_payload,
    send_lines,
    unpack_framed_payloads,
)


class SocketPipe:
    def __init__(self, sock: socket.socket) -> None:
        self.sock = sock

    def read(self, size: int = 65536) -> bytes:
        return self.sock.recv(size)

    def write(self, data: bytes) -> None:
        self.sock.sendall(data)

    def close(self) -> None:
        try:
            self.sock.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass
        self.sock.close()


def handle_socket_client(sock: socket.socket, modem: AtModem, first: threading.Event) -> None:
    pipe = SocketPipe(sock)
    rx_buffer = bytearray()
    at_buffer = ""
    try:
        if not first.is_set():
            send_lines(pipe, modem.state.on_first_client())
            first.set()
            print("modem_simulator_host: first RIL client connected", flush=True)
        while True:
            chunk = pipe.read()
            if not chunk:
                return
            rx_buffer.extend(chunk)
            for payload in unpack_framed_payloads(rx_buffer):
                at_buffer += payload.decode("ascii", errors="replace").replace("\n", "\r")
                while "\r" in at_buffer:
                    command, at_buffer = at_buffer.split("\r", 1)
                    command = command.strip()
                    if command:
                        send_lines(pipe, modem.dispatch(command))
    finally:
        pipe.close()


def unix_listener(guest_cid: int, port: int) -> tuple[socket.socket, str]:
    if sys.platform == "darwin":
        path = f"/tmp/binder_rpc_vsock_{guest_cid}_{port}.sock"
        try:
            os.unlink(path)
        except FileNotFoundError:
            pass
        listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        listener.bind(path)
        listener.listen(8)
        return listener, path

    if not hasattr(socket, "AF_VSOCK"):
        raise RuntimeError("this Python does not expose AF_VSOCK")
    listener = socket.socket(socket.AF_VSOCK, socket.SOCK_STREAM)
    listener.bind((getattr(socket, "VMADDR_CID_ANY", 0xFFFFFFFF), port))
    listener.listen(8)
    return listener, f"vsock:any:{port}"


def main() -> int:
    if sys.platform == "win32":
        from modem_simulator_windows import main as windows_main

        return windows_main()

    parser = argparse.ArgumentParser()
    parser.add_argument("--guest-cid", type=int, required=True)
    parser.add_argument("--base-port", type=int, default=9600)
    parser.add_argument("--ril-gateway", default="192.168.97.1")
    parser.add_argument("--ril-ipaddr", default="192.168.97.2")
    parser.add_argument("--ril-prefixlen", type=int, default=30)
    parser.add_argument("--ril-dns", default="8.8.8.8")
    args = parser.parse_args()
    port = cf_modem_vsock_port(args.guest_cid, args.base_port)
    modem = AtModem(
        ModemState(
            ril_address=f"{args.ril_ipaddr}/{args.ril_prefixlen}",
            ril_gateway=args.ril_gateway,
            ril_dns=args.ril_dns,
        )
    )
    listener, endpoint = unix_listener(args.guest_cid, port)
    first = threading.Event()
    stopping = threading.Event()

    def request_stop(_signum: int, _frame: object) -> None:
        stopping.set()
        listener.close()

    signal.signal(signal.SIGTERM, request_stop)
    signal.signal(signal.SIGINT, request_stop)
    print(f"modem_simulator_host: listening on {endpoint}", flush=True)
    try:
        while not stopping.is_set():
            client, _ = listener.accept()
            threading.Thread(
                target=handle_socket_client,
                args=(client, modem, first),
                daemon=True,
            ).start()
    except (KeyboardInterrupt, OSError):
        return 0
    finally:
        listener.close()
        if sys.platform == "darwin":
            try:
                os.unlink(endpoint)
            except FileNotFoundError:
                pass


if __name__ == "__main__":
    raise SystemExit(main())
