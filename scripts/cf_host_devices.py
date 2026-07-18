#!/usr/bin/env python3
"""Own and supervise Cuttlefish host-side device services."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path
from typing import IO, Optional


def executable_names(name: str) -> list[str]:
    if sys.platform == "win32":
        return [name + ".exe", name]
    return [name]


def resolve_binary(name: str, search_dirs: list[Path]) -> Optional[str]:
    for directory in search_dirs:
        for candidate_name in executable_names(name):
            candidate = directory / candidate_name
            if candidate.is_file():
                return str(candidate.resolve())
    return shutil.which(name)


def wait_tcp(port: int, timeout: float = 10.0) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.25):
                return True
        except OSError:
            time.sleep(0.1)
    return False


class Supervisor:
    def __init__(self, log_dir: Path) -> None:
        self.log_dir = log_dir
        self.log_dir.mkdir(parents=True, exist_ok=True)
        self.processes: list[subprocess.Popen[bytes]] = []
        self.logs: list[IO[bytes]] = []
        self.stopping = False

    def start(self, name: str, argv: list[str]) -> subprocess.Popen[bytes]:
        stdout = open(self.log_dir / f"{name}.stdout.txt", "wb")
        stderr = open(self.log_dir / f"{name}.stderr.txt", "wb")
        self.logs.extend((stdout, stderr))
        process = subprocess.Popen(argv, stdout=stdout, stderr=stderr)
        self.processes.append(process)
        print(f"cf_host_devices: started {name} pid={process.pid}", flush=True)
        return process

    def stop(self) -> None:
        if self.stopping:
            return
        self.stopping = True
        for process in reversed(self.processes):
            if process.poll() is None:
                process.terminate()
        deadline = time.monotonic() + 5.0
        for process in reversed(self.processes):
            remaining = max(0.0, deadline - time.monotonic())
            try:
                process.wait(timeout=remaining)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        for stream in self.logs:
            stream.close()


def add_common_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--work-dir", required=True)
    parser.add_argument("--log-dir", required=True)
    parser.add_argument("--host-bin-dir", action="append", default=[])
    parser.add_argument("--guest-cid", type=int, default=100)
    parser.add_argument("--bt-out")
    parser.add_argument("--bt-in")
    parser.add_argument("--nfc-out")
    parser.add_argument("--nfc-in")
    parser.add_argument("--sensors-out")
    parser.add_argument("--sensors-in")
    parser.add_argument("--bt-hci-port", type=int, default=7300)
    parser.add_argument("--casimir-nci-port", type=int, default=7800)
    parser.add_argument("--casimir-rf-port", type=int, default=7900)
    parser.add_argument("--modem-base-port", type=int, default=9600)
    parser.add_argument("--bluetooth", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--nfc", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--modem", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--sensors", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--require-native", action="store_true")


def validate_hvc(args: argparse.Namespace, prefix: str) -> bool:
    return bool(getattr(args, prefix + "_out") and getattr(args, prefix + "_in"))


def build_status(args: argparse.Namespace) -> dict[str, object]:
    script_dir = Path(__file__).resolve().parent
    search_dirs = [Path(p) for p in args.host_bin_dir if p]
    search_dirs.extend((script_dir, script_dir.parent))
    return {
        "platform": sys.platform,
        "root_canal": resolve_binary("root-canal", search_dirs),
        "casimir": resolve_binary("casimir", search_dirs),
        "root_canal_stub": str(script_dir / "root_canal_stub.py"),
        "casimir_stub": str(script_dir / "casimir_stub.py"),
        "hvc_bridge": str(script_dir / "cf_hvc_bridge.py"),
        "modem_host": str(script_dir / "modem_simulator_host.py"),
        "sensors_host": str(script_dir / "sensors_simulator_host.py"),
    }


def run(args: argparse.Namespace) -> int:
    script_dir = Path(__file__).resolve().parent
    status = build_status(args)
    if args.probe:
        print(json.dumps(status, indent=2, sort_keys=True))
        return 0

    supervisor = Supervisor(Path(args.log_dir))
    python = sys.executable
    active = {"bluetooth": False, "nfc": False, "modem": False, "sensors": False}
    backends = {"bluetooth": "disabled", "nfc": "disabled", "modem": "disabled", "sensors": "disabled"}

    def missing(device: str, engine: str) -> None:
        message = f"{device} disabled: {engine} not found"
        if args.require_native:
            raise RuntimeError(message)
        print(f"cf_host_devices: warning: {message}", file=sys.stderr)

    try:
        if args.bluetooth:
            if not validate_hvc(args, "bt"):
                missing("Bluetooth", "HVC endpoints")
            else:
                if status["root_canal"]:
                    root_name = "root-canal"
                    root_argv = [
                        str(status["root_canal"]),
                        f"--test_port={args.bt_hci_port + 1}",
                        f"--hci_port={args.bt_hci_port}",
                        f"--link_port={args.bt_hci_port + 2}",
                        f"--link_ble_port={args.bt_hci_port + 3}",
                    ]
                    backends["bluetooth"] = "native"
                elif args.require_native:
                    missing("Bluetooth", "root-canal")
                    root_argv = []
                    root_name = "root-canal"
                else:
                    root_name = "root-canal-stub"
                    root_argv = [
                        python,
                        str(status["root_canal_stub"]),
                        "--hci-port",
                        str(args.bt_hci_port),
                    ]
                    backends["bluetooth"] = "stub"
                supervisor.start(
                    root_name,
                    root_argv,
                )
                if not wait_tcp(args.bt_hci_port):
                    raise RuntimeError("root-canal HCI port did not become ready")
                supervisor.start(
                    "bt-hvc-bridge",
                    [python, str(status["hvc_bridge"]), "--guest-out", args.bt_out,
                     "--guest-in", args.bt_in, "--tcp-port", str(args.bt_hci_port),
                     "--reconnect"],
                )
                active["bluetooth"] = True

        if args.nfc:
            if not validate_hvc(args, "nfc"):
                missing("NFC", "HVC endpoints")
            else:
                if status["casimir"]:
                    casimir_name = "casimir"
                    casimir_argv = [
                        str(status["casimir"]),
                        "--nci-port",
                        str(args.casimir_nci_port),
                        "--rf-port",
                        str(args.casimir_rf_port),
                    ]
                    backends["nfc"] = "native"
                elif args.require_native:
                    missing("NFC", "casimir")
                    casimir_name = "casimir"
                    casimir_argv = []
                else:
                    casimir_name = "casimir-stub"
                    casimir_argv = [
                        python,
                        str(status["casimir_stub"]),
                        "--nci-port",
                        str(args.casimir_nci_port),
                        "--rf-port",
                        str(args.casimir_rf_port),
                    ]
                    backends["nfc"] = "stub"
                supervisor.start(
                    casimir_name,
                    casimir_argv,
                )
                if not wait_tcp(args.casimir_nci_port):
                    raise RuntimeError("Casimir NCI port did not become ready")
                supervisor.start(
                    "nfc-hvc-bridge",
                    [python, str(status["hvc_bridge"]), "--guest-out", args.nfc_out,
                     "--guest-in", args.nfc_in, "--tcp-port", str(args.casimir_nci_port),
                     "--reconnect"],
                )
                active["nfc"] = True

        if args.modem:
            supervisor.start(
                "modem-simulator",
                [python, str(status["modem_host"]), "--guest-cid", str(args.guest_cid),
                 "--base-port", str(args.modem_base_port)],
            )
            active["modem"] = True
            backends["modem"] = "stub"

        if args.sensors:
            if not validate_hvc(args, "sensors"):
                missing("Sensors", "HVC endpoints")
            else:
                supervisor.start(
                    "sensors-simulator",
                    [python, str(status["sensors_host"]), "--guest-out", args.sensors_out,
                     "--guest-in", args.sensors_in],
                )
                active["sensors"] = True
                backends["sensors"] = "stub"

        time.sleep(0.1)
        for process in supervisor.processes:
            rc = process.poll()
            if rc is not None:
                raise RuntimeError(
                    f"host child pid={process.pid} exited before readiness with {rc}"
                )

        status["enabled"] = active
        status["backends"] = backends
        status["ports"] = {
            "bluetooth_hci": args.bt_hci_port,
            "nfc_nci": args.casimir_nci_port,
            "nfc_rf": args.casimir_rf_port,
            "modem_base": args.modem_base_port,
        }
        status["processes"] = [process.pid for process in supervisor.processes]
        ready = Path(args.work_dir) / "host-devices-ready.json"
        ready.write_text(json.dumps(status, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        def request_stop(_signum: int, _frame: object) -> None:
            supervisor.stop()

        signal.signal(signal.SIGTERM, request_stop)
        signal.signal(signal.SIGINT, request_stop)
        while not supervisor.stopping:
            for process in supervisor.processes:
                rc = process.poll()
                if rc is not None:
                    raise RuntimeError(f"host child pid={process.pid} exited unexpectedly with {rc}")
            time.sleep(0.25)
        return 0
    finally:
        supervisor.stop()
        if sys.platform == "darwin":
            port = args.modem_base_port + args.guest_cid - 3
            socket_path = Path(f"/tmp/binder_rpc_vsock_{args.guest_cid}_{port}.sock")
            try:
                socket_path.unlink()
            except FileNotFoundError:
                pass


def main() -> int:
    parser = argparse.ArgumentParser()
    add_common_args(parser)
    parser.add_argument("--probe", action="store_true")
    args = parser.parse_args()
    try:
        return run(args)
    except RuntimeError as exc:
        print(f"cf_host_devices: error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
