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
    def __init__(
        self,
        control_source: Stream,
        control_sink: Stream,
        data_sink: Stream,
        interval: float,
    ) -> None:
        self.control_source = control_source
        self.control_sink = control_sink
        self.data_sink = data_sink
        self.interval = interval
        self.stop = threading.Event()
        self.activated = threading.Event()
        self.framed = True

    def send_control(self, payload: bytes) -> None:
        data = encode_message(K_UPDATE_HAL, payload) if self.framed else payload
        self.control_sink.write(data)

    def send_report(self, payload: bytes) -> None:
        data = encode_message(K_UPDATE_HAL, payload) if self.framed else payload
        self.data_sink.write(data)

    def process_payload(self, payload: bytes, framed: bool) -> None:
        text = payload.decode("ascii", errors="ignore").strip().lower()
        if text.startswith("list-sensors"):
            self.framed = framed
            self.send_control(f"{HOST_SENSOR_MASK}\n".encode("ascii"))
            self.activated.set()
            print("sensors_simulator_host: HAL activated", flush=True)

    def request_loop(self) -> None:
        buffer = bytearray()
        line_buffer = bytearray()
        try:
            while not self.stop.is_set():
                data = self.control_source.read()
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
                    self.send_report(report)

    def run(self) -> None:
        reporter = threading.Thread(target=self.report_loop, daemon=True)
        reporter.start()
        self.request_loop()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--guest-out")
    parser.add_argument("--guest-in")
    parser.add_argument("--control-guest-out")
    parser.add_argument("--control-guest-in")
    parser.add_argument("--data-guest-out")
    parser.add_argument("--data-guest-in")
    parser.add_argument("--interval-ms", type=int, default=1000)
    args = parser.parse_args()
    combined = bool(args.guest_out and args.guest_in)
    split = all(
        (
            args.control_guest_out,
            args.control_guest_in,
            args.data_guest_out,
            args.data_guest_in,
        )
    )
    if combined == split:
        parser.error(
            "provide either --guest-out/--guest-in or all four split control/data endpoints"
        )
    stop = threading.Event()
    try:
        if combined:
            control_source, control_sink = open_hvc_streams(
                args.guest_out, args.guest_in, stop
            )
            data_sink = control_sink
            description = f"combined {args.guest_out}/{args.guest_in}"
        else:
            control_source, control_sink = open_hvc_streams(
                args.control_guest_out, args.control_guest_in, stop
            )
            _data_source, data_sink = open_hvc_streams(
                args.data_guest_out, args.data_guest_in, stop
            )
            description = (
                f"control {args.control_guest_out}/{args.control_guest_in}; "
                f"data {args.data_guest_out}/{args.data_guest_in}"
            )
        print(f"sensors_simulator_host: serving {description}", flush=True)
        SensorService(
            control_source,
            control_sink,
            data_sink,
            max(args.interval_ms, 10) / 1000.0,
        ).run()
    except KeyboardInterrupt:
        stop.set()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
