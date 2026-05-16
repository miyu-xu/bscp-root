#include "CoherentMemoryBacking.h"

#include "VulkanDispatch.h"
#include "host-common/logging.h"

namespace gfxstream {
namespace vk {

namespace {
constexpr VkDeviceSize kPageSize = 4096;
}  // namespace

CoherentMemoryBacking::CoherentMemoryBacking(CoherentHostMemoryProbeResult probeResult)
    : mProbeResult(probeResult) {}

// static
std::unique_ptr<CoherentMemoryBacking> CoherentMemoryBacking::createForPlatform(
    VulkanDispatch* vk,
    VkPhysicalDevice physdev,
    const VkPhysicalDeviceMemoryProperties& hostMemProps) {

    CoherentHostMemoryProbeResult result;

    // Allocate a single page-aligned probe buffer for
    // vkGetMemoryHostPointerPropertiesEXT.
    void* probePtr = nullptr;
#ifdef _WIN32
    probePtr = _aligned_malloc(kPageSize, kPageSize);
#else
    int ret = posix_memalign(&probePtr, kPageSize, kPageSize);
    if (ret != 0) {
        probePtr = nullptr;
    }
#endif

    if (!probePtr) {
        return std::unique_ptr<CoherentMemoryBacking>(
            new CoherentMemoryBacking(result));
    }

    VkMemoryHostPointerPropertiesEXT memoryHostPointerProperties = {
        VK_STRUCTURE_TYPE_MEMORY_HOST_POINTER_PROPERTIES_EXT, nullptr};

    VkResult probeResult = vk->vkGetMemoryHostPointerPropertiesEXT(
        reinterpret_cast<VkDevice>(physdev),
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_HOST_ALLOCATION_BIT_EXT,
        probePtr, &memoryHostPointerProperties);

    if (probeResult == VK_SUCCESS) {
        result.success = true;
        uint32_t memoryTypeBits = memoryHostPointerProperties.memoryTypeBits;

        // Intersect with compatible mask (all host memory types).
        uint32_t compatibleMemoryTypeMask = (1u << hostMemProps.memoryTypeCount) - 1;
        memoryTypeBits &= compatibleMemoryTypeMask;

        uint32_t coherentMask = 0;
        for (uint32_t i = 0; i < hostMemProps.memoryTypeCount; i++) {
            if (memoryTypeBits & (1u << i)) {
                VkMemoryPropertyFlags props = hostMemProps.memoryTypes[i].propertyFlags;
                if ((props & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) &&
                    (props & VK_MEMORY_PROPERTY_HOST_COHERENT_BIT)) {
                    coherentMask |= (1u << i);
                }
            }
        }
        result.coherentHostMemoryTypeMask = coherentMask;
    }

#ifdef _WIN32
    _aligned_free(probePtr);
#else
    free(probePtr);
#endif

    return std::unique_ptr<CoherentMemoryBacking>(new CoherentMemoryBacking(result));
}

VkResult CoherentMemoryBacking::validateCoherentAllocation(
    uint32_t hostTypeIndex,
    VkMemoryPropertyFlags guestRequestedPropertyFlags) const {

    const bool guestWantsCoherent =
        guestRequestedPropertyFlags & VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;

    if (!guestWantsCoherent) {
        return VK_SUCCESS;
    }

    if (!isHostTypeCoherent(hostTypeIndex)) {
        ERR("Coherent enforcement: guest requested HOST_COHERENT but "
            "host memory type %u is not in the coherent probe mask 0x%x",
            hostTypeIndex, coherentHostMemoryTypeMask());
        return VK_ERROR_INCOMPATIBLE_DRIVER;
    }

    return VK_SUCCESS;
}

// static
std::unique_ptr<CoherentMemoryBacking> CoherentMemoryBacking::createForTest(
    CoherentHostMemoryProbeResult probeResult) {
    return std::unique_ptr<CoherentMemoryBacking>(new CoherentMemoryBacking(probeResult));
}

}  // namespace vk
}  // namespace gfxstream
