cmake_minimum_required(VERSION 3.22)
project(bscp_a17_windows_rpc LANGUAGES C CXX)
if(NOT WIN32)
 message(FATAL_ERROR "Windows-only configuration")
endif()
set(CMAKE_CXX_STANDARD 20)
add_compile_definitions(__USE_MINGW_ANSI_STDIO=1 WIN32_LEAN_AND_MEAN NOMINMAX)
set(CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS ON)
set(A ${CMAKE_CURRENT_SOURCE_DIR}/..)
set(B ${A}/frameworks/native/libs/binder)
set(U ${A}/system/core/libutils/binder)
set(P ${A}/windows-platform)
add_compile_options(-include ${P}/WindowsHeaders.h)
add_library(binder_log SHARED ${A}/native-build/log_print.cpp)
target_include_directories(binder_log PRIVATE ${B}/include)
target_compile_definitions(binder_log PRIVATE BUILDING_LIBBINDER)
add_library(utils_binder_sdk SHARED ${U}/Errors.cpp ${U}/RefBase.cpp ${U}/SharedBuffer.cpp ${U}/String16.cpp ${U}/String8.cpp ${U}/StrongPointer.cpp ${U}/Unicode.cpp ${U}/VectorImpl.cpp)
target_include_directories(utils_binder_sdk PUBLIC ${B}/include ${U}/include PRIVATE ${B}/liblog_stub/include ${A}/system/core/libutils/include ${P}/win32)
target_compile_definitions(utils_binder_sdk PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE ANDROID_LOG_STUB_WEAK_PRINT ANDROID_UTILS_CALLSTACK_ENABLED=0 ANDROID_BASE_UNIQUE_FD_DISABLE_IMPLICIT_CONVERSION ANDROID_UTILS_REF_BASE_DISABLE_IMPLICIT_CONSTRUCTION NO_LIBBINDER_BINDER_OBSERVER BUILDING_LIBBINDER PLATFORM_WINDOWS _WIN32_WINNT=0x0601)
target_compile_options(utils_binder_sdk PRIVATE -Wall -Wextra -Werror -Wno-missing-field-initializers -fno-rtti)
target_link_libraries(utils_binder_sdk PRIVATE binder_log)

add_executable(a17_utils_test utils_test.cpp)
target_link_libraries(a17_utils_test PRIVATE utils_binder_sdk binder_log)
target_include_directories(a17_utils_test PRIVATE ${U} ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_utils_test PRIVATE BINDER_NO_LIBBASE)
target_compile_options(a17_utils_test PRIVATE -Wall -Wextra -Werror -fno-rtti)
add_library(binder_objects OBJECT
 ${B}/Binder.cpp ${B}/BpBinder.cpp ${B}/Debug.cpp ${P}/WindowsFdTrigger.cpp
 ${B}/IInterface.cpp ${B}/IResultReceiver.cpp ${B}/JavaBBinderBase.cpp
 ${B}/Parcel.cpp ${B}/ParcelFileDescriptor.cpp ${B}/RecordedTransaction.cpp
 ${B}/RpcSession.cpp ${B}/RpcServer.cpp ${B}/RpcState.cpp ${B}/RpcTransportRaw.cpp
 ${B}/Stability.cpp ${B}/Status.cpp ${B}/TextOutput.cpp ${B}/Utils.cpp
 ${B}/file.cpp ${B}/PersistableBundle.cpp)
target_include_directories(binder_objects PRIVATE ${P} ${B}/include ${B}/liblog_stub/include ${U}/include ${B} ${B}/include_private ${A}/system/core/libutils/include ${P}/win32)
target_compile_definitions(binder_objects PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE ANDROID_LOG_STUB_WEAK_PRINT ANDROID_UTILS_CALLSTACK_ENABLED=0 ANDROID_BASE_UNIQUE_FD_DISABLE_IMPLICIT_CONVERSION ANDROID_UTILS_REF_BASE_DISABLE_IMPLICIT_CONSTRUCTION NO_LIBBINDER_BINDER_OBSERVER BUILDING_LIBBINDER PLATFORM_WINDOWS _WIN32_WINNT=0x0601)
target_compile_options(binder_objects PRIVATE -Wall -Wextra -Werror -Wno-missing-field-initializers -fno-rtti)

add_executable(a17_fd_test fd_test.cpp ${P}/WindowsFileDescriptor.cpp ${P}/WindowsSocketOwner.cpp ${P}/WindowsSocketApi.cpp)
target_include_directories(a17_fd_test PRIVATE ${B} ${B}/include ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_fd_test PRIVATE BINDER_NO_LIBBASE)
target_compile_options(a17_fd_test PRIVATE -Wall -Wextra -Werror -fno-rtti)

add_executable(a17_cancellation_test cancellation_test.cpp)
target_include_directories(a17_cancellation_test PRIVATE ${P})
target_compile_options(a17_cancellation_test PRIVATE -Wall -Wextra -Werror)

add_executable(a17_mapping_test mapping_test.cpp)
target_include_directories(a17_mapping_test PRIVATE ${P}/win32)
target_compile_options(a17_mapping_test PRIVATE -Wall -Wextra -Werror)
target_link_libraries(a17_mapping_test PRIVATE winpthread)

add_executable(a17_socket_owner_test socket_owner_test.cpp ${P}/WindowsSocketOwner.cpp)
target_include_directories(a17_socket_owner_test PRIVATE ${P})
target_compile_options(a17_socket_owner_test PRIVATE -Wall -Wextra -Werror)
target_link_libraries(a17_socket_owner_test PRIVATE ws2_32 winpthread)

target_link_libraries(a17_fd_test PRIVATE ws2_32 winpthread)

add_executable(a17_trigger_test trigger_test.cpp ${P}/WindowsFdTrigger.cpp ${P}/WindowsSocketOwner.cpp ${P}/WindowsSocketApi.cpp ${P}/WindowsFileDescriptor.cpp)
target_include_directories(a17_trigger_test PRIVATE ${P} ${B} ${B}/include ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_trigger_test PRIVATE BINDER_NO_LIBBASE ANDROID_LOG_STUB_WEAK_PRINT)
target_compile_options(a17_trigger_test PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_libraries(a17_trigger_test PRIVATE binder_log ws2_32 winpthread)

add_library(binder_sdk SHARED $<TARGET_OBJECTS:binder_objects> ${P}/OS_Windows.cpp ${P}/WindowsFileDescriptor.cpp ${P}/WindowsSocketOwner.cpp ${P}/WindowsSocketApi.cpp)
target_include_directories(binder_sdk PUBLIC ${B}/include ${U}/include PRIVATE ${B} ${P} ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(binder_sdk PRIVATE $<TARGET_PROPERTY:binder_objects,COMPILE_DEFINITIONS>)
target_compile_options(binder_sdk PRIVATE -Wall -Wextra -Werror -Wno-missing-field-initializers -fno-rtti)
target_link_libraries(binder_sdk PRIVATE utils_binder_sdk binder_log ws2_32 bcrypt winpthread)

add_executable(a17_recording_test recording_test.cpp ${B}/RecordedTransaction.cpp)
target_include_directories(a17_recording_test PRIVATE ${CMAKE_CURRENT_SOURCE_DIR} ${P} ${B} ${B}/include ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_recording_test PRIVATE BINDER_NO_LIBBASE ANDROID_LOG_STUB_WEAK_PRINT)
target_compile_options(a17_recording_test PRIVATE -Wall -Wextra -Werror -Wno-missing-field-initializers -fno-rtti "SHELL:-include ${CMAKE_CURRENT_SOURCE_DIR}/mapping_fault_inject.h")
target_link_libraries(a17_recording_test PRIVATE binder_sdk utils_binder_sdk binder_log winpthread)
target_compile_definitions(a17_recording_test PRIVATE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_trigger_test PRIVATE "SHELL:-include ${CMAKE_CURRENT_SOURCE_DIR}/event_fault_inject.h")

add_library(binder_ndk_sdk SHARED ${B}/ndk/ibinder.cpp ${B}/ndk/libbinder.cpp ${B}/ndk/parcel.cpp ${B}/ndk/stability.cpp ${B}/ndk/status.cpp ${B}/ndk/persistable_bundle.cpp)
target_include_directories(binder_ndk_sdk PUBLIC ${B}/ndk/include_cpp ${B}/ndk/include_ndk ${B}/ndk/include_platform PRIVATE ${B}/ndk ${B} ${P} ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(binder_ndk_sdk PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE)
target_compile_options(binder_ndk_sdk PRIVATE -Wall -Wextra -Werror -Wno-missing-field-initializers -fno-rtti)
target_link_libraries(binder_ndk_sdk PRIVATE binder_sdk utils_binder_sdk binder_log)
add_library(binder_rpc_unstable_host SHARED ${B}/libbinder_rpc_unstable.cpp)
target_include_directories(binder_rpc_unstable_host PUBLIC ${B}/include_rpc_unstable PRIVATE ${B} ${P} ${B}/include ${U}/include ${B}/liblog_stub/include ${P}/win32 ${A}/system/core/libcutils/include)
target_compile_definitions(binder_rpc_unstable_host PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE)
target_compile_options(binder_rpc_unstable_host PRIVATE -Wall -Wextra -Werror -Wno-missing-field-initializers -fno-rtti)
target_link_libraries(binder_rpc_unstable_host PRIVATE binder_ndk_sdk binder_sdk utils_binder_sdk binder_log)

target_link_libraries(binder_ndk_sdk PRIVATE winpthread)

add_executable(a17_rpc_session_test rpc_session_test.cpp)
target_include_directories(a17_rpc_session_test PRIVATE ${B} ${B}/include ${U}/include ${B}/liblog_stub/include ${P} ${P}/win32)
target_compile_definitions(a17_rpc_session_test PRIVATE BINDER_NO_LIBBASE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_rpc_session_test PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_libraries(a17_rpc_session_test PRIVATE binder_sdk utils_binder_sdk binder_log winpthread ws2_32)

add_executable(a17_rpc_bridge_test rpc_bridge_test.cpp)
target_include_directories(a17_rpc_bridge_test PRIVATE ${B} ${B}/include ${U}/include ${B}/liblog_stub/include ${P} ${P}/win32)
target_compile_definitions(a17_rpc_bridge_test PRIVATE BINDER_NO_LIBBASE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_rpc_bridge_test PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_libraries(a17_rpc_bridge_test PRIVATE binder_rpc_unstable_host binder_ndk_sdk binder_sdk utils_binder_sdk binder_log winpthread ws2_32)

target_sources(binder_sdk PRIVATE ${P}/WindowsFdAbi.cpp)
add_executable(a17_fd_abi_test fd_abi_test.cpp)
target_include_directories(a17_fd_abi_test PRIVATE ${P})
target_compile_options(a17_fd_abi_test PRIVATE -Wall -Wextra -Werror)
target_link_libraries(a17_fd_abi_test PRIVATE binder_sdk ws2_32 winpthread)

# MinGW disables implicit exports when any explicit C ABI export exists.
# Retain the complete public Binder/import-library surface alongside the C ABI.
target_link_options(binder_sdk PRIVATE -Wl,--export-all-symbols)

target_sources(binder_sdk PRIVATE ${P}/WindowsCallerIdentity.cpp ${P}/WindowsAccessControl.cpp ${P}/WindowsIdentityAbi.cpp)
target_link_libraries(binder_sdk PRIVATE advapi32 ntdll)
add_executable(a17_identity_abi_test identity_abi_test.cpp)
target_include_directories(a17_identity_abi_test PRIVATE ${P})
target_compile_options(a17_identity_abi_test PRIVATE -Wall -Wextra -Werror)
target_link_libraries(a17_identity_abi_test PRIVATE binder_sdk advapi32 winpthread)

target_link_options(a17_identity_abi_test PRIVATE -municode)

add_executable(a17_ndk_stability_test ndk_stability_test.cpp)
target_link_libraries(a17_ndk_stability_test PRIVATE binder_ndk_sdk)
target_include_directories(a17_ndk_stability_test PRIVATE ${B}/include)

# Windows NDR identity linkage only; no server startup enabled here.
set(BSCP_WLR_IDENTITY_DIR ${P}/local-rpc-association-v1587-r3)
include(${CMAKE_CURRENT_SOURCE_DIR}/WindowsLocalRpc.cmake)
bscp_add_windows_local_rpc(bscp_wlr_bridge "${BSCP_WLR_IDENTITY_DIR}"
 "C:/Program Files (x86)/Windows Kits/10" "10.0.26100.0"
 "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.44.35207")
target_sources(binder_sdk PRIVATE ${P}/WindowsNdrCallerIdentity.cpp ${P}/WindowsNdrIdentityAbi.cpp ${BSCP_WLR_IDENTITY_DIR}/WindowsNdrRecordAdapter.cpp)
target_include_directories(binder_sdk PRIVATE ${BSCP_WLR_IDENTITY_DIR})
target_link_libraries(binder_sdk PRIVATE bscp_wlr_bridge)
add_custom_target(bscp_wlr_runtime_copy
 COMMAND "${CMAKE_COMMAND}" -E copy_if_different "$<TARGET_FILE:bscp_wlr_bridge>" "${CMAKE_CURRENT_BINARY_DIR}/WindowsLocalRpcBridge.dll"
 DEPENDS bscp_wlr_bridge_build "$<TARGET_FILE:bscp_wlr_bridge>"
 VERBATIM)
add_dependencies(binder_sdk bscp_wlr_runtime_copy)
add_executable(a17_ndr_pid_link_test ndr_pid_link_test.cpp)
target_include_directories(a17_ndr_pid_link_test PRIVATE ${P})
target_compile_options(a17_ndr_pid_link_test PRIVATE -Wall -Wextra -Werror)
target_link_libraries(a17_ndr_pid_link_test PRIVATE binder_sdk)

# Isolated unified ABI candidate, no entry startup.
target_sources(binder_sdk PRIVATE ${P}/RpcTransportNdr.cpp ${P}/packet-v1559/WindowsNdrReceivedPacket.cpp ${BSCP_WLR_IDENTITY_DIR}/WindowsNdrPacketCodec.cpp)
target_include_directories(binder_sdk PRIVATE ${P}/packet-v1559)
target_include_directories(binder_objects PRIVATE ${BSCP_WLR_IDENTITY_DIR} ${P}/packet-v1559)

add_executable(a17_ndr_binder_loopback binder_ndr_loopback_test.cpp)
target_include_directories(a17_ndr_binder_loopback PRIVATE ${P} ${BSCP_WLR_IDENTITY_DIR} ${B}/include ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_ndr_binder_loopback PRIVATE BINDER_NO_LIBBASE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_ndr_binder_loopback PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_options(a17_ndr_binder_loopback PRIVATE -municode)
target_link_libraries(a17_ndr_binder_loopback PRIVATE binder_sdk bscp_wlr_bridge utils_binder_sdk binder_log winpthread)

target_sources(binder_rpc_unstable_host PRIVATE ${P}/WindowsBinderRpc.cpp)
target_link_options(binder_rpc_unstable_host PRIVATE -Wl,--export-all-symbols)
add_executable(a17_ndr_cabi_test cabi_test.cpp)
target_include_directories(a17_ndr_cabi_test PRIVATE ${P} ${BSCP_WLR_IDENTITY_DIR} ${B}/include ${B}/include_rpc_unstable ${B}/ndk/include_cpp ${B}/ndk/include_ndk ${B}/ndk/include_platform ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_ndr_cabi_test PRIVATE BINDER_NO_LIBBASE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_ndr_cabi_test PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_options(a17_ndr_cabi_test PRIVATE -municode)
target_link_libraries(a17_ndr_cabi_test PRIVATE binder_rpc_unstable_host binder_ndk_sdk binder_sdk bscp_wlr_bridge utils_binder_sdk binder_log winpthread)

target_sources(binder_sdk PRIVATE ${P}/WindowsNdrServerLifecycle.cpp)
add_executable(a17_ndr_lifecycle_test lifecycle_test.cpp)
target_include_directories(a17_ndr_lifecycle_test PRIVATE ${P} ${BSCP_WLR_IDENTITY_DIR} ${B}/include ${B}/include_rpc_unstable ${B}/ndk/include_cpp ${B}/ndk/include_ndk ${B}/ndk/include_platform ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_ndr_lifecycle_test PRIVATE BINDER_NO_LIBBASE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_ndr_lifecycle_test PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_options(a17_ndr_lifecycle_test PRIVATE -municode)
target_link_libraries(a17_ndr_lifecycle_test PRIVATE binder_rpc_unstable_host binder_ndk_sdk binder_sdk bscp_wlr_bridge utils_binder_sdk binder_log winpthread)

target_sources(binder_sdk PRIVATE ${P}/WindowsInitialSessionLifecycle.cpp)
target_compile_definitions(binder_ndk_sdk PRIVATE BSCP_BINDER_NDK_BUILD)
add_executable(a17_initial_crossprocess binder_crossprocess_initial.cpp)
target_include_directories(a17_initial_crossprocess PRIVATE ${P} ${BSCP_WLR_IDENTITY_DIR} ${B}/include ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_initial_crossprocess PRIVATE BINDER_NO_LIBBASE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_initial_crossprocess PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_options(a17_initial_crossprocess PRIVATE -municode)
target_link_libraries(a17_initial_crossprocess PRIVATE binder_rpc_unstable_host binder_ndk_sdk binder_sdk bscp_wlr_bridge utils_binder_sdk binder_log winpthread)

target_link_options(binder_ndk_sdk PRIVATE -Wl,--export-all-symbols)

target_sources(binder_sdk PRIVATE ${P}/WindowsOverlappedPipeFd.cpp ${P}/WindowsPipeTransport.cpp)
add_executable(a17_guest_raw_factory guest_raw_factory_test.cpp)
target_include_directories(a17_guest_raw_factory PRIVATE ${P} ${B} ${B}/include ${U}/include ${B}/liblog_stub/include ${P}/win32)
target_compile_definitions(a17_guest_raw_factory PRIVATE BINDER_NO_LIBBASE BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB)
target_compile_options(a17_guest_raw_factory PRIVATE -Wall -Wextra -Werror -fno-rtti)
target_link_libraries(a17_guest_raw_factory PRIVATE binder_sdk utils_binder_sdk binder_log winpthread)

# Windows Guest endpoint lifecycle; real pipe transports, no socket surrogate.
target_sources(binder_sdk PRIVATE ${P}/WindowsGuestListener.cpp ${P}/WindowsPipeFrame.cpp)
