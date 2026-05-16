#pragma once

#include <cstdint>
#include <memory>

#include <vulkan/vulkan.h>

#include "VkEmulatedPhysicalDeviceMemory.h"

namespace gfxstream {
namespace vk {

struct VulkanDispatch;

class CoherentMemoryBacking {
public:
    // Runs the VK_EXT_external_memory_host probe synchronously.
    static std::unique_ptr<CoherentMemoryBacking> createForPlatform(
        VulkanDispatch* vk,
        VkPhysicalDevice physdev,
        const VkPhysicalDeviceMemoryProperties& hostMemProps);

    // Bitmask of host memory type indices proven genuinely coherent.
    uint32_t coherentHostMemoryTypeMask() const { return mProbeResult.coherentHostMemoryTypeMask; }

    bool probeSucceeded() const { return mProbeResult.success; }
    bool hasCoherentTypes() const { return mProbeResult.coherentHostMemoryTypeMask != 0; }

    bool isHostTypeCoherent(uint32_t hostTypeIndex) const {
        return (mProbeResult.coherentHostMemoryTypeMask & (1u << hostTypeIndex)) != 0;
    }

    // Validate that an allocation targeting hostTypeIndex with the given guest-
    // requested property flags is backed by a genuinely coherent host memory type.
    // Returns VK_SUCCESS or VK_ERROR_INCOMPATIBLE_DRIVER.
    VkResult validateCoherentAllocation(uint32_t hostTypeIndex,
                                        VkMemoryPropertyFlags guestRequestedPropertyFlags) const;

    const CoherentHostMemoryProbeResult& probeResult() const { return mProbeResult; }

    static std::unique_ptr<CoherentMemoryBacking> createForTest(
        CoherentHostMemoryProbeResult probeResult);

private:
    explicit CoherentMemoryBacking(CoherentHostMemoryProbeResult probeResult);

    CoherentHostMemoryProbeResult mProbeResult;
};

}  // namespace vk
}  // namespace gfxstream
