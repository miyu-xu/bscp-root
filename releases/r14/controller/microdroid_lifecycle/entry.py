"""Microdroid foreground lifecycle candidate; deployment profile review is required."""
import argparse,hashlib,json,os,pathlib,platform,re,sys
from version import VERSION
ROOT=pathlib.Path(__file__).resolve().parent
HOST={'Windows':'windows','Linux':'linux','Darwin':'macos'}.get(platform.system(),'unsupported')

def need(ok,msg):
    if not ok:raise ValueError(msg)
def digest(p):
    digest = hashlib.sha256()
    with pathlib.Path(p).open('rb') as f:
        while True:
            chunk = f.read(1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()
def read(p):
    from strict_json import loads
    return loads(pathlib.Path(p).read_text(encoding='utf-8-sig'))
def no_links(p):
    for q in (p,*p.parents):
        need(not q.is_symlink() and not (hasattr(q,'is_junction') and q.is_junction()),'link/junction rejected')
def profile(name):
    need(name in ('windows','linux','macos'),'unknown platform')
    no_links(ROOT/'profiles'/f'{name}.json')
    p=read(ROOT/'profiles'/f'{name}.json')
    need(pathlib.Path(p['instance_root'])==ROOT/'instances','instance_root must equal state-root/instances; no historical takeover')
    need(isinstance(p['closure_manifest'],str) and '\\' not in p['closure_manifest'] and ':' not in p['closure_manifest'],'closure uses slash-only relative paths')
    manifest=pathlib.PurePosixPath(p['closure_manifest'])
    need(not manifest.is_absolute() and '..' not in manifest.parts and manifest.parts[0]=='closure','closure must be inside state-root/closure')
    no_links(ROOT/p['closure_manifest'])
    need(p['platform']==name and type(p['schema']) is int and p['schema']==1,'profile mismatch')
    need(p['debug']=='none' and p['gpu'] is True and p['payload_port']==5558,'unsupported capability')
    for path,h in p['hashes'].items():need(bool(re.fullmatch('[0-9a-f]{64}',h)),'invalid asset hash')
    return p
def parameters(name,cid,port,timeout):
    need(bool(re.fullmatch('[a-z][a-z0-9-]{0,23}',name)),'invalid name')
    need(type(cid) is int and 3<=cid<2**32-1,'invalid CID')
    need(type(port) is int and 1024<=port<=65535,'invalid forwarded port')
    need(type(timeout) is int and 1<=timeout<=300,'timeout outside 1..300 seconds')
def paths(p,name):
    cls=pathlib.PureWindowsPath if p['platform']=='windows' else pathlib.PurePosixPath
    root=cls(p['instance_root'])/name
    return {k:str(root/v) for k,v in {'root':'','state':'state','service':'svc','logs':'logs','temp':'tmp'}.items()}

def gpu_mode(cfg):
    # Missing mode is only the historical GPU-on configuration, never GPU-off.
    mode=cfg.get('gpu_mode','on')
    need(type(mode) is str and mode in ('on','off'),'invalid GPU mode')
    return mode

def verify_mode(record,plan,expected=None):
    mode=gpu_mode(record['config'])
    if record['schema']==1:
        need('gpu_mode' not in record['config'] and 'gpu_mode' not in plan,'mixed legacy mode record')
        need(mode=='on','legacy records are GPU-on only')
    else:
        need(record['schema']==2 and 'gpu_mode' in record['config'],'unsupported mode record')
        need(plan.get('gpu_mode')==mode,'instance GPU mode differs from creation plan')
        need(plan.get('config_binding')==record['config'],'instance configuration differs from creation plan')
    need(expected is None or expected==mode,'requested GPU mode differs from instance')
    return mode

def vm_configuration(p,mode):
    gpu_mode({'gpu_mode':mode})
    source=p['environment']['VIRTMGR_MICRODROID_JSON']
    # Profiles may use Windows slash variants for the same pinned source.
    canonical=lambda s:s.replace('\\','/').casefold() if p['platform']=='windows' else s
    pins=[h for path,h in p['hashes'].items() if canonical(path)==canonical(source)]
    need(len(pins)==1 and digest(source)==pins[0],'VM configuration source mismatch')
    value=read(source)
    gpu=value.get('gpu')
    need(isinstance(gpu,dict) and set(gpu)=={'enabled'} and gpu['enabled'] is True,'unexpected pinned GPU configuration')
    if mode=='off':value['gpu']={'enabled':False}
    return value

def mode_environment(p,mode):
    env=dict(p['environment'])
    if mode=='off':
        for key in list(env):
            if key.startswith(('VK_','GFXSTREAM_','ANDROID_EMU_','MVK_')):del env[key]
    return env
def launch_plan(p,cfg):
    mode=gpu_mode(cfg)
    parameters(cfg['name'],cfg['cid'],cfg['port'],cfg['timeout'])
    if p['platform']=='linux':need(cfg['cid']==2048,'fixed Linux service allocates2048; arbitrary CID not validated')
    d=paths(p,cfg['name']);sep='\\' if p['platform']=='windows' else '/'
    state=lambda f:d['state']+sep+f
    if p['platform']!='windows':
        limit=104 if p['platform']=='macos' else 108
        need(len((d['service']+'/virtmgr-service.sock').encode())<limit,'service Unix socket pathname too long')
    common=['--debug','none','--console',d['logs']+sep+'guest-console.txt','--storage',state('storage.img'),'--storage-size','67108864','--vsock-forward',f"{cfg['port']}:5558"]
    if p['platform']=='windows':
        argv=[p['vm'],'run-app',p['apk'],state('app.idsig'),state('instance.img'),'--payload-binary-name','libheadless_counter.so',*common]
    else:
        argv=['/bin/bash',p['launcher'],'--command','run-app','--dist-root',p['dist'],'--work-dir',d['state'],'--instance',state('instance.img'),'--idsig',state('app.idsig'),'--log-dir',d['logs'],'--temp-root',d['temp'],'--service-root',d['service'],'--persist-virtmgr','--no-root','--no-shell','--apk',p['apk'],'--payload-binary-name','libheadless_counter.so']
        if p['platform']=='macos':argv+=['--apex-tree-root',p['apex']]
        argv+=['--',*common]
    return {'argv':argv,'paths':d,'environment':mode_environment(p,mode),'gpu_mode':mode,'config_binding':dict(cfg),'requested_cid':cfg['cid'],'timeout_seconds':cfg['timeout'],'normal_stop':{'payload_request':'STOP\n','expected_reply':'BYE\n','transport':'existing forwarded vsock','guest_exit_required':0},'service_cleanup':p['ownership'],'execution':'PLAN_ONLY_NO_PROCESS_CREATED','automatic_recovery':False}
def state(record):
    # Persisted records alone cannot attest to a live process or authorize sending STOP.
    s=record.get('state')
    need(s in ('PREPARED','STARTING','RUNNING','STOPPING','STOPPED','FAILED','UNKNOWN'),'invalid state')
    if s in ('STARTING','RUNNING','STOPPING'):return 'UNKNOWN'
    if s=='STOPPED':
        e=record.get('exit',{})
        if not (type(e.get('guest_exit')) is int and e.get('guest_exit')==0 and e.get('stop_reply')=='BYE' and e.get('service_cleaned') is True and e.get('storage_released') is True and e.get('forced_guest') is False):return 'UNKNOWN'
    return s
def can_resume(record,snapshot):
    need(record.get('needsRecovery',False) is False,'recovery-required instance cannot resume')
    need(record.get('resource_state','REAPED')=='REAPED','resources not proven reaped')
    need(state(record)=='STOPPED','only proven healthy STOPPED can resume')
    need(bool(snapshot) and snapshot==record.get('storage_snapshot'),'state identity/content mismatch')
def check(p,mode='on'):
    need(HOST==p['platform'],'asset checks must run on matching host; this is not a remote executor')
    vm_configuration(p,mode)
    for path,h in p['hashes'].items():need(digest(path)==h,'asset mismatch: '+path)
    for item in p.get('layout',[]):need(pathlib.Path(item['source']).stat().st_size==item['bytes'] and digest(item['source'])==item['sha256'],'layout changed')
    from closure_check import check_manifest
    manifest=ROOT/p['closure_manifest'];need(digest(manifest)==p['closure_sha256'],'closure manifest changed')
    closure=check_manifest(read(manifest),gpu=(mode=='on'))
    return {'asset_hashes':'PASS','gpu_mode':mode,'closure':closure,'live_adapter':'READY_FOR_NATIVE_ACCEPTANCE' if p.get('runtime_profile_reviewed') else 'IMPLEMENTED_REVIEW_GATED','vm_started':False}
def config_dir(name):
    parameters(name,2048,45600,60)
    p=ROOT/'prepared'/name;no_links(p);return p
def create(p,args):
    cfg={'name':args.name,'cid':args.cid,'port':args.port,'timeout':args.timeout,'platform':p['platform'],'gpu_mode':args.gpu}
    plan=launch_plan(p,cfg);d=config_dir(args.name);need(not d.exists(),'existing preparation never overwritten')
    d.mkdir(parents=True)
    record={'candidate_version':VERSION,'schema':2,'state':'PREPARED','config':cfg,'profile_sha256':digest(ROOT/'profiles'/f"{p['platform']}.json"),'vm_started':False,'disks_created':False}
    # This local configuration is not a claim that a remote/native instance exists.
    with (d/'record.json').open('x') as f:json.dump(record,f,indent=2)
    with (d/'launch-plan.json').open('x') as f:json.dump(plan,f,indent=2)
    return record
def main():
    global ROOT
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--state-root',type=pathlib.Path,required=True);a.add_argument('--platform',choices=['windows','linux','macos'],required=True)
    sub=a.add_subparsers(dest='action',required=True);sub.add_parser('check').add_argument('--gpu',choices=['on','off'],default='on')
    c=sub.add_parser('create');c.add_argument('--name',required=True);c.add_argument('--cid',type=int,required=True);c.add_argument('--port',type=int,required=True);c.add_argument('--timeout',type=int,default=120)
    c.add_argument('--gpu',choices=['on','off'],required=True)
    for n in ('status','start','resume','stop'):
        q=sub.add_parser(n);q.add_argument('--name',required=True)
        q.add_argument('--gpu',choices=['on','off'],help='assert stored mode, never change it')
    args=a.parse_args();need(args.state_root.is_absolute(),'state-root must be absolute');no_links(args.state_root);need(args.state_root.is_dir(),'state-root must exist');ROOT=args.state_root;p=profile(args.platform)
    if args.action=='check':out=check(p,args.gpu)
    elif args.action=='create':out=create(p,args)
    else:
        d=config_dir(args.name);no_links(d/'record.json');r=read(d/'record.json')
        need(r.get('candidate_version')==VERSION,'candidate version mismatch; no historical instance takeover')
        need(r['config']['platform']==args.platform,'instance platform mismatch')
        need(r['profile_sha256']==digest(ROOT/'profiles'/f'{args.platform}.json'),'profile changed')
        no_links(d/'launch-plan.json');mode=verify_mode(r,read(d/'launch-plan.json'),args.gpu)
        if args.action=='status':out={'recorded_state':r['state'],'record_consistency_state':state(r),'live_state':'UNKNOWN','live_observation':'NOT_PERFORMED','vm_started_by_entry':False}
        elif args.action=='stop':
            raise RuntimeError('send stop on the owning start/resume controller stdin; no persisted-PID or cross-process takeover')
        else:
            import runtime
            out=runtime.run(p,r['config'],r,d,args.action=='resume',lambda p:check(p,mode),can_resume,lambda p,c,a,s:runtime.materialize(p,c,a,s,launch_plan),sys.stdin,profile_path=ROOT/'profiles'/f'{args.platform}.json')
    print(json.dumps(out,indent=2))
if __name__=='__main__':
    try:main()
    except Exception as e:print(json.dumps({'status':'BLOCKED_OR_FAILED','reason':str(e),'automatic_recovery':False}),file=sys.stderr);sys.exit(2)
