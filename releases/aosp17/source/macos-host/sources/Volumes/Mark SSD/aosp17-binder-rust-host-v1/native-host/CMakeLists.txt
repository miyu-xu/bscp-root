set(A17_BINDER_SOURCE_ROOT "/Volumes/Mark SSD/aosp17-binder-host-v1")
cmake_minimum_required(VERSION 3.22)
project(a17_binder_sdk_mac LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(B ${A17_BINDER_SOURCE_ROOT}/frameworks/native/libs/binder)
set(U ${A17_BINDER_SOURCE_ROOT}/system/core/libutils/binder)
add_library(binder_log SHARED ${A17_BINDER_SOURCE_ROOT}/platform/log_print.cpp)
target_include_directories(binder_log PRIVATE ${B}/include)
target_compile_definitions(binder_log PRIVATE BUILDING_LIBBINDER)
add_library(utils_binder_sdk SHARED
 ${U}/Errors.cpp
 ${U}/RefBase.cpp
 ${U}/SharedBuffer.cpp
 ${U}/String16.cpp
 ${U}/String8.cpp
 ${U}/StrongPointer.cpp
 ${U}/Unicode.cpp
 ${U}/VectorImpl.cpp
)
add_library(binder_sdk SHARED
 ${B}/Binder.cpp
 ${B}/BpBinder.cpp
 ${B}/Debug.cpp
 ${B}/FdTrigger.cpp
 ${B}/IInterface.cpp
 ${B}/IResultReceiver.cpp
 ${B}/JavaBBinderBase.cpp
 ${B}/Parcel.cpp
 ${B}/ParcelFileDescriptor.cpp
 ${B}/RecordedTransaction.cpp
 ${B}/RpcSession.cpp
 ${B}/RpcServer.cpp
 ${B}/RpcState.cpp
 ${B}/RpcTransportRaw.cpp
 ${B}/Stability.cpp
 ${B}/Status.cpp
 ${B}/TextOutput.cpp
 ${B}/Utils.cpp
 ${B}/file.cpp
 ${B}/OS_unix_base.cpp
 ${A17_BINDER_SOURCE_ROOT}/platform/OS_macos.cpp)
foreach(t utils_binder_sdk binder_sdk)
 target_include_directories(${t} PUBLIC ${B}/include ${B}/liblog_stub/include ${U}/include PRIVATE ${B} ${B}/include_private ${A17_BINDER_SOURCE_ROOT}/system/core/libutils/include)
 target_compile_definitions(${t} PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE ANDROID_LOG_STUB_WEAK_PRINT ANDROID_UTILS_CALLSTACK_ENABLED=0 ANDROID_BASE_UNIQUE_FD_DISABLE_IMPLICIT_CONVERSION ANDROID_UTILS_REF_BASE_DISABLE_IMPLICIT_CONSTRUCTION NO_LIBBINDER_BINDER_OBSERVER BUILDING_LIBBINDER)
 target_compile_options(${t} PRIVATE -Wall -Wextra -Werror)
endforeach()
target_link_libraries(binder_sdk PRIVATE utils_binder_sdk)

target_link_libraries(utils_binder_sdk PRIVATE binder_log)
target_link_libraries(binder_sdk PRIVATE binder_log)

target_compile_options(binder_sdk PRIVATE -fvisibility=hidden)

add_executable(socket_tests ${A17_BINDER_SOURCE_ROOT}/platform/socket_tests.cpp)
target_include_directories(socket_tests PRIVATE ${B})
target_compile_options(socket_tests PRIVATE -Wall -Wextra -Werror)

add_library(binder_ndk_sdk SHARED ${B}/ndk/ibinder.cpp ${B}/ndk/libbinder.cpp ${B}/ndk/parcel.cpp ${B}/ndk/stability.cpp ${B}/ndk/status.cpp)
target_include_directories(binder_ndk_sdk PUBLIC ${B}/ndk/include_cpp ${B}/ndk/include_ndk ${B}/ndk/include_platform PRIVATE ${B}/ndk)
target_compile_definitions(binder_ndk_sdk PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE)
target_compile_options(binder_ndk_sdk PRIVATE -Wall -Wextra -Werror)
target_link_libraries(binder_ndk_sdk PRIVATE binder_sdk utils_binder_sdk binder_log)

foreach(t utils_binder_sdk binder_sdk binder_ndk_sdk)
 target_compile_options(${t} PUBLIC -fno-rtti)
endforeach()

add_executable(receive_tests ${A17_BINDER_SOURCE_ROOT}/platform/receive_tests.cpp)
target_include_directories(receive_tests PRIVATE ${B} ${B}/include_private)
target_compile_definitions(receive_tests PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE)
target_compile_options(receive_tests PRIVATE -Wall -Wextra -Werror)
target_link_libraries(receive_tests PRIVATE binder_sdk utils_binder_sdk binder_log)

target_sources(receive_tests PRIVATE ${B}/file.cpp)

target_sources(binder_sdk PRIVATE ${B}/PersistableBundle.cpp)
target_sources(binder_ndk_sdk PRIVATE ${B}/ndk/persistable_bundle.cpp)

add_library(binder_rpc_unstable_host SHARED ${B}/libbinder_rpc_unstable.cpp)
target_include_directories(binder_rpc_unstable_host PUBLIC ${B}/include_rpc_unstable PRIVATE ${B} ${A17_BINDER_SOURCE_ROOT}/system/core/libcutils/include)
target_compile_definitions(binder_rpc_unstable_host PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE)
target_compile_options(binder_rpc_unstable_host PRIVATE -Wall -Wextra -Werror)
target_link_libraries(binder_rpc_unstable_host PRIVATE binder_ndk_sdk binder_sdk utils_binder_sdk binder_log)

# Explicit desktop-only server opt-in; no Guest/wire change.
target_sources(binder_sdk PRIVATE ${A17_BINDER_SOURCE_ROOT}/platform/DesktopRpcIdentity.cpp)
foreach(t binder_sdk binder_ndk_sdk binder_rpc_unstable_host)
 target_include_directories(${t} PUBLIC ${A17_BINDER_SOURCE_ROOT}/platform)
 target_compile_definitions(${t} PUBLIC BSCP_DESKTOP_RPC_AUTH)
endforeach()

add_executable(desktop_rpc_identity_test ${A17_BINDER_SOURCE_ROOT}/platform/desktop-auth-v737/rpc_identity_test.cpp)
target_compile_options(desktop_rpc_identity_test PRIVATE -Wall -Wextra -Werror)
target_link_libraries(desktop_rpc_identity_test PRIVATE binder_sdk utils_binder_sdk binder_log)

target_compile_definitions(desktop_rpc_identity_test PRIVATE BINDER_ENABLE_LIBLOG_ASSERT BINDER_DISABLE_NATIVE_HANDLE BINDER_DISABLE_BLOB BINDER_NO_LIBBASE)

target_include_directories(desktop_rpc_identity_test PRIVATE ${B})
