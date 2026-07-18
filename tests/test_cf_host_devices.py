#!/usr/bin/env python3

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIR))

import cf_hvc_bridge
import casimir_stub
import root_canal_stub
import sensors_simulator_host as sensors
from modem_simulator_windows import AtModem, ModemState, pack_framed_payload, unpack_framed_payloads


class ProtocolTests(unittest.TestCase):
    def test_modem_framing_round_trip(self) -> None:
        buffer = bytearray(pack_framed_payload(b"AT+CFUN?\r"))
        self.assertEqual(unpack_framed_payloads(buffer), [b"AT+CFUN?\r"])
        self.assertEqual(buffer, b"")

    def test_modem_basic_registration(self) -> None:
        modem = AtModem(ModemState("192.168.97.2/30", "192.168.97.1", "8.8.8.8"))
        self.assertIn("OK", modem.dispatch("AT+CFUN?"))
        self.assertTrue(any(line.startswith("+CREG:") for line in modem.dispatch("AT+CREG?")))

    def test_sensor_framing_round_trip(self) -> None:
        encoded = sensors.encode_message(2, b"list-sensors\n", response=False)
        buffer = bytearray(encoded)
        self.assertEqual(sensors.decode_messages(buffer), [(2, False, b"list-sensors\n")])
        self.assertEqual(buffer, b"")

    def test_sensor_baseline(self) -> None:
        self.assertEqual(sensors.HOST_SENSOR_MASK, 7)
        reports = b"".join(sensors.sensor_reports())
        self.assertIn(b"acceleration:0:9.80665:0", reports)
        self.assertIn(b"gyroscope:0:0:0", reports)

    def test_bluetooth_stub_hci_reset_and_identity(self) -> None:
        reset = root_canal_stub.command_complete(0x0C03)
        self.assertEqual(reset, bytes.fromhex("040e0401030c00"))
        version = root_canal_stub.command_complete(0x1001)
        self.assertEqual(version[:6], bytes.fromhex("040e0c010110"))
        self.assertEqual(version[6], 0)

    def test_nfc_stub_nci20_reset_and_init(self) -> None:
        reset = casimir_stub.replies_for_command(0, 0, b"\x00")
        self.assertEqual(reset[0], bytes.fromhex("40000100"))
        self.assertEqual(reset[1][:3], bytes.fromhex("600005"))
        self.assertEqual(reset[1][5], 0x20)
        init = casimir_stub.replies_for_command(0, 1, b"\x00\x00")
        self.assertEqual(init[0][:2], bytes.fromhex("4001"))
        self.assertEqual(init[0][3], 0)
        self.assertGreaterEqual(init[0][2], 16)


class UnixBridgeTests(unittest.TestCase):
    @unittest.skipIf(sys.platform == "win32", "Unix FIFO test")
    def test_regular_file_to_fifo_streams(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            guest_out = root / "out.bin"
            guest_in = root / "in.fifo"
            guest_out.write_bytes(b"")
            os.mkfifo(guest_in)
            stop = threading.Event()
            source, sink = cf_hvc_bridge.open_hvc_streams(str(guest_out), str(guest_in), stop)
            reader_fd = os.open(guest_in, os.O_RDWR | os.O_NONBLOCK)
            with guest_out.open("ab", buffering=0) as out:
                out.write(b"guest")
            self.assertEqual(source.read(5), b"guest")
            sink.write(b"host")
            deadline = time.monotonic() + 1
            data = b""
            while time.monotonic() < deadline and not data:
                try:
                    data = os.read(reader_fd, 4)
                except BlockingIOError:
                    time.sleep(0.01)
            self.assertEqual(data, b"host")
            stop.set()
            source.close()
            sink.close()
            os.close(reader_fd)


class HostProcessTests(unittest.TestCase):
    @unittest.skipUnless(sys.platform == "darwin", "macOS UDS transport test")
    def test_modem_uds_at_and_cleanup(self) -> None:
        cid = 123
        base_port = 19600
        socket_path = Path(
            f"/tmp/binder_rpc_vsock_{cid}_{base_port + cid - 3}.sock"
        )
        socket_path.unlink(missing_ok=True)
        process = subprocess.Popen(
            [
                sys.executable,
                str(SCRIPT_DIR / "modem_simulator_host.py"),
                "--guest-cid",
                str(cid),
                "--base-port",
                str(base_port),
            ]
        )
        try:
            deadline = time.monotonic() + 5
            while not socket_path.exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            self.assertTrue(socket_path.exists())
            with socket.socket(socket.AF_UNIX) as client:
                client.connect(str(socket_path))
                client.sendall(pack_framed_payload(b"AT+CREG?\r"))
                client.settimeout(2)
                buffer = bytearray()
                replies = []
                deadline = time.monotonic() + 2
                while time.monotonic() < deadline and not any(
                    b"OK" in reply for reply in replies
                ):
                    buffer.extend(client.recv(4096))
                    replies.extend(unpack_framed_payloads(buffer))
            self.assertTrue(replies)
            response = b"".join(replies)
            self.assertIn(b"+CREG:", response)
            self.assertIn(b"OK", response)
        finally:
            process.terminate()
            process.wait(timeout=5)
            deadline = time.monotonic() + 2
            while socket_path.exists() and time.monotonic() < deadline:
                time.sleep(0.05)
            self.assertFalse(socket_path.exists())


if __name__ == "__main__":
    unittest.main()
