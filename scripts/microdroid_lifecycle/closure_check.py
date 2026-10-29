"""Pre-spawn immutable private assets and explicit OS-provider contract."""
import hashlib,json,os,pathlib,subprocess,sys
def sha(p):
    digest = hashlib.sha256()
    with pathlib.Path(p).open('rb') as f:
        while True:
            chunk = f.read(1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()
def check_manifest(m,gpu=True):
    for path,item in m['files'].items():
        p=pathlib.Path(path)
        if p.stat().st_size!=item['bytes'] or sha(p)!=item['sha256']:raise RuntimeError('runtime file changed: '+path)
    for path,target in m['links'].items():
        p=pathlib.Path(path);actual=os.readlink(p) if p.is_symlink() else str(p.resolve())
        if actual!=target:raise RuntimeError('dist link changed: '+path)
    for path,names in m['directories'].items():
        actual=sorted(x.name for x in pathlib.Path(path).iterdir())
        if actual!=names:raise RuntimeError('runtime directory inventory changed: '+path)
    s=m['system_contract']
    if m['platform']=='macos':
        if subprocess.check_output(['/usr/bin/sw_vers'],text=True,timeout=5)!=s['sw_vers']:raise RuntimeError('Mac OS provider changed')
    if m['platform']=='windows':
        import winreg
        if list(sys.getwindowsversion()[:3])!=s['windows_version']:raise RuntimeError('Windows OS provider changed')
        if gpu:
            x=s['nvidia_registry']
            with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE,x['key']) as key:
                for name,expected in [('DriverDesc',x['description']),('DriverVersion',x['version']),('VulkanDriverName',x['icd'])]:
                    if winreg.QueryValueEx(key,name)[0]!=expected:raise RuntimeError('Vulkan driver registration changed')
    return {'files':len(m['files']),'links':len(m['links']),'directories':len(m['directories']),'checked':'private files/OS provider contract','historical_limits':m['limitations']}
