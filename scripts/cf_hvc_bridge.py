#!/usr/bin/env python3
"""Cross-platform duplex bridge between a Cuttlefish HVC and a TCP service.

Unix runners expose HVC output as a FIFO or growing regular file and HVC input
as a FIFO.  The Windows runner exposes both directions as named pipes.  This
tool deliberately has no third-party dependencies so it can be packaged with
the direct runners.
"""

from __future__ import annotations

import argparse
import ctypes
import errno
import os
import socket
import stat
import sys
import threading
import time
from pathlib import Path
from typing import Optional


class Stream:
    def read(self, size: int = 65536) -> bytes:
        raise NotImplementedError

    def write(self, data: bytes) -> None:
        raise NotImplementedError

    def close(self) -> None:
        raise NotImplementedError


class SocketStream(Stream):
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


class UnixReadStream(Stream):
    """Read a FIFO or follow a regular file while crosvm appends to it."""

    def __init__(self, path: str, stop: threading.Event) -> None:
        self.path = path
        self.stop = stop
        self.fd: Optional[int] = None
        self.regular = False
        self._open_when_ready()

    def _open_when_ready(self) -> None:
        while not self.stop.is_set():
            try:
                mode = os.stat(self.path).st_mode
                self.regular = stat.S_ISREG(mode)
                flags = os.O_RDONLY | (0 if self.regular else os.O_NONBLOCK)
                self.fd = os.open(self.path, flags)
                return
            except FileNotFoundError:
                time.sleep(0.05)
        raise InterruptedError("stopped while waiting for HVC output")

    def read(self, size: int = 65536) -> bytes:
        assert self.fd is not None
        while not self.stop.is_set():
            try:
                data = os.read(self.fd, size)
                if data:
                    return data
                # EOF on a growing file is temporary.  A FIFO can also report
                # an empty read while the crosvm side is reconnecting.
                time.sleep(0.02)
            except BlockingIOError:
                time.sleep(0.02)
            except InterruptedError:
                continue
        return b""

    def write(self, data: bytes) -> None:
        raise OSError(errno.EBADF, "read-only HVC stream")

    def close(self) -> None:
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None


class UnixWriteStream(Stream):
    def __init__(self, path: str, stop: threading.Event) -> None:
        self.path = path
        self.stop = stop
        self.fd: Optional[int] = None
        self._open_when_ready()

    def _open_when_ready(self) -> None:
        while not self.stop.is_set():
            try:
                mode = os.stat(self.path).st_mode
                flags = os.O_RDWR if stat.S_ISFIFO(mode) else os.O_WRONLY
                self.fd = os.open(self.path, flags | os.O_NONBLOCK)
                return
            except (FileNotFoundError, OSError) as exc:
                if isinstance(exc, OSError) and exc.errno not in (
                    errno.ENOENT,
                    errno.ENXIO,
                    errno.EACCES,
                ):
                    raise
                time.sleep(0.05)
        raise InterruptedError("stopped while waiting for HVC input")

    def read(self, size: int = 65536) -> bytes:
        raise OSError(errno.EBADF, "write-only HVC stream")

    def write(self, data: bytes) -> None:
        assert self.fd is not None
        view = memoryview(data)
        while view and not self.stop.is_set():
            try:
                written = os.write(self.fd, view)
                view = view[written:]
            except BlockingIOError:
                time.sleep(0.01)

    def close(self) -> None:
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None


class WindowsNamedPipeStream(Stream):
    """Thin adapter around the existing tested Windows modem pipe code."""

    def __init__(self, handle: int) -> None:
        from modem_simulator_windows import WindowsPipe

        self.pipe = WindowsPipe(handle)

    def read(self, size: int = 65536) -> bytes:
        return self.pipe.read(size)

    def write(self, data: bytes) -> None:
        self.pipe.write(data)

    def close(self) -> None:
        self.pipe.close()


def connect_windows_pipe(path: str, stop: threading.Event) -> int:
    """Connect to the named-pipe server created by crosvm."""
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel32.CreateFileW.argtypes = (
        ctypes.c_wchar_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.c_void_p,
        ctypes.c_uint32,
        ctypes.c_uint32,
        ctypes.c_void_p,
    )
    kernel32.CreateFileW.restype = ctypes.c_void_p

    generic_read_write = 0x80000000 | 0x40000000
    open_existing = 3
    invalid_handle = ctypes.c_void_p(-1).value
    retryable_errors = {2, 231}  # ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY

    while not stop.is_set():
        handle = kernel32.CreateFileW(
            path,
            generic_read_write,
            0,
            None,
            open_existing,
            0,
            None,
        )
        if handle != invalid_handle:
            return int(handle)
        error = ctypes.get_last_error()
        if error not in retryable_errors:
            raise OSError(f"CreateFileW failed for {path}: winerr={error}")
        time.sleep(0.05)
    raise InterruptedError(f"stopped while connecting to {path}")


def open_hvc_streams(
    guest_out: str, guest_in: str, stop: threading.Event
) -> tuple[Stream, Stream]:
    """Return (from_guest, to_guest), waiting for both endpoints."""
    if sys.platform != "win32":
        return UnixReadStream(guest_out, stop), UnixWriteStream(guest_in, stop)

    # crosvm owns both server endpoints. Connect concurrently because it creates
    # them in sequence while bringing up the virtio-console device.
    handles: dict[str, int] = {}
    errors: list[BaseException] = []

    def connect(name: str, path: str) -> None:
        try:
            handles[name] = connect_windows_pipe(path, stop)
        except BaseException as exc:  # propagate after both join
            errors.append(exc)

    threads = [
        threading.Thread(target=connect, args=("out", guest_out)),
        threading.Thread(target=connect, args=("in", guest_in)),
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    if errors:
        raise errors[0]
    return WindowsNamedPipeStream(handles["out"]), WindowsNamedPipeStream(handles["in"])


def connect_tcp(host: str, port: int, stop: threading.Event) -> SocketStream:
    while not stop.is_set():
        try:
            sock = socket.create_connection((host, port), timeout=1.0)
            sock.settimeout(None)
            return SocketStream(sock)
        except OSError:
            time.sleep(0.2)
    raise InterruptedError("stopped while connecting TCP backend")


def pump(source: Stream, destination: Stream, stop: threading.Event) -> None:
    try:
        while not stop.is_set():
            data = source.read()
            if not data:
                break
            destination.write(data)
    except (BrokenPipeError, ConnectionError, OSError) as exc:
        if not stop.is_set():
            print(f"cf_hvc_bridge: pump stopped: {exc}", file=sys.stderr)
    finally:
        stop.set()


def bridge_once(args: argparse.Namespace, stop: threading.Event) -> None:
    from_guest, to_guest = open_hvc_streams(args.guest_out, args.guest_in, stop)
    tcp = connect_tcp(args.tcp_host, args.tcp_port, stop)
    print(
        f"cf_hvc_bridge: {args.guest_out}/{args.guest_in} <-> "
        f"{args.tcp_host}:{args.tcp_port}",
        flush=True,
    )
    local_stop = threading.Event()
    threads = [
        threading.Thread(
            target=pump,
            args=(from_guest, tcp, local_stop),
            daemon=True,
        ),
        threading.Thread(
            target=pump,
            args=(tcp, to_guest, local_stop),
            daemon=True,
        ),
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    for stream in (from_guest, to_guest, tcp):
        stream.close()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--guest-out", required=True, help="guest-to-host HVC path")
    parser.add_argument("--guest-in", required=True, help="host-to-guest HVC path")
    parser.add_argument("--tcp-host", default="127.0.0.1")
    parser.add_argument("--tcp-port", type=int, required=True)
    parser.add_argument("--reconnect", action="store_true")
    args = parser.parse_args()

    stop = threading.Event()
    try:
        while not stop.is_set():
            bridge_once(args, stop)
            if not args.reconnect:
                break
            stop.clear()
            time.sleep(0.2)
    except KeyboardInterrupt:
        stop.set()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
