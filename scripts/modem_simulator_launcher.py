#!/usr/bin/env python3
"""Prepare Cuttlefish config and exec modem_simulator with a vsock server FD."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import socket
import sys
from pathlib import Path


def cf_modem_vsock_port(cid: int, base_port: int = 9600) -> int:
    # Match Cuttlefish GetVsockServerPort(base, vsock_guest_cid).
    return base_port + (cid - 3)


def write_cuttlefish_config(
    config_root: Path,
    *,
    guest_cid: int,
    instance_num: int,
    ril_gateway: str,
    ril_ipaddr: str,
    ril_prefixlen: int,
    ril_dns: str,
) -> Path:
    instance_id = str(instance_num)
    instance_name = f"cvd-{instance_id}"
    instance_dir = config_root / "instances" / instance_name
    logs_dir = instance_dir / "logs"
    internal_dir = instance_dir / "internal"
    uds_dir = config_root / "instances" / instance_name
    for path in (logs_dir, internal_dir, uds_dir):
        path.mkdir(parents=True, exist_ok=True)

    config = {
        "root_dir": str(config_root),
        "instances_uds_dir": str(config_root / "instances"),
        "instance_names": [instance_name],
        "instances": {
            instance_id: {
                "enable_modem_simulator": True,
                "modem_simulator_host_id": 1000 + instance_num,
                "modem_simulator_instance_number": 1,
                "modem_simulator_sim_type": 1,
                "vsock_guest_cid": guest_cid,
                "ril_gateway": ril_gateway,
                "ril_ipaddr": ril_ipaddr,
                "ril_prefixlen": ril_prefixlen,
                "ril_dns": ril_dns,
            }
        },
    }
    config_path = config_root / "cuttlefish_config.json"
    config_path.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
    return config_path


def create_vsock_server(port: int) -> socket.socket:
    sock = socket.socket(socket.AF_VSOCK, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind((socket.VMADDR_CID_ANY, port))
    sock.listen(4)
    return sock


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config-root", required=True)
    parser.add_argument("--modem-bin", required=True)
    parser.add_argument("--guest-cid", type=int, required=True)
    parser.add_argument("--instance-num", type=int, default=1)
    parser.add_argument("--sim-type", type=int, default=1)
    parser.add_argument("--base-port", type=int, default=9600)
    parser.add_argument("--ril-gateway", default="192.168.97.1")
    parser.add_argument("--ril-ipaddr", default="192.168.97.2")
    parser.add_argument("--ril-prefixlen", type=int, default=30)
    parser.add_argument("--ril-dns", default="8.8.8.8")
    parser.add_argument("--aosp-host-out", default="")
    args = parser.parse_args()

    config_root = Path(args.config_root)
    config_root.mkdir(parents=True, exist_ok=True)
    config_path = write_cuttlefish_config(
        config_root,
        guest_cid=args.guest_cid,
        instance_num=args.instance_num,
        ril_gateway=args.ril_gateway,
        ril_ipaddr=args.ril_ipaddr,
        ril_prefixlen=args.ril_prefixlen,
        ril_dns=args.ril_dns,
    )

    port = cf_modem_vsock_port(args.guest_cid, args.base_port)
    vsock = create_vsock_server(port)
    fd = vsock.fileno()
    flags = fcntl.fcntl(fd, fcntl.F_GETFD)
    fcntl.fcntl(fd, fcntl.F_SETFD, flags & ~fcntl.FD_CLOEXEC)

    os.environ["CUTTLEFISH_CONFIG_FILE"] = str(config_path)
    if args.aosp_host_out:
        os.environ.setdefault("ANDROID_HOST_OUT", args.aosp_host_out)

    modem_bin = args.modem_bin
    argv = [
        modem_bin,
        f"-server_fds={fd}",
        f"-sim_type={args.sim_type}",
    ]
    os.execv(modem_bin, argv)
    return 1


if __name__ == "__main__":
    sys.exit(main())
