#!/usr/bin/env bash
# Build OVMF X64 DEBUG with DEBUG_ON_SERIAL_PORT (logs on COM1 / 0x3F8).
# Output: out/dist/firmware/OVMF_DEBUG.fd
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FW_DIR="${REPO_ROOT}/out/dist/firmware"
EDK2_TAG="${EDK2_TAG:-edk2-stable202411}"
# Build under WSL home (faster than /mnt/c); copy artifact back to repo.
EDK2_DIR="${EDK2_DIR:-${HOME}/.cache/bscp-edk2}"
OUT_FD="${FW_DIR}/OVMF_DEBUG.fd"
LOG="${FW_DIR}/ovmf_debug_build.log"

mkdir -p "${FW_DIR}"
exec > >(tee -a "${LOG}") 2>&1
echo "==> Log: ${LOG}"

need_setup=1
if [[ -x "${EDK2_DIR}/BaseTools/BinWrappers/PosixLike/build" ]]; then
  need_setup=0
fi

if [[ "${need_setup}" -eq 1 ]]; then
  if ! command -v gcc >/dev/null 2>&1 || ! command -v nasm >/dev/null 2>&1; then
    echo "==> Installing build deps (apt)"
    if command -v apt-get >/dev/null 2>&1; then
      APT="apt-get"
      if [[ "$(id -u)" -ne 0 ]]; then APT="sudo apt-get"; fi
      ${APT} update -qq
      DEBIAN_FRONTEND=noninteractive ${APT} install -y --no-install-recommends \
        build-essential uuid-dev acpica-tools nasm python3 python-is-python3 git ca-certificates
    fi
  else
    echo "==> Build deps already present (gcc/nasm)"
  fi

  mkdir -p "$(dirname "${EDK2_DIR}")"
  if [[ ! -d "${EDK2_DIR}/.git" ]]; then
    echo "==> Cloning edk2 (${EDK2_TAG}) -> ${EDK2_DIR}"
    git clone --depth 1 --branch "${EDK2_TAG}" \
      https://github.com/tianocore/edk2.git "${EDK2_DIR}"
  fi

  echo "==> edk2 submodules"
  git -C "${EDK2_DIR}" submodule update --init --recursive

  echo "==> BaseTools (C tools only; skip Tests)"
  make -C "${EDK2_DIR}/BaseTools/Source/C" -j"$(nproc)"
fi

cd "${EDK2_DIR}"
export PYTHON_COMMAND="${PYTHON_COMMAND:-python3}"
export EDK_TOOLS_PATH="${EDK2_DIR}/BaseTools"
# edksetup references unset vars under 'set -u'
set +u
# shellcheck disable=SC1091
source edksetup.sh
set -u

TOOLCHAIN="${EDK2_TOOLCHAIN:-GCC}"
SERIAL_FLAG="${OVMF_SERIAL_DEBUG:-1}"
BUILD_OPTS="-D DEBUG_ON_SERIAL_PORT"
PCD_OPTS="--pcd gEfiMdeModulePkgTokenSpaceGuid.PcdCpuStackGuard=FALSE --pcd gUefiCpuPkgTokenSpaceGuid.PcdCpuApLoopMode=2"
if [[ "${SERIAL_FLAG}" == "0" ]]; then
  BUILD_OPTS=""
  OUT_FD="${FW_DIR}/OVMF_DEBUG_DEBUGCON.fd"
  echo "==> Building OvmfPkgX64 DEBUG (debugcon 0x402, no serial debug)"
else
  echo "==> Building OvmfPkgX64 DEBUG + DEBUG_ON_SERIAL_PORT (toolchain=${TOOLCHAIN})"
fi
echo "==> Building OvmfPkgX64 DEBUG (toolchain=${TOOLCHAIN}) ${BUILD_OPTS}"
# ClearCacheOnMpServicesAvailable issues LAPIC INIT/SIPI via StartupAllAPs; WHPX LAPIC
# emulation does not complete ICR DeliveryStatus without X64ApicInitSipiExitTrap (unsupported
# on many hosts). Skip the callback for crosvm/WHPX bring-up.
PLATFORM_PEI="${EDK2_DIR}/OvmfPkg/PlatformPei/Platform.c"
if [[ -f "${PLATFORM_PEI}" ]] && grep -q 'InstallClearCacheCallback ();' "${PLATFORM_PEI}"; then
  sed -i 's/InstallClearCacheCallback ();/\/\/ InstallClearCacheCallback (); \/\/ crosvm\/WHPX/' "${PLATFORM_PEI}"
  echo "==> Patched OvmfPkg/PlatformPei/Platform.c: disabled InstallClearCacheCallback"
fi
# BuildMpInformationHob calls MpInitLibStartupAllCPUs (ExcludeBsp=FALSE) to read hybrid
# core types; on WHPX this issues LAPIC INIT/SIPI and hangs without ApicInitSipiTrap.
CPU_MPEI="${EDK2_DIR}/UefiCpuPkg/CpuMpPei/CpuMpPei.c"
if [[ -f "${CPU_MPEI}" ]] && grep -q 'if (CpuidMaxInput >= CPUID_HYBRID_INFORMATION) {' "${CPU_MPEI}"; then
  sed -i 's/if (CpuidMaxInput >= CPUID_HYBRID_INFORMATION) {/if (CpuidMaxInput >= CPUID_HYBRID_INFORMATION \&\& NumberOfProcessors > 1) {/' "${CPU_MPEI}"
  echo "==> Patched CpuMpPei.c: skip hybrid StartupAllCPUs on uniprocessor"
fi
build -a X64 -b DEBUG -t "${TOOLCHAIN}" \
  -p OvmfPkg/OvmfPkgX64.dsc \
  ${BUILD_OPTS} ${PCD_OPTS}

BUILT="$(find Build/OvmfX64 -path '*/FV/OVMF.fd' -type f 2>/dev/null | head -1)"
if [[ -z "${BUILT}" ]]; then
  echo "Error: OVMF.fd not found under Build/OvmfX64" >&2
  find Build -name 'OVMF.fd' 2>/dev/null || true
  exit 1
fi

cp -f "${BUILT}" "${OUT_FD}"
echo "==> Installed ${OUT_FD} ($(wc -c < "${OUT_FD}") bytes) from ${BUILT}"
sha256sum "${OUT_FD}"
