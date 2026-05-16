// Copyright (C) 2024 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include "CoherentMemoryBacking.h"

namespace gfxstream {
namespace vk {
namespace {

using ::testing::Eq;

TEST(CoherentMemoryBackingTest, validateAllocationPasses) {
    CoherentHostMemoryProbeResult probeResult;
    probeResult.success = true;
    probeResult.coherentHostMemoryTypeMask = (1u << 0) | (1u << 2);  // types 0 and 2

    auto backing = CoherentMemoryBacking::createForTest(probeResult);
    ASSERT_NE(backing, nullptr);

    // Type 0 is coherent, guest requests coherent → passes
    EXPECT_THAT(backing->validateCoherentAllocation(
                    0, VK_MEMORY_PROPERTY_HOST_COHERENT_BIT),
                Eq(VK_SUCCESS));

    // Type 2 is coherent, guest requests coherent → passes
    EXPECT_THAT(backing->validateCoherentAllocation(
                    2, VK_MEMORY_PROPERTY_HOST_COHERENT_BIT),
                Eq(VK_SUCCESS));
}

TEST(CoherentMemoryBackingTest, validateAllocationRejects) {
    CoherentHostMemoryProbeResult probeResult;
    probeResult.success = true;
    probeResult.coherentHostMemoryTypeMask = (1u << 0);  // only type 0 coherent

    auto backing = CoherentMemoryBacking::createForTest(probeResult);
    ASSERT_NE(backing, nullptr);

    // Type 1 is NOT coherent, guest requests coherent → rejects
    EXPECT_THAT(backing->validateCoherentAllocation(
                    1, VK_MEMORY_PROPERTY_HOST_COHERENT_BIT),
                Eq(VK_ERROR_INCOMPATIBLE_DRIVER));

    // Type 3 is NOT coherent, guest requests coherent → rejects
    EXPECT_THAT(backing->validateCoherentAllocation(
                    3, VK_MEMORY_PROPERTY_HOST_COHERENT_BIT),
                Eq(VK_ERROR_INCOMPATIBLE_DRIVER));
}

TEST(CoherentMemoryBackingTest, validateAllocationNoCoherentReq) {
    CoherentHostMemoryProbeResult probeResult;
    probeResult.success = true;
    probeResult.coherentHostMemoryTypeMask = (1u << 0);  // only type 0 coherent

    auto backing = CoherentMemoryBacking::createForTest(probeResult);
    ASSERT_NE(backing, nullptr);

    // Guest did NOT request coherent → always VK_SUCCESS,
    // even for types that are NOT in the coherent mask.
    EXPECT_THAT(backing->validateCoherentAllocation(0, 0), Eq(VK_SUCCESS));
    EXPECT_THAT(backing->validateCoherentAllocation(1, 0), Eq(VK_SUCCESS));
    EXPECT_THAT(backing->validateCoherentAllocation(5, 0), Eq(VK_SUCCESS));
}

TEST(CoherentMemoryBackingTest, isHostTypeCoherent) {
    CoherentHostMemoryProbeResult probeResult;
    probeResult.success = true;
    probeResult.coherentHostMemoryTypeMask = (1u << 0) | (1u << 2);

    auto backing = CoherentMemoryBacking::createForTest(probeResult);
    ASSERT_NE(backing, nullptr);

    EXPECT_TRUE(backing->isHostTypeCoherent(0));
    EXPECT_FALSE(backing->isHostTypeCoherent(1));
    EXPECT_TRUE(backing->isHostTypeCoherent(2));
    EXPECT_FALSE(backing->isHostTypeCoherent(3));
}

TEST(CoherentMemoryBackingTest, hasCoherentTypes) {
    {
        CoherentHostMemoryProbeResult probeResult;
        probeResult.success = true;
        probeResult.coherentHostMemoryTypeMask = 0;

        auto backing = CoherentMemoryBacking::createForTest(probeResult);
        ASSERT_NE(backing, nullptr);

        EXPECT_FALSE(backing->hasCoherentTypes());
        EXPECT_TRUE(backing->probeSucceeded());
    }

    {
        CoherentHostMemoryProbeResult probeResult;
        probeResult.success = true;
        probeResult.coherentHostMemoryTypeMask = (1u << 1);

        auto backing = CoherentMemoryBacking::createForTest(probeResult);
        ASSERT_NE(backing, nullptr);

        EXPECT_TRUE(backing->hasCoherentTypes());
        EXPECT_TRUE(backing->probeSucceeded());
    }
}

TEST(CoherentMemoryBackingTest, probeFailedHasNoTypes) {
    CoherentHostMemoryProbeResult probeResult;
    probeResult.success = false;
    probeResult.coherentHostMemoryTypeMask = 0;

    auto backing = CoherentMemoryBacking::createForTest(probeResult);
    ASSERT_NE(backing, nullptr);

    EXPECT_FALSE(backing->probeSucceeded());
    EXPECT_FALSE(backing->hasCoherentTypes());
    EXPECT_EQ(backing->coherentHostMemoryTypeMask(), 0u);
}

}  // namespace
}  // namespace vk
}  // namespace gfxstream
