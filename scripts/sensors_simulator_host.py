#!/usr/bin/env python3
"""Small Cuttlefish SensorsHalProxy-compatible HVC service.

Supports the AOSP RawMessage framing used by the newer Cuttlefish sensor
sub-HAL and a line-mode fallback for older images.  It intentionally exposes
the stable baseline sensors only: accelerometer, gyroscope, and magnetometer.
"""

from __future__ import annotations

import argparse
import math
import struct
import sys
import threading
import time

from cf_hvc_bridge import Stream, open_hvc_streams

K_UPDATE_HAL = 2
HOST_SENSOR_MASK = (1 << 0) | (1 << 1) | (1 << 2)
HEADER = struct.Struct("<II")
MAX_PAYLOAD = 1024 * 1024


def encode_message(command: int, payload: bytes, response: bool = True) -> bytes:
    command_word = command | ((1 << 31) if response else 0)
    return HEADER.pack(command_word, len(payload)) + payload


def decode_messages(buffer: bytearray) -> list[tuple[int, bool, bytes]]:
    messages: list[tuple[int, bool, bytes]] = []
    while len(buffer) >= HEADER.size:
        command_word, size = HEADER.unpack_from(buffer)
        command = command_word & 0x7FFFFFFF
        if size > MAX_PAYLOAD or command > 0xFFFF:
            break
        total = HEADER.size + size
        if len(buffer) < total:
            break
        payload = bytes(buffer[HEADER.size:total])
        del buffer[:total]
        messages.append((command, bool(command_word >> 31), payload))
    return messages


def sensor_reports() -> list[bytes]:
    # AOSP's neutral phone orientation uses gravity on +Y.
    return [
        b"acceleration:0:9.80665:0\n",
        b"gyroscope:0:0:0\n",
        b"magnetic:0:5.9:-48.4\n",
    ]


class SensorService:
    def __init__(self, source: Stream, sink: Stream, interval: float) -> None:
        self.source = source
        self.sink = sink
        self.interval = interval
        self.stop = threading.Event()
        self.activated = threading.Event()
        self.framed = True

    def send(self, payload: bytes) -> None:
        data = encode_message(K_UPDATE_HAL, payload) if self.framed else payload
        self.sink.write(data)

    def process_payload(self, payload: bytes, framed: bool) -> None:
        text = payload.decode("ascii", errors="ignore").strip().lower()
        if text.startswith("list-sensors"):
            self.framed = framed
            self.send(f"{HOST_SENSOR_MASK}\n".encode("ascii"))
            self.activated.set()
            print("sensors_simulator_host: HAL activated", flush=True)

    def request_loop(self) -> None:
        buffer = bytearray()
        line_buffer = bytearray()
        try:
            while not self.stop.is_set():
                data = self.source.read()
                if not data:
                    break
                buffer.extend(data)
                decoded = decode_messages(buffer)
                if decoded:
                    for _cmd, _response, payload in decoded:
                        self.process_payload(payload, True)
                    continue

                # Do not consume an incomplete plausible framed header.
                if len(buffer) >= HEADER.size:
                    command_word, size = HEADER.unpack_from(buffer)
                    if (command_word & 0x7FFFFFFF) <= 0xFFFF and size <= MAX_PAYLOAD:
                        continue
                line_buffer.extend(buffer)
                buffer.clear()
                while b"\n" in line_buffer:
                    line, _, rest = line_buffer.partition(b"\n")
                    line_buffer[:] = rest
                    self.process_payload(line, False)
        finally:
            self.stop.set()

    def report_loop(self) -> None:
        while not self.stop.wait(self.interval):
            if self.activated.is_set():
                for report in sensor_reports():
                    self.send(report)

    def run(self) -> None:
        reporter = threading.Thread(target=self.report_loop, daemon=True)
        reporter.start()
        self.request_loop()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--guest-out", required=True)
    parser.add_argument("--guest-in", required=True)
    parser.add_argument("--interval-ms", type=int, default=1000)
    args = parser.parse_args()
    stop = threading.Event()
    try:
        source, sink = open_hvc_streams(args.guest_out, args.guest_in, stop)
        print(
            f"sensors_simulator_host: serving {args.guest_out}/{args.guest_in}",
            flush=True,
        )
        SensorService(source, sink, max(args.interval_ms, 10) / 1000.0).run()
    except KeyboardInterrupt:
        stop.set()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
