"""Public command overlay; inherited ambient values/names never serialized."""
FIXED={'VIRTMGR_GUEST_CID','VIRTMGR_SERVICE_DIR','VIRTMGR_TRACE_FILE','VMCLIENT_TRACE_FILE','VIRTMGR_CONSOLE_LOG_DIR','VIRTMGR_PATH','VIRTMGR_CROSVM_PATH','VIRTMGR_APEX_ROOT','VIRTMGR_SYSTEM_ROOT','VIRTMGR_SYSTEM_EXT_ROOT','VIRTMGR_KEEP_TEMP','ANDROID_PROP_RO_BUILD_VERSION_SDK','TMP','TEMP','PATH','VIRTMGR_MICRODROID_JSON'}
REMOVED={'VIRTMGR_DEBUG_POLICY_JSON','GFXSTREAM_KVM_COHERENT_CACHED_ONLY','VIRTMGR_CAPTURE_GUEST_CONSOLE','VIRTMGR_CAPTURE_CROSVM_STDIO'}
def command_environment(plan,ambient,reviewed_profile_keys):
 allowed=FIXED|set(reviewed_profile_keys)
 for k,v in plan.items():
  if k not in allowed and (k not in ambient or ambient[k]!=v):raise ValueError('unreviewed non-task environment change')
 overlay={k:v for k,v in plan.items() if k in allowed}
 removed=sorted((set(ambient)-set(plan))|REMOVED)
 actual=dict(ambient)
 for k in removed:actual.pop(k,None)
 actual.update(overlay)
 if actual!=plan:raise ValueError('environment preservation failed')
 return overlay,sorted(REMOVED),tuple(removed)
