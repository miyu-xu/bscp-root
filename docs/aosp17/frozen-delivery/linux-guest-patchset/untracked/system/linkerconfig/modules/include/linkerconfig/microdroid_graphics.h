// BSCP graphics Microdroid's fixed, signed-image opt-in.
#pragma once
#include <string>
#include <utility>
#include <initializer_list>
#include <cerrno>
#include <cstring>
#include <fcntl.h>
#include <sys/stat.h>
#include <android-base/file.h>
#include <android-base/result.h>
#include <android-base/unique_fd.h>
namespace android::linkerconfig::modules {
inline android::base::Result<bool> ReadMicrodroidGraphicsConfig(const std::string& root) {
  using android::base::unique_fd;
  constexpr char kContent[] = "sphal.default.vulkan.ranchu.so.v1\n";
  unique_fd dir(open(root.empty() ? "/" : root.c_str(), O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
  if (dir.get() < 0) return android::base::ErrnoError() << "Cannot open configuration root";
  for (const char* component : {"system", "etc"}) {
    unique_fd next(openat(dir.get(), component, O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
    if (next.get() < 0) {
      if (errno == ENOENT) return false;
      return android::base::ErrnoError() << "Cannot open Microdroid configuration directory";
    }
    dir = std::move(next);
  }
  unique_fd fd(openat(dir.get(), "microdroid-graphics-linkerconfig", O_RDONLY | O_CLOEXEC | O_NOFOLLOW | O_NONBLOCK));
  if (fd.get() < 0) {
    if (errno == ENOENT) return false;
    return android::base::ErrnoError() << "Cannot open Microdroid graphics configuration";
  }
  struct stat st;
  if (fstat(fd.get(), &st) != 0) return android::base::ErrnoError() << "Cannot stat Microdroid graphics configuration";
  if (!S_ISREG(st.st_mode) || st.st_size != static_cast<off_t>(sizeof(kContent)-1))
    return android::base::Error() << "Invalid Microdroid graphics configuration type/size";
  char content[sizeof(kContent)] = {};
  if (!android::base::ReadFully(fd.get(), content, sizeof(kContent)-1) ||
      memcmp(content, kContent, sizeof(kContent)-1) != 0)
    return android::base::Error() << "Invalid Microdroid graphics configuration content";
  return true;
}
}  // namespace android::linkerconfig::modules
