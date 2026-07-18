#!/usr/bin/env python3
"""Windows-native Cuttlefish modem simulator over binder_rpc named pipes.

AOSP only ships ``modem_simulator`` for Linux hosts. On Windows, crosvm virtio-vsock
connects guest RIL traffic to ``\\\\.\\pipe\\binder_rpc_vsock_{cid}_{port}``, the
same path used by libbinder RPC. This process implements enough of the Cuttlefish
modem_simulator AT command surface for ``com.google.cf.rild`` / ``com.android.phone``.
"""

from __future__ import annotations

import argparse
import ctypes
import datetime
import re
import struct
import sys
import threading
import time
from dataclasses import dataclass, field
from typing import Callable

PIPE_ACCESS_DUPLEX = 0x00000003
PIPE_TYPE_BYTE = 0x00000000
PIPE_READMODE_BYTE = 0x00000000
PIPE_WAIT = 0x00000000
PIPE_UNLIMITED_INSTANCES = 255
PIPE_HEADER_SIZE = 8
ERROR_PIPE_CONNECTED = 535
ERROR_IO_PENDING = 997
INVALID_HANDLE_VALUE = ctypes.c_void_p(-1).value
KERNEL32 = ctypes.WinDLL("kernel32", use_last_error=True) if sys.platform == "win32" else None


def pack_framed_payload(payload: bytes) -> bytes:
    header = struct.pack("<II", len(payload), 0)
    return header + payload


def unpack_framed_payloads(buffer: bytearray) -> list[bytes]:
    payloads: list[bytes] = []
    while True:
        if len(buffer) < PIPE_HEADER_SIZE:
            break
        payload_size, handle_count = struct.unpack_from("<II", buffer, 0)
        frame_size = PIPE_HEADER_SIZE + (handle_count * 8) + payload_size
        if len(buffer) < frame_size:
            break
        payload_start = PIPE_HEADER_SIZE + (handle_count * 8)
        payload_end = payload_start + payload_size
        payloads.append(bytes(buffer[payload_start:payload_end]))
        del buffer[:frame_size]
    return payloads


NET_REGISTRATION_HOME = 1
M_MODEM_TECH_LTE = 14
K_AREA_CODE = "2142"
K_CELL_ID = "0000B804"
DEFAULT_IMEI = "867400022047199"
DEFAULT_IMSI = "311740010000001"
DEFAULT_ICCID = "89014103211118510720"
DEFAULT_OPERATOR_NUMERIC = "311740"
DEFAULT_OPERATOR_LONG = "Android Virtual Operator"
DEFAULT_OPERATOR_SHORT = "Android"
CME_NOT_SUPPORTED = "+CME ERROR: 4"


def cf_modem_vsock_port(cid: int, base_port: int = 9600) -> int:
    return base_port + (cid - 3)


def binder_rpc_vsock_pipe_path(guest_cid: int, port: int) -> str:
    return rf"\\.\pipe\binder_rpc_vsock_{guest_cid}_{port}"


def normalize_command(command: str) -> str:
    command = command.strip().upper()
    command = command.replace("\n", "")
    if command.startswith("AT"):
        command = command[2:]
    return command


@dataclass
class PdpContext:
    cid: int
    active: bool = True
    ip_type: str = "IP"
    apn: str = ""
    address: str = ""
    gateway: str = ""
    dns: str = ""


@dataclass
class ModemState:
    ril_address: str
    ril_gateway: str
    ril_dns: str
    radio_on: bool = False
    voice_reg: int = NET_REGISTRATION_HOME
    data_reg: int = NET_REGISTRATION_HOME
    creg_mode: int = 0
    cgreg_mode: int = 0
    cereg_mode: int = 0
    pdp_contexts: dict[int, PdpContext] = field(default_factory=dict)

    def on_first_client(self) -> list[str]:
        self.radio_on = True
        lines = [
            f"+CREG: {self.voice_reg}",
            f"+CGREG: {self.data_reg}",
            f"+CEREG: {self.data_reg}",
            self.build_ctzv(),
        ]
        return lines

    def build_ctzv(self) -> str:
        now = datetime.datetime.now(datetime.timezone.utc)
        local = datetime.datetime.now().astimezone()
        offset_quarters = int(local.utcoffset().total_seconds() // (15 * 60)) if local.utcoffset() else 0
        sign = "+" if offset_quarters >= 0 else "-"
        tz = abs(offset_quarters)
        return (
            f"%CTZV: {now.year % 100:02d}/{now.month:02d}/{now.day:02d}:"
            f"{now.hour:02d}:{now.minute:02d}:{now.second:02d}"
            f"{sign}{tz}:{local.dst()}"
        )


class AtModem:
    def __init__(self, state: ModemState) -> None:
        self.state = state
        self._prefix_handlers: list[tuple[str, Callable[[str], list[str]]]] = [
            ("+CFUN=", self._handle_cfun_set),
            ("+CREG", self._handle_creg),
            ("+CGREG", self._handle_cgreg),
            ("+CEREG", self._handle_cereg),
            ("+COPS=", self._handle_cops_set),
            ("+CGDCONT=", self._handle_cgdcont_set),
            ("+CGACT=", self._handle_cgact_set),
            ("+CGCONTRDP", self._handle_cgcontrdp),
            ("+CGDATA", self._handle_cgdata),
            ("+CGSN", self._handle_cgsn),
            ("+CPIN=", self._handle_cpin_set),
            ("+CRSM=", self._handle_ok),
            ("+CSIM=", self._handle_ok),
            ("+CLCK=", self._handle_clck),
            ("+CCHO=", self._handle_ok),
            ("+CCHC=", self._handle_ok),
            ("+CGLA=", self._handle_ok),
            ("+CPWD=", self._handle_ok),
            ("+CPINR=", self._handle_ok),
            ("+CHLD=", self._handle_ok),
            ("+VTS=", self._handle_ok),
            ("+CUSD=", self._handle_ok),
            ("D", self._handle_dial),
        ]
        self._exact_handlers: dict[str, Callable[[], list[str]]] = {
            "": self._handle_ping,
            "+CFUN?": self._handle_cfun_query,
            "+CPIN?": lambda: ["+CPIN: READY", "OK"],
            "+COPS?": self._handle_cops_query,
            "+COPS=3,0;+COPS?;+COPS=3,1;+COPS?;+COPS=3,2;+COPS?": self._handle_cops_request_operator,
            "+COPS=?": self._handle_cops_scan,
            "+CSQ": self._handle_csq,
            "+CIMI": lambda: [DEFAULT_IMSI, "OK"],
            "+CICCID": lambda: [DEFAULT_ICCID, "OK"],
            "+CGDCONT?": self._handle_cgdcont_query,
            "+CGACT?": self._handle_cgact_query,
            "+CGSN": lambda: [DEFAULT_IMEI, "OK"],
            "+CMEE=1": self._handle_ok,
            "+CMOD=0": self._handle_ok,
            "+CSSN=0,1": self._handle_ok,
            "+COLP=0": self._handle_ok,
            "+CSCS=\"HEX\"": self._handle_ok,
            "+CMGF=0": self._handle_ok,
            "+CGQREQ=1": self._handle_ok,
            "+CGQMIN=1": self._handle_ok,
            "+CGEREP=1,0": self._handle_ok,
            "+WSOS=0": self._handle_ok,
            "E0Q0V1": self._handle_ok,
            "S0=0": self._handle_ok,
            "D*99***1#": self._handle_ok,
            "+CTEC?": lambda: [f"+CTEC: {M_MODEM_TECH_LTE}", "OK"],
            "+CTEC=?": lambda: [f"+CTEC: ({M_MODEM_TECH_LTE})", "OK"],
        }

    def dispatch(self, raw_command: str) -> list[str]:
        command = normalize_command(raw_command)
        if not command:
            return ["OK"]

        handler = self._exact_handlers.get(command)
        if handler:
            return handler()

        responses: list[str] = []
        for part in command.split(";"):
            part = part.strip()
            if not part:
                continue
            responses.extend(self._dispatch_one(part))
        return responses

    def _dispatch_one(self, command: str) -> list[str]:
        handler = self._exact_handlers.get(command)
        if handler:
            return handler()

        for prefix, prefix_handler in self._prefix_handlers:
            if command.startswith(prefix):
                return prefix_handler(command)

        if command.endswith("?"):
            return ["OK"]
        if "=" in command:
            return ["OK"]
        return [CME_NOT_SUPPORTED]

    def _handle_ping(self) -> list[str]:
        return ["OK"]

    def _handle_ok(self, _command: str = "") -> list[str]:
        return ["OK"]

    def _handle_cfun_query(self) -> list[str]:
        return [f"+CFUN: {1 if self.state.radio_on else 0}", "OK"]

    def _handle_cfun_set(self, command: str) -> list[str]:
        match = re.search(r"\+CFUN=(\d+)", command)
        if not match:
            return [CME_NOT_SUPPORTED]
        mode = int(match.group(1))
        if mode == 0:
            self.state.radio_on = False
            self.state.voice_reg = 0
            self.state.data_reg = 0
        elif mode == 1:
            self.state.radio_on = True
            self.state.voice_reg = NET_REGISTRATION_HOME
            self.state.data_reg = NET_REGISTRATION_HOME
        else:
            return [CME_NOT_SUPPORTED]
        return ["OK"]

    def _registered_suffix(self) -> str:
        if self.state.voice_reg in (NET_REGISTRATION_HOME, 5):
            return f',"{K_AREA_CODE}","{K_CELL_ID}",1'
        return ""

    def _handle_creg(self, command: str) -> list[str]:
        if command.endswith("?"):
            suffix = self._registered_suffix()
            return [f"+CREG: {self.state.creg_mode},{self.state.voice_reg}{suffix}", "OK"]
        match = re.search(r"\+CREG=(\d+)", command)
        if match:
            self.state.creg_mode = int(match.group(1))
            return ["OK"]
        return [CME_NOT_SUPPORTED]

    def _handle_cgreg(self, command: str) -> list[str]:
        if command.endswith("?"):
            suffix = ""
            if self.state.data_reg in (NET_REGISTRATION_HOME, 5):
                suffix = f',"{K_AREA_CODE}","{K_CELL_ID}",{M_MODEM_TECH_LTE}'
            return [f"+CGREG: {self.state.cgreg_mode},{self.state.data_reg}{suffix}", "OK"]
        match = re.search(r"\+CGREG=(\d+)", command)
        if match:
            self.state.cgreg_mode = int(match.group(1))
            return ["OK"]
        return [CME_NOT_SUPPORTED]

    def _handle_cereg(self, command: str) -> list[str]:
        if command.endswith("?"):
            suffix = ""
            if self.state.data_reg in (NET_REGISTRATION_HOME, 5):
                suffix = f',"{K_AREA_CODE}","{K_CELL_ID}",{M_MODEM_TECH_LTE}'
            return [f"+CEREG: {self.state.cereg_mode},{self.state.data_reg}{suffix}", "OK"]
        match = re.search(r"\+CEREG=(\d+)", command)
        if match:
            self.state.cereg_mode = int(match.group(1))
            return ["OK"]
        return [CME_NOT_SUPPORTED]

    def _handle_cops_query(self) -> list[str]:
        if not self.state.radio_on:
            return ["+COPS: 0,0,0", "OK"]
        return [f"+COPS: 0,2,{DEFAULT_OPERATOR_NUMERIC}", "OK"]

    def _handle_cops_request_operator(self) -> list[str]:
        if not self.state.radio_on:
            return [CME_NOT_SUPPORTED]
        return [
            f"+COPS: 0,0,{DEFAULT_OPERATOR_LONG}",
            f"+COPS: 0,1,{DEFAULT_OPERATOR_SHORT}",
            f"+COPS: 0,2,{DEFAULT_OPERATOR_NUMERIC}",
            "OK",
        ]

    def _handle_cops_scan(self) -> list[str]:
        if not self.state.radio_on:
            return [CME_NOT_SUPPORTED]
        return [
            f'+COPS: (2,"{DEFAULT_OPERATOR_LONG}","{DEFAULT_OPERATOR_SHORT}","{DEFAULT_OPERATOR_NUMERIC}",{M_MODEM_TECH_LTE}),',
            "OK",
        ]

    def _handle_cops_set(self, command: str) -> list[str]:
        if command.startswith("+COPS=3,"):
            return ["OK"]
        return ["OK"]

    def _handle_csq(self) -> list[str]:
        return [
            "+CSQ: 20,99,120,120,120,120,120,120,20,95,10,120,120,120,120,120,120,120,120,120,120",
            "OK",
        ]

    def _handle_cgsn(self, command: str) -> list[str]:
        if command == "+CGSN":
            return [DEFAULT_IMEI, "OK"]
        return [DEFAULT_IMEI, "OK"]

    def _handle_cpin_set(self, _command: str) -> list[str]:
        return ["OK"]

    def _handle_clck(self, command: str) -> list[str]:
        fields = command.split(",")
        if len(fields) >= 2 and fields[1].strip() == "2":
            return ["+CLCK: 0", "OK"]
        return ["OK"]

    def _handle_cgdcont_set(self, command: str) -> list[str]:
        match = re.match(r"\+CGDCONT=(\d+),([^,]*),([^,]*)", command)
        if not match:
            return [CME_NOT_SUPPORTED]
        cid = int(match.group(1))
        ip_type = match.group(2).strip('"')
        apn = match.group(3).strip('"')
        self.state.pdp_contexts[cid] = PdpContext(
            cid=cid,
            ip_type=ip_type or "IP",
            apn=apn,
            address=self.state.ril_address,
            gateway=self.state.ril_gateway,
            dns=self.state.ril_dns,
        )
        return ["OK"]

    def _handle_cgdcont_query(self) -> list[str]:
        lines = []
        for ctx in self.state.pdp_contexts.values():
            lines.append(f"+CGDCONT: {ctx.cid},{ctx.ip_type},{ctx.apn},{ctx.address},0,0")
        lines.append("OK")
        return lines

    def _handle_cgact_set(self, _command: str) -> list[str]:
        return ["OK"]

    def _handle_cgact_query(self) -> list[str]:
        lines = []
        for ctx in self.state.pdp_contexts.values():
            if ctx.active:
                lines.append(f"+CGACT: {ctx.cid},1")
        lines.append("OK")
        return lines

    def _handle_cgcontrdp(self, command: str) -> list[str]:
        match = re.search(r"\+CGCONTRDP=(\d+)", command)
        if not match:
            return [CME_NOT_SUPPORTED]
        cid = int(match.group(1))
        ctx = self.state.pdp_contexts.get(cid)
        if not ctx or not ctx.active:
            return ["+CME ERROR: 21"]
        return [
            f"+CGCONTRDP: {ctx.cid},5,{ctx.apn},{ctx.address},{ctx.gateway},{ctx.dns}",
            "OK",
        ]

    def _handle_cgdata(self, command: str) -> list[str]:
        match = re.search(r"\+CGDATA=[^,]*,(\d+)", command)
        if match and int(match.group(1)) in self.state.pdp_contexts:
            return ["CONNECT"]
        return ["ERROR"]

    def _handle_dial(self, command: str) -> list[str]:
        if command.startswith("D") and not command.startswith("D*99"):
            return ["OK"]
        return ["OK"]


class WindowsPipe:
    def __init__(self, handle: int) -> None:
        self._kernel32 = KERNEL32
        self.handle = handle

    def close(self) -> None:
        if self.handle not in (0, INVALID_HANDLE_VALUE):
            self._kernel32.CloseHandle(self.handle)
            self.handle = INVALID_HANDLE_VALUE

    def read(self, size: int = 4096) -> bytes:
        buf = ctypes.create_string_buffer(size)
        read = ctypes.c_uint32(0)
        ok = self._kernel32.ReadFile(self.handle, buf, size, ctypes.byref(read), None)
        if not ok:
            err = ctypes.get_last_error()
            if err == 109:  # ERROR_BROKEN_PIPE
                return b""
            raise OSError(f"ReadFile failed: winerr={err}")
        return buf.raw[: read.value]

    def write(self, data: bytes) -> None:
        if not data:
            return
        written = ctypes.c_uint32(0)
        ok = self._kernel32.WriteFile(
            self.handle, data, len(data), ctypes.byref(written), None
        )
        if not ok:
            raise OSError(f"WriteFile failed: winerr={ctypes.get_last_error()}")


def create_pipe_server(path: str) -> int:
    kernel32 = KERNEL32
    handle = kernel32.CreateNamedPipeW(
        path,
        PIPE_ACCESS_DUPLEX,
        PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
        PIPE_UNLIMITED_INSTANCES,
        4096,
        4096,
        0,
        None,
    )
    if handle == INVALID_HANDLE_VALUE:
        raise OSError(f"CreateNamedPipeW failed for {path}: winerr={ctypes.get_last_error()}")
    return handle


def accept_pipe(handle: int) -> int:
    kernel32 = KERNEL32
    connected = kernel32.ConnectNamedPipe(handle, None)
    if not connected:
        err = ctypes.get_last_error()
        if err not in (ERROR_PIPE_CONNECTED, ERROR_IO_PENDING):
            kernel32.CloseHandle(handle)
            raise OSError(f"ConnectNamedPipe failed: winerr={err}")
    return handle


def send_lines(pipe: WindowsPipe, lines: list[str], *, framed: bool = True) -> None:
    for line in lines:
        if not line:
            continue
        payload = line if line.endswith("\r") else f"{line}\r"
        data = payload.encode("ascii", errors="replace")
        pipe.write(pack_framed_payload(data) if framed else data)


_first_client_lock = threading.Lock()
_first_client_sent = False


def handle_client(handle: int, modem: AtModem) -> None:
    global _first_client_sent
    pipe = WindowsPipe(handle)
    rx_buffer = bytearray()
    at_buffer = ""
    try:
        with _first_client_lock:
            if not _first_client_sent:
                send_lines(pipe, modem.state.on_first_client())
                _first_client_sent = True
                print("modem_simulator_windows: first RIL client connected", flush=True)

        while True:
            chunk = pipe.read()
            if not chunk:
                break
            rx_buffer.extend(chunk)
            for payload in unpack_framed_payloads(rx_buffer):
                at_buffer += payload.decode("ascii", errors="replace").replace("\n", "\r")
                while "\r" in at_buffer:
                    command, at_buffer = at_buffer.split("\r", 1)
                    command = command.strip()
                    if not command:
                        continue
                    responses = modem.dispatch(command)
                    send_lines(pipe, responses)
    finally:
        pipe.close()


def serve_namedpipe_vsock(
    guest_cid: int,
    port: int,
    modem: AtModem,
    stop_event: threading.Event,
) -> None:
    path = binder_rpc_vsock_pipe_path(guest_cid, port)
    print(f"modem_simulator_windows: listening on {path} (guest vsock host port {port})", flush=True)
    while not stop_event.is_set():
        handle = create_pipe_server(path)
        try:
            accept_pipe(handle)
        except OSError as exc:
            print(f"modem_simulator_windows: accept failed: {exc}", file=sys.stderr, flush=True)
            time.sleep(0.5)
            continue
        threading.Thread(
            target=handle_client,
            args=(handle, modem),
            name="modem-ril-client",
            daemon=True,
        ).start()


def main() -> int:
    if sys.platform != "win32":
        print("modem_simulator_windows.py is Windows-only", file=sys.stderr)
        return 2

    parser = argparse.ArgumentParser()
    parser.add_argument("--guest-cid", type=int, required=True)
    parser.add_argument("--base-port", type=int, default=9600)
    parser.add_argument("--ril-gateway", default="192.168.97.1")
    parser.add_argument("--ril-ipaddr", default="192.168.97.2")
    parser.add_argument("--ril-prefixlen", type=int, default=30)
    parser.add_argument("--ril-dns", default="8.8.8.8")
    args = parser.parse_args()

    port = cf_modem_vsock_port(args.guest_cid, args.base_port)
    state = ModemState(
        ril_address=f"{args.ril_ipaddr}/{args.ril_prefixlen}",
        ril_gateway=args.ril_gateway,
        ril_dns=args.ril_dns,
    )
    modem = AtModem(state)
    stop_event = threading.Event()
    try:
        serve_namedpipe_vsock(args.guest_cid, port, modem, stop_event)
    except KeyboardInterrupt:
        stop_event.set()
    return 0


if __name__ == "__main__":
    sys.exit(main())
