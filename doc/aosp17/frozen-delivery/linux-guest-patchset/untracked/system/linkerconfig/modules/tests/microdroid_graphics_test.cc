#include <vector>
#include "linkerconfig/microdroid_graphics.h"
#include <android-base/file.h>
#include <gtest/gtest.h>
#include <sys/stat.h>
#include <unistd.h>
using android::linkerconfig::modules::ReadMicrodroidGraphicsConfig;
TEST(microdroid_graphics, missing_and_exact_and_invalid_content) {
  TemporaryDir root;
  std::string system=std::string(root.path)+"/system", etc=system+"/etc", file=etc+"/microdroid-graphics-linkerconfig";
  auto absent=ReadMicrodroidGraphicsConfig(root.path);ASSERT_TRUE(absent.ok());EXPECT_FALSE(*absent);
  ASSERT_EQ(0,mkdir(system.c_str(),0700));ASSERT_EQ(0,mkdir(etc.c_str(),0700));
  absent=ReadMicrodroidGraphicsConfig(root.path);ASSERT_TRUE(absent.ok());EXPECT_FALSE(*absent);
  const std::string good="sphal.default.vulkan.ranchu.so.v1\n";
  for(const auto& content:std::vector<std::string>{"",good.substr(1),good+"x",std::string(good.size(),'x')}) {
    ASSERT_TRUE(android::base::WriteStringToFile(content,file));EXPECT_FALSE(ReadMicrodroidGraphicsConfig(root.path).ok());
  }
  ASSERT_TRUE(android::base::WriteStringToFile(good,file));auto valid=ReadMicrodroidGraphicsConfig(root.path);ASSERT_TRUE(valid.ok());EXPECT_TRUE(*valid);
}
TEST(microdroid_graphics, rejects_leaf_symlink_fifo_and_directory) {
  TemporaryDir root;std::string system=std::string(root.path)+"/system",etc=system+"/etc",file=etc+"/microdroid-graphics-linkerconfig",target=etc+"/target";
  ASSERT_EQ(0,mkdir(system.c_str(),0700));ASSERT_EQ(0,mkdir(etc.c_str(),0700));ASSERT_TRUE(android::base::WriteStringToFile("sphal.default.vulkan.ranchu.so.v1\n",target));
  ASSERT_EQ(0,symlink("target",file.c_str()));EXPECT_FALSE(ReadMicrodroidGraphicsConfig(root.path).ok());ASSERT_EQ(0,unlink(file.c_str()));
  ASSERT_EQ(0,mkfifo(file.c_str(),0600));EXPECT_FALSE(ReadMicrodroidGraphicsConfig(root.path).ok());ASSERT_EQ(0,unlink(file.c_str()));
  ASSERT_EQ(0,mkdir(file.c_str(),0700));EXPECT_FALSE(ReadMicrodroidGraphicsConfig(root.path).ok());
}
TEST(microdroid_graphics, rejects_each_ancestor_symlink) {
  TemporaryDir root;TemporaryDir outside;std::string system=std::string(root.path)+"/system";
  ASSERT_EQ(0,symlink(outside.path,system.c_str()));EXPECT_FALSE(ReadMicrodroidGraphicsConfig(root.path).ok());ASSERT_EQ(0,unlink(system.c_str()));
  ASSERT_EQ(0,mkdir(system.c_str(),0700));ASSERT_EQ(0,symlink(outside.path,(system+"/etc").c_str()));EXPECT_FALSE(ReadMicrodroidGraphicsConfig(root.path).ok());
}
