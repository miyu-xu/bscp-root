// Copyright (C) 2022 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include "DisplaySurfaceVk.h"

#include "host-common/GfxstreamFatalError.h"
#include "host-common/logging.h"
#include "vk_util.h"

#ifdef __linux__
#include <xcb/xcb.h>
#endif

namespace gfxstream {
namespace vk {

using emugl::ABORT_REASON_OTHER;
using emugl::FatalError;

std::unique_ptr<DisplaySurfaceVk> DisplaySurfaceVk::create(const VulkanDispatch& vk,
                                                           VkInstance instance,
                                                           FBNativeWindowType window) {
    VkSurfaceKHR surface = VK_NULL_HANDLE;
#ifdef __linux__
    xcb_connection_t* xcbConnection = nullptr;
#endif
#ifdef _WIN32
    const VkWin32SurfaceCreateInfoKHR surfaceCi = {
        .sType = VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
        .pNext = nullptr,
        .flags = 0,
        .hinstance = GetModuleHandle(nullptr),
        .hwnd = window,
    };
    VK_CHECK(vk.vkCreateWin32SurfaceKHR(instance, &surfaceCi, nullptr, &surface));
#elif defined(__linux__)
    xcbConnection = xcb_connect(nullptr, nullptr);
    if (!xcbConnection || xcb_connection_has_error(xcbConnection)) {
        GFXSTREAM_ABORT(FatalError(ABORT_REASON_OTHER))
            << "Failed to connect to X server for Vulkan XCB surface.";
    }

    const VkXcbSurfaceCreateInfoKHR surfaceCi = {
        .sType = VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
        .pNext = nullptr,
        .flags = 0,
        .connection = xcbConnection,
        .window = static_cast<xcb_window_t>(window),
    };
    VK_CHECK(vk.vkCreateXcbSurfaceKHR(instance, &surfaceCi, nullptr, &surface));
#endif
    if (surface == VK_NULL_HANDLE) {
        GFXSTREAM_ABORT(FatalError(ABORT_REASON_OTHER))
            << "No VkSurfaceKHR created?";
    }

    return std::unique_ptr<DisplaySurfaceVk>(new DisplaySurfaceVk(
        vk, instance, surface
#ifdef __linux__
        ,
        xcbConnection
#endif
        ));
}

DisplaySurfaceVk::DisplaySurfaceVk(const VulkanDispatch& vk, VkInstance instance,
                                   VkSurfaceKHR surface
#ifdef __linux__
                                   ,
                                   xcb_connection_t* xcbConnection
#endif
                                   )
    : mVk(vk),
      mInstance(instance),
      mSurface(surface)
#ifdef __linux__
      ,
      mXcbConnection(xcbConnection)
#endif
{}

DisplaySurfaceVk::~DisplaySurfaceVk() {
    if (mSurface != VK_NULL_HANDLE) {
        mVk.vkDestroySurfaceKHR(mInstance, mSurface, nullptr);
    }
#ifdef __linux__
    if (mXcbConnection) {
        xcb_disconnect(mXcbConnection);
    }
#endif
}

}  // namespace vk
}  // namespace gfxstream
