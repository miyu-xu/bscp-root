"""Foreground lifetime binding. No historical PID attachment or control listener."""
import json,os,pathlib,shutil,socket,uuid,time,queue,errno,stat
from strict_json import loads
from lifecycle import supervise,stdin_commands
from native import WindowsOwner,PosixOwner,sha

def refresh_terminal_receipts(result,owner):
    # Observation-only update after delayed terminal proof. Never change UNKNOWN
    # or replace the original exception with cleanup observations.
    result['pipe_log']=getattr(owner,'log_receipt',None)
    if getattr(owner,'cleanup',None):result['cleanup_receipt']=owner.cleanup
    if owner.p['platform']=='windows' and owner.owner_terminal:
        receipt=owner.terminal_resource_receipt()
        if receipt is not None:
            result.update(resource_state='REAPED',storage_snapshot=receipt['storage_snapshot'],
                          storage_state='RELEASED',storage_released=True,failure_cleanup=receipt)
    if getattr(owner,'fault_owner',None) is not None and owner.owner_terminal:
        receipt=owner.cleanup
        if receipt.get('resource_state')=='REAPED' and receipt.get('storage_snapshot'):
            result['resource_state']='REAPED';result['storage_snapshot']=receipt['storage_snapshot']
            result['storage_released']=True;result['storage_state']='RELEASED'
            result['failure_cleanup']={'owner_terminal':True,'resource_state':'REAPED','storage_snapshot':receipt['storage_snapshot'],'receipt':receipt,'no_repeat':True}
            result['forced_macos_cleanup' if owner.p['platform']=='macos' else 'forced_linux_cleanup']=True;result['forced_guest']=None;result['normal_guest_exit_proven']=False


def local_path(p):
    for q in (p,*p.parents):
        if q.is_symlink() or (hasattr(q,'is_junction') and q.is_junction()):raise RuntimeError('instance path link/junction')

def windows_layout(p,root,resume):
    p=dict(p);tree=root/'apex-tree'
    for x in p['layout']:
        q=tree/pathlib.PureWindowsPath(x['relative']);local_path(q)
        source=pathlib.Path(x['source'])
        if source.stat().st_size!=x['bytes'] or sha(source)!=x['sha256']:raise RuntimeError('immutable layout source changed')
        if not resume:
            q.parent.mkdir(parents=True,exist_ok=True)
            with source.open('rb') as src,q.open('xb') as dst:shutil.copyfileobj(src,dst,1024*1024)
        if q.stat().st_size!=x['bytes'] or sha(q)!=x['sha256']:raise RuntimeError('instance layout changed')
    p.update(apex=str(tree/'apex'),system=str(tree/'system'),system_ext=str(root/'system_ext'))
    if not resume:(root/'system_ext').mkdir()
    return p

def probe_port(port):
    # Availability only, never a listener or reservation. Keep Windows exclusivity.
    with socket.socket() as probe:
        if hasattr(socket,'SO_EXCLUSIVEADDRUSE'):
            probe.setsockopt(socket.SOL_SOCKET,socket.SO_EXCLUSIVEADDRUSE,1)
        probe.bind(('127.0.0.1',port))

def await_port(port,seconds=90,clock=time.monotonic,sleep=time.sleep):
    deadline=clock()+seconds
    while True:
        try:probe_port(port);return
        except OSError as exc:
            if exc.errno!=errno.EADDRINUSE:raise
            remaining=deadline-clock()
            if remaining<=0:raise TimeoutError('forwarded port unavailable; no instance mutation') from exc
            sleep(min(1,remaining))

def release_prelaunch_claim(claim,identity,token):
    # Only our newly-created claim, before any launch/filesystem preparation.
    current=claim.lstat()
    if not stat.S_ISREG(current.st_mode) or (current.st_dev,current.st_ino)!=identity or claim.read_bytes()!=token:
        raise RuntimeError('prelaunch claim identity changed; retained, no takeover')
    claim.unlink()

def run(p,cfg,record,record_dir,resume,check,can_resume,plan_factory,stream,profile_path=None):
    # This preflight is deliberately before claims, disks, or process creation.
    check(p)
    if not p.get('runtime_profile_reviewed'):
        raise RuntimeError('runtime profile/namespace closure has not passed independent review')
    from entry import read,verify_mode
    mode_plan=record_dir/'launch-plan.json';local_path(mode_plan)
    mode_plan_bytes=mode_plan.read_bytes()
    verify_mode(record,read(mode_plan))
    root=pathlib.Path(p['instance_root'])/cfg['name']
    local_path(root)
    # Failure here must not turn a healthy stopped instance into UNKNOWN.
    record_path=record_dir/'record.json'
    before=record_path.read_bytes()
    if json.dumps(loads(before),sort_keys=True)!=json.dumps(record,sort_keys=True):
        raise RuntimeError('record changed before preflight')
    if record['state']!=('STOPPED' if resume else 'PREPARED'):
        raise RuntimeError('start/resume record is not eligible')
    expected_profile=record_dir.parent.parent/'profiles'/f"{p['platform']}.json"
    if record_dir.parent.name!='prepared' or record_dir.name!=cfg['name']:
        raise RuntimeError('invalid record directory layout')
    if profile_path is None:profile_path=expected_profile
    if pathlib.Path(profile_path)!=expected_profile:
        raise RuntimeError('profile/record root mismatch')
    if sha(profile_path)!=record['profile_sha256']:
        raise RuntimeError('profile changed before preflight')
    claim=record_dir/'owner.claim'
    try:claim.lstat()
    except FileNotFoundError:pass
    else:raise RuntimeError('owner claim exists; no takeover')
    await_port(cfg['port'])
    token=uuid.uuid4().hex.encode()
    fd=os.open(claim,os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600)
    identity=None
    try:
        info=os.fstat(fd);identity=(info.st_dev,info.st_ino)
        if os.write(fd,token)!=len(token):raise OSError('short owner claim write; retained')
        os.fsync(fd)
    finally:os.close(fd)
    try:
        # Revalidate after waiting/acquiring ownership; do not trust stale state.
        if record_path.read_bytes()!=before or sha(profile_path)!=record['profile_sha256'] or mode_plan.read_bytes()!=mode_plan_bytes:
            raise RuntimeError('record/profile changed while waiting; no launch')
        probe_port(cfg['port'])
    except BaseException:
        release_prelaunch_claim(claim,identity,token)
        raise
    # Claim is intentionally retained on failure, including controller death.
    def save(value):
        temporary=record_dir/'record.next'
        with temporary.open('x',encoding='utf-8') as f:
            json.dump(value,f,indent=2);f.flush();os.fsync(f.fileno())
        os.replace(temporary,record_dir/'record.json')
    try:
        if resume:
            if not root.is_dir():raise RuntimeError('missing healthy instance')
        else:
            if record['state']!='PREPARED':raise RuntimeError('start requires PREPARED')
            root.mkdir(parents=True,exist_ok=False)
            (root/'state').mkdir()
        local_path(root/'state');local_path(root/'system_ext')
        needed=sum(x['bytes'] for x in p.get('layout',[])) if not resume else 0
        if shutil.disk_usage(root).free < 10*1024**3+needed:raise RuntimeError('physical free reserve below10GiB+layout')
        if p['platform']=='windows':p=windows_layout(p,root,resume)
        attempt=root/('r'+uuid.uuid4().hex[:6]);attempt.mkdir()
        for name in ('service','logs','tmp','console'): (attempt/name).mkdir()
        plan=plan_factory(p,cfg,attempt,root/'state')
        if p['platform']=='macos':
            from mac_owner import MacOwner
            owner=MacOwner(p,cfg,plan,attempt,root/'state')
        else:owner=(WindowsOwner if p['platform']=='windows' else PosixOwner)(p,cfg,plan,attempt,root/'state')
        if resume:can_resume(record,owner.snapshot())
        # Port was checked again under our claim before any instance mutation.
        # Binding can still race afterwards; actual launch failures stay UNKNOWN.
        record.update(state='STARTING',attempt=str(attempt),vm_started=True)
        save(record)
        def emit(result):
            record.update(state=result['state'],exit=result,storage_snapshot=result.get('storage_snapshot'),
                          task_result=result.get('task_result','UNKNOWN'),resource_state=result.get('resource_state','UNKNOWN'),
                          needsRecovery=result.get('needsRecovery',True))
            save(record)
            print(json.dumps(result),flush=True)
        commands=stdin_commands(stream)
        result=supervise(owner,commands,cfg['timeout'],emit)
        if result['state']=='UNKNOWN' and not owner.controller_release_ready():
            # Cleanup attempts are bounded. Unknown OS completion is not permission
            # to discard live HANDLEs. Retain this exact owner until observation
            # proves natural Guest termination; EOF/interrupt is not that proof.
            result['controller_retained']=True
            result['retention_notice']='UNKNOWN: owner retained; external producer logs can grow; use status, or stop only if no request outcome is unknown. No forced cleanup retry.'
            try:print(json.dumps(result),flush=True)
            except OSError:pass
            while True:
                try:
                    if owner.finalize_if_terminal():
                        result['owner_terminal']=True
                        if owner.controller_release_ready():break
                    try:command=commands.get(timeout=1)
                    except queue.Empty:continue
                    if command=='status':print(json.dumps(owner.retention_status()),flush=True)
                    elif command=='stop' and not owner.sent_stop and not owner.transport_unknown:
                        # One normal request through the current owner's verified
                        # endpoint. Result stays UNKNOWN regardless of reply.
                        result['retained_stop_reply']=owner.stop()
                    else:time.sleep(.2)
                except BaseException as observation_error:
                    result['retention_error']=repr(observation_error);time.sleep(1)
            refresh_terminal_receipts(result,owner)
            emit(result)
        if result['state']=='STOPPED':claim.unlink()
        return result
    except BaseException as exc:
        record.update(state='UNKNOWN',resource_state='UNKNOWN',needsRecovery=True,error=repr(exc));save(record)
        raise

def materialize(p,cfg,attempt,state,base_plan):
    """Only rewrite per-run paths; immutable assets remain selected by profile."""
    plan=base_plan(p,cfg)
    old=plan['paths'];replacements={old['state']:str(state),old['service']:str(attempt/'service'),old['logs']:str(attempt/'logs'),old['temp']:str(attempt/'tmp')}
    def translate(value):
        for a,b in replacements.items():value=value.replace(a,b)
        return value
    plan['argv']=[translate(v) for v in plan['argv']]
    if p['platform']=='windows':plan['argv']=[v.replace('app.idsig','payload.idsig') for v in plan['argv']]
    clean={k:v for k,v in os.environ.items() if k not in ('BASH_ENV','ENV','SHELLOPTS','BASHOPTS','ANGLE_DEFAULT_PLATFORM') and not k.startswith(('BASH_FUNC_','BSCP_','GFXSTREAM_','VIRTMGR_','VMCLIENT_','ANDROID_EMU_','MVK_','VK_','DYLD_','LD_'))}
    from entry import gpu_mode,mode_environment,vm_configuration
    mode=gpu_mode(cfg)
    clean.update(mode_environment(p,mode))
    if mode=='off':
        # Existing VmConfig supports explicit disabled; this is an attempt-local
        # derived input, never a modification of the fixed source or Guest image.
        value=vm_configuration(p,mode)
        target=pathlib.Path(attempt)/'microdroid-gpu-off.json'
        local_path(target)
        with target.open('x',encoding='utf-8') as f:
            json.dump(value,f,sort_keys=True,indent=2);f.flush();os.fsync(f.fileno())
        plan['vm_configuration']={'path':str(target),'sha256':sha(target),'gpu_mode':mode}
        clean['VIRTMGR_MICRODROID_JSON']=str(target)
    clean.update(VIRTMGR_GUEST_CID=str(cfg['cid']),VIRTMGR_SERVICE_DIR=str(attempt/'service'),VIRTMGR_TRACE_FILE=str(attempt/'service'/'virtmgr-trace.log'),VMCLIENT_TRACE_FILE=str(attempt/'client-trace.log'),VIRTMGR_CONSOLE_LOG_DIR=str(attempt/'console'))
    if p['platform']=='windows':
        clean.update(VIRTMGR_PATH=p['virtmgr'],VIRTMGR_CROSVM_PATH=p['crosvm'],VIRTMGR_APEX_ROOT=p['apex'],VIRTMGR_SYSTEM_ROOT=p['system'],VIRTMGR_SYSTEM_EXT_ROOT=p['system_ext'],VIRTMGR_KEEP_TEMP='1',ANDROID_PROP_RO_BUILD_VERSION_SDK='35',TMP=str(attempt/'tmp'),TEMP=str(attempt/'tmp'))
        clean['PATH']=p['path_prefix']+';C:/Windows/System32;C:/Windows'
    else:clean['PATH']='/usr/bin:/bin:/usr/sbin:/sbin'
    if p['platform']!='windows' and len(str(attempt/'service'/'virtmgr-service.sock').encode())>=(104 if p['platform']=='macos' else 108):raise RuntimeError('per-attempt socket pathname too long')
    plan['environment']=clean;plan['execution']='FOREGROUND_OWNER';return plan
