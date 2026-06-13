#!/usr/bin/env python3
"""Fix selinux.cpp to skip SELinux policy loading in permissive mode."""
import sys

with open(sys.argv[1], 'r') as f:
    c = f.read()

# Patch ReadPolicy: add early return when permissive
old_rp = '''void ReadPolicy(std::string* policy) {
    PolicyFile policy_file;'''
new_rp = '''void ReadPolicy(std::string* policy) {
    if (ALLOW_PERMISSIVE_SELINUX && !IsEnforcing()) {
        LOG(WARNING) << "Skipping SELinux policy (permissive mode)";
        return;
    }
    PolicyFile policy_file;'''
c = c.replace(old_rp, new_rp, 1)

# Patch LoadSelinuxPolicy: skip empty policy
old_lp = '''    if (security_load_policy(policy.data(), policy.size()) < 0) {
        PLOG(FATAL) << "SELinux:  Could not load policy";
    }'''
new_lp = '''    if (policy.empty()) {
        LOG(WARNING) << "Empty policy, skipping load (permissive)";
        return;
    }
    if (security_load_policy(policy.data(), policy.size()) < 0) {
        PLOG(FATAL) << "SELinux:  Could not load policy";
    }'''
c = c.replace(old_lp, new_lp, 1)

with open(sys.argv[1], 'w') as f:
    f.write(c)
print('PATCHED')
