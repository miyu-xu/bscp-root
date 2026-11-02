"""Read-only check of an already admitted AOSP17 deployment. Never installs or starts a VM."""
from pathlib import Path
import argparse,hashlib,json,platform,subprocess,sys

def load(path):
    def unique(pairs):
        out={}
        for k,v in pairs:
            if k in out:raise ValueError('duplicate JSON key')
            out[k]=v
        return out
    return json.loads(path.read_bytes(),object_pairs_hook=unique)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--platform',choices=['windows','linux','macos'],required=True)
    a=p.parse_args();actual={'Windows':'windows','Linux':'linux','Darwin':'macos'}.get(platform.system())
    if actual!=a.platform:raise ValueError('must run on matching platform')
    here=Path(__file__).resolve().parent;d=here/'deployment-profiles'/a.platform;b=load(d/'BINDING.json')
    state=Path(b['admitted_state_root'])
    for name,key in [('profile','profile_path'),('closure','closure_path')]:
        local=d/(name+'.json');installed=Path(b[key])
        if local.is_symlink() or installed.is_symlink():raise ValueError('unexpected profile link')
        expected=b[name+'_sha256']
        if any(hashlib.sha256(x.read_bytes()).hexdigest()!=expected for x in [local,installed]):raise ValueError(name+' binding mismatch')
    profile=load(d/'profile.json');closure=load(d/'closure.json')
    if Path(profile['instance_root'])!=state/'instances':raise ValueError('state binding mismatch')
    if b['expected_host_sha256'] not in [v['sha256'] for v in closure['files'].values()]:raise ValueError('Host identity missing')
    results=[]
    for gpu in ['on','off']:
        cmd=[sys.executable,'-B',str(here/'controllers'/a.platform/'microdroid.py'),'--platform',a.platform,'--state-root',str(state),'check','--gpu',gpu]
        r=subprocess.run(cmd,capture_output=True)
        if r.returncode:raise RuntimeError('production check failed: '+r.stderr.decode(errors='replace'))
        value=json.loads(r.stdout)
        if value.get('asset_hashes')!='PASS' or value.get('vm_started') is not False:raise ValueError('unexpected production check receipt')
        results.append({'gpu':gpu,'exit':r.returncode,'stdout_stderr_eof':True,'receipt':value})
    print(json.dumps({'platform':a.platform,'state_root':str(state),'checks':results,'vm_started':False,'security_settings_changed':False,'scope':'Existing admitted deployment only; no fresh installation or new runtime acceptance'},indent=2))

if __name__=='__main__':
    main()
