#!/usr/bin/env bash
set -euo pipefail

# Start Cuttlefish host-side daemons used by the direct crosvm runners.
# When sourced, defines cf_start_host_daemons and keeps FIFO FDs open in the caller.
# When executed directly with --probe-only, prints availability variables to stdout.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
AOSP_ROOT="${AOSP_ROOT:-$REPO_ROOT/../aosp}"
AOSP_HOST_BIN="${AOSP_HOST_BIN:-$AOSP_ROOT/out/host/linux-x86/bin}"

cf_resolve_host_bin() {
    local name="$1"
    local dist_bin="${CF_DIST_BIN:-}"
    local candidate
    for candidate in \
        "$REPO_ROOT/out/dist/host-tools/linux-x86_64/bin/$name" \
        "$REPO_ROOT/out/dist/host-tools/linux-arm64/bin/$name" \
        "$AOSP_HOST_BIN/$name" \
        ${dist_bin:+"$dist_bin/$name"}; do
        if [[ -n "$candidate" && -x "$candidate" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done
    return 1
}

cf_ensure_fifo() {
    local path="$1"
    if [[ ! -p "$path" ]]; then
        rm -f "$path"
        mkfifo -m 660 "$path"
    fi
}

cf_modem_vsock_port() {
    local guest_cid="$1"
    local base_port="${2:-9600}"
    echo $((base_port + guest_cid - 3))
}

cf_probe_host_daemons() {
    local enable_bluetooth="${1:-1}"
    local enable_nfc="${2:-1}"
    local dist_bin="${3:-}"
    local enable_modem="${4:-1}"

    CF_DIST_BIN="$dist_bin"
    CF_HOST_BLUETOOTH_BACKEND=disabled
    CF_HOST_NFC_BACKEND=disabled
    CF_HOST_MODEM_BACKEND=disabled
    if [[ "$enable_bluetooth" -eq 1 ]]; then
        if cf_resolve_host_bin root-canal >/dev/null && cf_resolve_host_bin tcp_connector >/dev/null; then
            CF_HOST_BLUETOOTH_BACKEND=native
        elif [[ -f "$SCRIPT_DIR/root_canal_stub.py" && -f "$SCRIPT_DIR/cf_hvc_bridge.py" ]] && command -v python3 >/dev/null; then
            CF_HOST_BLUETOOTH_BACKEND=stub
        else
            enable_bluetooth=0
        fi
    fi
    if [[ "$enable_nfc" -eq 1 ]]; then
        if cf_resolve_host_bin casimir >/dev/null && cf_resolve_host_bin tcp_connector >/dev/null; then
            CF_HOST_NFC_BACKEND=native
        elif [[ -f "$SCRIPT_DIR/casimir_stub.py" && -f "$SCRIPT_DIR/cf_hvc_bridge.py" ]] && command -v python3 >/dev/null; then
            CF_HOST_NFC_BACKEND=stub
        else
            enable_nfc=0
        fi
    fi
    if [[ "$enable_modem" -eq 1 ]]; then
        if cf_resolve_host_bin modem_simulator >/dev/null; then
            CF_HOST_MODEM_BACKEND=native
        elif [[ -f "$SCRIPT_DIR/modem_simulator_host.py" ]] && command -v python3 >/dev/null; then
            CF_HOST_MODEM_BACKEND=stub
        else
            enable_modem=0
        fi
    fi
    CF_HOST_BLUETOOTH_ENABLED="$enable_bluetooth"
    CF_HOST_NFC_ENABLED="$enable_nfc"
    CF_HOST_MODEM_ENABLED="$enable_modem"
}

cf_start_host_daemons() {
    local work_dir="$1"
    local log_dir="$2"
    local dist_bin="${3:-}"
    local enable_bluetooth="${4:-1}"
    local enable_nfc="${5:-1}"
    local bt_hci_port="${6:-7300}"
    local casimir_nci_port="${7:-7800}"
    local casimir_rf_port="${8:-7900}"
    local enable_modem="${9:-1}"
    local guest_cid="${10:-100}"
    local modem_instance_num="${11:-1}"
    local modem_sim_type="${12:-1}"
    local ril_gateway="${13:-192.168.97.1}"
    local ril_ipaddr="${14:-192.168.97.2}"
    local ril_prefixlen="${15:-30}"
    local ril_dns="${16:-8.8.8.8}"

    CF_DIST_BIN="$dist_bin"
    cf_probe_host_daemons "$enable_bluetooth" "$enable_nfc" "$dist_bin" "$enable_modem"
    enable_bluetooth="$CF_HOST_BLUETOOTH_ENABLED"
    enable_nfc="$CF_HOST_NFC_ENABLED"
    enable_modem="$CF_HOST_MODEM_ENABLED"

    local fifo_dir="$work_dir/hvc"
    mkdir -p "$work_dir" "$log_dir" "$fifo_dir"

    CF_HOST_DAEMON_PIDS=()
    CF_HOST_FIFO_FDS=()

    cf_start_daemon() {
        local name="$1"
        shift
        local stdout="$log_dir/${name}.stdout.txt"
        local stderr="$log_dir/${name}.stderr.txt"
        : >"$stdout"
        : >"$stderr"
        (
            if [[ -d "$AOSP_HOST_BIN/../lib" ]]; then
                export LD_LIBRARY_PATH="$AOSP_HOST_BIN/../lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            fi
            "$@"
        ) >"$stdout" 2>"$stderr" &
        CF_HOST_DAEMON_PIDS+=("$!")
        echo "Started $name (pid ${CF_HOST_DAEMON_PIDS[-1]}); logs: $stdout $stderr" >&2
    }

    cf_open_fifo_rw() {
        local path="$1"
        local fd_var="$2"
        cf_ensure_fifo "$path"
        eval "exec {${fd_var}}<>\"$path\""
        CF_HOST_FIFO_FDS+=("$fd_var")
    }

    if [[ "$enable_bluetooth" -eq 1 ]]; then
        local bt_in_fd bt_out_fd
        cf_open_fifo_rw "$fifo_dir/bt.in" bt_in_fd
        cf_open_fifo_rw "$fifo_dir/bt.out" bt_out_fd
        if [[ "$CF_HOST_BLUETOOTH_BACKEND" == native ]]; then
            cf_start_daemon root-canal \
                "$(cf_resolve_host_bin root-canal)" \
                --test_port="$((bt_hci_port + 1))" \
                --hci_port="$bt_hci_port" \
                --link_port="$((bt_hci_port + 2))" \
                --link_ble_port="$((bt_hci_port + 3))"
            sleep 1
            cf_start_daemon bt-connector \
                "$(cf_resolve_host_bin tcp_connector)" \
                -fifo_in="$bt_out_fd" \
                -fifo_out="$bt_in_fd" \
                -data_port="$bt_hci_port" \
                -buffer_size=2050
        else
            cf_start_daemon root-canal-stub \
                python3 "$SCRIPT_DIR/root_canal_stub.py" --hci-port "$bt_hci_port"
            sleep 0.2
            cf_start_daemon bt-stub-bridge \
                python3 "$SCRIPT_DIR/cf_hvc_bridge.py" \
                --guest-out "$fifo_dir/bt.out" --guest-in "$fifo_dir/bt.in" \
                --tcp-port "$bt_hci_port" --reconnect
        fi
    fi

    if [[ "$enable_nfc" -eq 1 ]]; then
        local nfc_in_fd nfc_out_fd
        cf_open_fifo_rw "$fifo_dir/nfc.in" nfc_in_fd
        cf_open_fifo_rw "$fifo_dir/nfc.out" nfc_out_fd
        if [[ "$CF_HOST_NFC_BACKEND" == native ]]; then
            cf_start_daemon casimir \
                "$(cf_resolve_host_bin casimir)" \
                --nci-port "$casimir_nci_port" \
                --rf-port "$casimir_rf_port"
            cf_start_daemon nfc-connector \
                "$(cf_resolve_host_bin tcp_connector)" \
                -fifo_in="$nfc_out_fd" \
                -fifo_out="$nfc_in_fd" \
                -data_port="$casimir_nci_port" \
                -buffer_size=1024
        else
            cf_start_daemon casimir-stub \
                python3 "$SCRIPT_DIR/casimir_stub.py" \
                --nci-port "$casimir_nci_port" --rf-port "$casimir_rf_port"
            sleep 0.2
            cf_start_daemon nfc-stub-bridge \
                python3 "$SCRIPT_DIR/cf_hvc_bridge.py" \
                --guest-out "$fifo_dir/nfc.out" --guest-in "$fifo_dir/nfc.in" \
                --tcp-port "$casimir_nci_port" --reconnect
        fi
    fi

    if [[ "$enable_modem" -eq 1 ]]; then
        if [[ "$CF_HOST_MODEM_BACKEND" == native ]]; then
            local launcher="$SCRIPT_DIR/modem_simulator_launcher.py"
            local config_root="$work_dir/cf_modem"
            local aosp_host_out
            aosp_host_out="$(cd "$AOSP_HOST_BIN/.." && pwd)"
            if [[ ! -x "$launcher" ]]; then
                echo "Error: missing modem launcher: $launcher" >&2
                exit 1
            fi
            cf_start_daemon modem-simulator \
                python3 "$launcher" \
                --config-root "$config_root" \
                --modem-bin "$(cf_resolve_host_bin modem_simulator)" \
                --guest-cid "$guest_cid" \
                --instance-num "$modem_instance_num" \
                --sim-type "$modem_sim_type" \
                --ril-gateway "$ril_gateway" \
                --ril-ipaddr "$ril_ipaddr" \
                --ril-prefixlen "$ril_prefixlen" \
                --ril-dns "$ril_dns" \
                --aosp-host-out "$aosp_host_out"
        else
            cf_start_daemon modem-simulator-stub \
                python3 "$SCRIPT_DIR/modem_simulator_host.py" \
                --guest-cid "$guest_cid" \
                --ril-gateway "$ril_gateway" \
                --ril-ipaddr "$ril_ipaddr" \
                --ril-prefixlen "$ril_prefixlen" \
                --ril-dns "$ril_dns"
        fi
    fi

    local sensors_host="$SCRIPT_DIR/sensors_simulator_host.py"
    if [[ -f "$sensors_host" ]]; then
        : >"$log_dir/sensors-hvc13.txt"
        if [[ ! -p "$fifo_dir/sensors.in" ]]; then
            rm -f "$fifo_dir/sensors.in"
            mkfifo "$fifo_dir/sensors.in"
        fi
        cf_start_daemon sensors-simulator \
            python3 "$sensors_host" \
            --guest-out "$log_dir/sensors-hvc13.txt" \
            --guest-in "$fifo_dir/sensors.in"
    fi
}

cf_stop_host_daemons() {
    local pid
    for pid in "${CF_HOST_DAEMON_PIDS[@]:-}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
    CF_HOST_DAEMON_PIDS=()
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    usage() {
        cat <<EOF
Usage: $0 [options]

Options:
  --work-dir DIR          Runtime FIFO/work directory
  --log-dir DIR           Log directory for daemon stdout/stderr
  --dist-bin DIR          Directory containing host binaries
  --aosp-host-bin DIR     Fallback AOSP host bin directory
  --bluetooth             Require Bluetooth daemons
  --no-bluetooth          Do not require Bluetooth daemons
  --nfc                   Require NFC daemons
  --no-nfc                Do not require NFC daemons
  --probe-only            Print availability variables and exit
  --help                  Show this help
EOF
    }

    WORK_DIR=""
    LOG_DIR=""
    DIST_BIN=""
    ENABLE_BLUETOOTH=1
    ENABLE_NFC=1
    PROBE_ONLY=0

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --work-dir) WORK_DIR="$2"; shift 2 ;;
            --log-dir) LOG_DIR="$2"; shift 2 ;;
            --dist-bin) DIST_BIN="$2"; shift 2 ;;
            --aosp-host-bin) AOSP_HOST_BIN="$2"; shift 2 ;;
            --bluetooth) ENABLE_BLUETOOTH=1; shift ;;
            --no-bluetooth) ENABLE_BLUETOOTH=0; shift ;;
            --nfc) ENABLE_NFC=1; shift ;;
            --no-nfc) ENABLE_NFC=0; shift ;;
            --probe-only) PROBE_ONLY=1; shift ;;
            --help|-h) usage; exit 0 ;;
            *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
        esac
    done

    if [[ -z "$WORK_DIR" ]]; then
        echo "Error: --work-dir is required" >&2
        exit 2
    fi

    cf_probe_host_daemons "$ENABLE_BLUETOOTH" "$ENABLE_NFC" "$DIST_BIN"
    if [[ "$PROBE_ONLY" -eq 1 ]]; then
        printf 'CF_HOST_BLUETOOTH_ENABLED=%s\n' "$CF_HOST_BLUETOOTH_ENABLED"
        printf 'CF_HOST_NFC_ENABLED=%s\n' "$CF_HOST_NFC_ENABLED"
        printf 'CF_HOST_MODEM_ENABLED=%s\n' "${CF_HOST_MODEM_ENABLED:-0}"
        printf 'CF_HOST_BLUETOOTH_BACKEND=%s\n' "$CF_HOST_BLUETOOTH_BACKEND"
        printf 'CF_HOST_NFC_BACKEND=%s\n' "$CF_HOST_NFC_BACKEND"
        printf 'CF_HOST_MODEM_BACKEND=%s\n' "$CF_HOST_MODEM_BACKEND"
        exit 0
    fi

    if [[ -z "$LOG_DIR" ]]; then
        echo "Error: --log-dir is required unless --probe-only is set" >&2
        exit 2
    fi

    cf_start_host_daemons "$WORK_DIR" "$LOG_DIR" "$DIST_BIN" \
        "$ENABLE_BLUETOOTH" "$ENABLE_NFC"
    printf 'CF_HOST_DAEMON_PIDS=(%s)\n' "${CF_HOST_DAEMON_PIDS[*]}"
    printf 'CF_HOST_BLUETOOTH_ENABLED=%s\n' "$CF_HOST_BLUETOOTH_ENABLED"
    printf 'CF_HOST_NFC_ENABLED=%s\n' "$CF_HOST_NFC_ENABLED"
    printf 'CF_HOST_BLUETOOTH_BACKEND=%s\n' "$CF_HOST_BLUETOOTH_BACKEND"
    printf 'CF_HOST_NFC_BACKEND=%s\n' "$CF_HOST_NFC_BACKEND"
fi
