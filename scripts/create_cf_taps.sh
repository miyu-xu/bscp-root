#!/usr/bin/env bash
set -euo pipefail

MOBILE_TAP="${CROSVM_ANDROID_MOBILE_TAP:-cvd-android-mobile}"
ETHERNET_TAP="${CROSVM_ANDROID_ETHERNET_TAP:-cvd-android-eth1}"
MOBILE_HOST_IP="${CROSVM_ANDROID_MOBILE_HOST_IP:-192.168.96.1}"
ETHERNET_HOST_IP="${CROSVM_ANDROID_ETHERNET_HOST_IP:-192.168.97.1}"
NETMASK="${CROSVM_ANDROID_NETMASK:-255.255.255.0}"
PREFIX="${NETMASK_PREFIX:-24}"

usage() {
    cat <<EOF
Usage: $0 [--mobile-tap NAME] [--ethernet-tap NAME] [--delete]

Create or delete the Cuttlefish-style TAP pair used by direct runners.
Prints:
  MOBILE_TAP=...
  ETHERNET_TAP=...
EOF
}

DELETE=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --mobile-tap) MOBILE_TAP="$2"; shift 2 ;;
        --ethernet-tap) ETHERNET_TAP="$2"; shift 2 ;;
        --mobile-host-ip) MOBILE_HOST_IP="$2"; shift 2 ;;
        --ethernet-host-ip) ETHERNET_HOST_IP="$2"; shift 2 ;;
        --delete) DELETE=1; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if ! command -v ip >/dev/null; then
    echo "Error: ip(8) not found" >&2
    exit 1
fi

tap_exists() {
    ip link show "$1" &>/dev/null
}

delete_tap() {
    local name="$1"
    if tap_exists "$name"; then
        ip link delete "$name" 2>/dev/null || sudo ip link delete "$name"
    fi
}

create_tap() {
    local name="$1"
    local host_ip="$2"
    if tap_exists "$name"; then
        if ip link show "$name" | grep -q 'state DOWN'; then
            ip link set "$name" up 2>/dev/null || sudo ip link set "$name" up || true
        fi
        return 0
    fi
    if ip tuntap add dev "$name" mode tap user "$(id -un)" 2>/dev/null; then
        :
    elif sudo ip tuntap add dev "$name" mode tap user "$(id -un)"; then
        :
    else
        echo "Error: failed to create TAP $name (need CAP_NET_ADMIN or sudo)" >&2
        return 1
    fi
    if ip addr add "${host_ip}/${PREFIX}" dev "$name" 2>/dev/null; then
        :
    else
        sudo ip addr add "${host_ip}/${PREFIX}" dev "$name"
    fi
    if ip link set "$name" up 2>/dev/null; then
        :
    else
        sudo ip link set "$name" up
    fi
}

if [[ "$DELETE" -eq 1 ]]; then
    delete_tap "$MOBILE_TAP" || true
    delete_tap "$ETHERNET_TAP" || true
    exit 0
fi

if tap_exists "cvd-mtap-01" && tap_exists "cvd-etap-01"; then
    MOBILE_TAP="cvd-mtap-01"
    ETHERNET_TAP="cvd-etap-01"
    ip link set "$MOBILE_TAP" up 2>/dev/null || true
    ip link set "$ETHERNET_TAP" up 2>/dev/null || true
else
    create_tap "$MOBILE_TAP" "$MOBILE_HOST_IP"
    create_tap "$ETHERNET_TAP" "$ETHERNET_HOST_IP"
fi

printf 'MOBILE_TAP=%s\n' "$MOBILE_TAP"
printf 'ETHERNET_TAP=%s\n' "$ETHERNET_TAP"
