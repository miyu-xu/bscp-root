# Include from native CMake; creates no service and never references evidence output.
# Required: Python3_EXECUTABLE, source path and installed toolchain roots.
function(bscp_add_windows_local_rpc target source sdk sdk_version vc)
  if(NOT CMAKE_HOST_WIN32)
    message(FATAL_ERROR "The MIDL/MSVC C bridge requires the existing Windows toolchain")
  endif()
  if(DEFINED CMAKE_SIZEOF_VOID_P AND NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(FATAL_ERROR "This bridge is x64 only")
  endif()
  find_package(Python3 REQUIRED COMPONENTS Interpreter)
  get_filename_component(source "${source}" REALPATH)
  set(builder "${source}/build_windows_local_rpc.py")
  set(helper "${CMAKE_CURRENT_FUNCTION_LIST_DIR}/cmake_wlr_build.py")
  set(inputs "${source}/WindowsLocalRpc.idl" "${source}/WindowsLocalRpc.acf"
      "${source}/WindowsLocalRpcBridge.c" "${source}/WindowsLocalRpcBridge.h"
      "${source}/InitialBootstrap.c" "${source}/InitialBootstrap.h" "${source}/BootstrapCallback.idl"
      "${builder}" "${helper}" "${CMAKE_CURRENT_FUNCTION_LIST_FILE}")
  # MIDL invokes its backend; CL invokes its C front-end and linker. Track the
  # installed tool binaries/runtime DLLs, headers and libraries used by the exact
  # SDK/VC search directories. The conservative closure avoids stale transitive
  # include/default-CRT libraries; no global toolchain install or mutation.
  file(GLOB tool_files CONFIGURE_DEPENDS
       "${sdk}/bin/${sdk_version}/x64/midl*" "${vc}/bin/Hostx64/x64/*.exe"
       "${vc}/bin/Hostx64/x64/*.dll")
  file(GLOB_RECURSE include_files CONFIGURE_DEPENDS LIST_DIRECTORIES FALSE
       "${vc}/include/*" "${sdk}/Include/${sdk_version}/ucrt/*"
       "${sdk}/Include/${sdk_version}/shared/*" "${sdk}/Include/${sdk_version}/um/*")
  file(GLOB lib_files CONFIGURE_DEPENDS
       "${vc}/lib/x64/*.lib" "${sdk}/Lib/${sdk_version}/ucrt/x64/*.lib"
       "${sdk}/Lib/${sdk_version}/um/x64/*.lib")
  list(APPEND inputs ${tool_files} ${include_files} ${lib_files})
  list(REMOVE_DUPLICATES inputs)
  list(SORT inputs)
  set(manifest "")
  foreach(path IN LISTS inputs)
    if(NOT EXISTS "${path}" OR IS_DIRECTORY "${path}")
      message(FATAL_ERROR "Missing bridge build input: ${path}")
    endif()
    file(SHA256 "${path}" sha)
    string(APPEND manifest "${sha}  ${path}\n")
  endforeach()
  foreach(required "${sdk}/bin/${sdk_version}/x64/midl.exe"
                   "${sdk}/bin/${sdk_version}/x64/midlc.exe"
                   "${vc}/bin/Hostx64/x64/cl.exe" "${vc}/bin/Hostx64/x64/c1.dll"
                   "${vc}/bin/Hostx64/x64/link.exe")
    if(NOT EXISTS "${required}")
      message(FATAL_ERROR "Missing installed tool: ${required}")
    endif()
  endforeach()
  # Input edits rerun configuration, producing a different output location. A
  # previous recipe's DLL/import library cannot satisfy the new link target.
  set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${inputs})
  string(SHA256 key "${manifest}\n${source}|${sdk}|${sdk_version}|${vc}|association")
  set(root "${CMAKE_CURRENT_BINARY_DIR}/windows-local-rpc/${key}")
  set(out "${root}/artifacts")
  file(MAKE_DIRECTORY "${out}")
  file(GENERATE OUTPUT "${root}/inputs.sha256" CONTENT "${manifest}")
  set(stamp "${root}/verified-success.json")
  set(dll "${out}/WindowsLocalRpcBridge.dll")
  set(implib "${out}/WindowsLocalRpcBridge.lib")
  add_custom_command(OUTPUT "${stamp}"
    COMMAND "${CMAKE_COMMAND}" -E env "CL=" "_CL_=" "LINK=" "LIBPATH="
      "${Python3_EXECUTABLE}" -X utf8 "${helper}"
       --manifest "${root}/inputs.sha256" --stamp "${stamp}"
       --builder "${builder}" --source "${source}" --out "${out}"
       --sdk-root "${sdk}" --sdk-version "${sdk_version}" --vc-root "${vc}"
    DEPENDS ${inputs} "${root}/inputs.sha256"
    BYPRODUCTS "${dll}" "${implib}" "${out}/WindowsLocalRpc.h"
      "${out}/WindowsLocalRpc_c.c" "${out}/WindowsLocalRpc_s.c"
      "${out}/WindowsLocalRpc_c.obj" "${out}/WindowsLocalRpc_s.obj"
      "${out}/BootstrapCallback.h" "${out}/BootstrapCallback_c.c" "${out}/BootstrapCallback_s.c"
      "${out}/BootstrapCallback_c.obj" "${out}/BootstrapCallback_s.obj" "${out}/InitialBootstrap.obj"
      "${out}/midl-callback.stdout" "${out}/midl-callback.stderr"
      "${out}/WindowsLocalRpcBridge.obj" "${out}/WindowsLocalRpcBridge.exp"
      "${out}/build-recipe.json" "${out}/build-result.json" "${out}/artifacts.json"
      "${out}/midl.stdout" "${out}/midl.stderr"
      "${out}/bridge.stdout" "${out}/bridge.stderr" "${root}/build-status.json"
    COMMENT "Build and verify Windows typed-RPC C ABI bridge (no listener start)"
    VERBATIM)
  add_custom_target(${target}_build DEPENDS "${stamp}")
  add_library(${target} SHARED IMPORTED GLOBAL)
  set_target_properties(${target} PROPERTIES IMPORTED_LOCATION "${dll}"
      IMPORTED_IMPLIB "${implib}" INTERFACE_INCLUDE_DIRECTORIES "${source};${out}")
  add_dependencies(${target} ${target}_build)
  set(${target}_OUTPUT_DIR "${out}" PARENT_SCOPE)
endfunction()


