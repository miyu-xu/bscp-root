# bscp-root

macOS host AVF runs require an **arm64** `com.android.virt` apex tree. To stage one into the shared runtime layout, use:

```bash
MACOS_AVF_APEX_TREE_SOURCE=/path/to/arm64/apex_tree ./build_all.sh
```
