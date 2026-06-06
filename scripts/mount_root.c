/* Minimal static mount tool for initramfs.
 * Supports: normal mount, remount, bind mount
 * Compile: musl-gcc -static -o mount_root mount_root.c
 */
#include <stdio.h>
#include <string.h>
#include <sys/mount.h>
#include <errno.h>

int main(int argc, char **argv) {
    const char *target = "/newroot";

    if (argc >= 2 && strcmp(argv[1], "--remount") == 0) {
        if (argc >= 3) target = argv[2];
        printf("mount_root: remounting %s RW\n", target);
        if (mount(NULL, target, NULL, MS_REMOUNT, NULL) == 0) {
            printf("mount_root: remount success\n");
            return 0;
        }
        printf("mount_root: remount failed: %s\n", strerror(errno));
        return 1;
    }

    if (argc >= 2 && strcmp(argv[1], "--bind") == 0) {
        // mount_root --bind /src /dst
        if (argc < 4) { printf("usage: mount_root --bind SRC DST\n"); return 1; }
        const char *src = argv[2];
        const char *dst = argv[3];
        printf("mount_root: bind %s -> %s\n", src, dst);
        if (mount(src, dst, NULL, MS_BIND, NULL) == 0) {
            printf("mount_root: bind success\n");
            return 0;
        }
        printf("mount_root: bind failed: %s\n", strerror(errno));
        return 1;
    }

    // Normal mount
    const char *dev = "/dev/vda3";
    const char *fstype = "ext4";

    if (argc >= 2) dev = argv[1];
    if (argc >= 3) target = argv[2];
    if (argc >= 4) fstype = argv[3];

    printf("mount_root: mounting %s -> %s (%s)\n", dev, target, fstype);

    // Try noload first (skip journal recovery)
    unsigned long noload = (1 << 14); // MS_NOLOAD
    if (mount(dev, target, fstype, noload, NULL) == 0) {
        printf("mount_root: noload success\n");
        return 0;
    }
    printf("mount_root: noload: %s\n", strerror(errno));

    // Try RO
    printf("mount_root: trying RO...\n");
    if (mount(dev, target, fstype, MS_RDONLY, NULL) == 0) {
        printf("mount_root: RO success\n");
        return 2; // RO mounted
    }
    printf("mount_root: all failed: %s\n", strerror(errno));
    return 1;
}
