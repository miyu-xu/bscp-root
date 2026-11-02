"""Reviewed Windows Initial / ordinary run role contract, original Job only."""
import ctypes,pathlib,re
ARTIFACTS={
 'vm':'2b8201136e51f289973c3a8c2e1f15d05cff5026c86f87df48bbc812e8090dba',
 'virtmgr':'1ab2fe0af279a6d7257fc50b0dd187d4c9a65a3b0f2321c800e546a1b8104a7a',
 'crosvm':'d84910b25d70fd64f9093bd8d9ea235bbe6bb8ce44ac64327ff6d1cb5446442d'}
CONTRACT='a17-windows-initial-ordinary-run-v1'
APPROVED_ROOT=pathlib.PureWindowsPath('E:/bscp/out/aosp17-windows-formal-v1461/runtime-private-v1550')
STATE_ROOT=APPROVED_ROOT/'run-gpu-v1828'

def bad_component(value):
    return (value in ('','..','.') or value.endswith((' ','.')) or
        any(ord(c)<32 or c in '<>:"|?*' for c in value) or
        re.fullmatch(r'(CON|PRN|AUX|NUL|CONIN\$|CONOUT\$|COM[1-9¹²³]|LPT[1-9¹²³])(?:\..*)?',value,re.I) is not None)
def validate_layout(p):
    """Reject escape/aliasing before the first immutable layout file is copied."""
    rows=p.get('layout')
    if not isinstance(rows,list) or not rows:raise ValueError('complete Guest layout required')
    if pathlib.PureWindowsPath(p['instance_root'])!=STATE_ROOT/'instances':
        raise ValueError('SDK37 Initial instance root differs from approved private root')
    sources=APPROVED_ROOT/'assets'/'guest'
    seen=set()
    for row in rows:
        if not isinstance(row,dict) or set(row)!= {'source','relative','bytes','sha256'}:
            raise ValueError('invalid layout entry fields')
        rel=row['relative']
        if not isinstance(rel,str) or not rel or '\\' in rel or ':' in rel:
            raise ValueError('layout requires slash-only relative path')
        q=pathlib.PurePosixPath(rel)
        if q.is_absolute() or q.parts[0] not in ('apex','system') or any(bad_component(x) for x in rel.split('/')):
            raise ValueError('layout destination escape or alias')
        key=rel.casefold()
        if key in seen:raise ValueError('duplicate case-insensitive layout destination')
        seen.add(key)
        source=pathlib.PureWindowsPath(row['source'])
        if not source.is_absolute() or not source.is_relative_to(sources) or any(bad_component(x) for x in source.parts[1:]):
            raise ValueError('layout source outside private Guest assets')
        if type(row['bytes']) is not int or row['bytes']<0 or not isinstance(row['sha256'],str) or not re.fullmatch('[0-9a-f]{64}',row['sha256']):
            raise ValueError('invalid layout size/hash')
def require_initial_contract(p):
    if p.get('windows_service_mode')!='initial-original-handle':
        raise ValueError('Windows SDK37 requires Initial original HANDLE ownership')
    if 'VIRTMGR_SERVICE_DIR' in p.get('environment',{}):
        raise ValueError('Initial profile cannot request persistent discovery')
def require_final_topology(p):
    from entry import digest
    if p.get('windows_process_contract')!=CONTRACT:
        raise ValueError('missing reviewed ordinary-run topology contract')
    for role,expected in ARTIFACTS.items():
        if expected is None:raise ValueError('final '+role+' artifact pending correctness review')
        source=p.get(role)
        if not source or p.get('hashes',{}).get(source)!=expected or digest(source)!=expected:
            raise ValueError('final '+role+' artifact binding mismatch')

def original_parent(member):
    """Parent ID from the retained process HANDLE, never reopen a PID."""
    class PBI(ctypes.Structure):
        _fields_=[('reserved1',ctypes.c_void_p),('peb',ctypes.c_void_p),
          ('reserved2',ctypes.c_void_p*2),('pid',ctypes.c_size_t),('parent',ctypes.c_size_t)]
    q=ctypes.WinDLL('ntdll').NtQueryInformationProcess
    q.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.c_void_p,ctypes.c_ulong,ctypes.POINTER(ctypes.c_ulong)]
    q.restype=ctypes.c_long;data=PBI();length=ctypes.c_ulong()
    status=q(member._h,0,ctypes.byref(data),ctypes.sizeof(data),ctypes.byref(length))
    if status!=0:raise OSError('original process parent query failed: '+str(status))
    if data.pid!=member.initial['pid']:raise RuntimeError('original HANDLE identity changed')
    return int(data.parent)

def observe_roles(family,p,canon,parent_query=original_parent):
    """Historical members retained; READY requires all three primary roles live.

    Ordinary run has one crosvm and no broker. No-VM production launcher
    observation establishes one optional console host parented to original root;
    any additional/differently parented helper is rejected, not auto-enrolled.
    """
    roles={k:[] for k in ('vm','virtmgr','crosvm','console')}
    paths={canon(p[k]):k for k in ('vm','virtmgr','crosvm')}
    paths[canon('C:/Windows/System32/conhost.exe')]='console'
    for m in family.members.values():
        role=paths.get(m.initial['exe_path'])
        if role is None:raise RuntimeError('unknown original Job image role')
        if role=='vm' and m is not family.root:raise RuntimeError('additional vm is not original root')
        roles[role].append(m)
        if len(roles[role])>1:raise RuntimeError('additional '+role+' member exceeds ordinary-run contract')
    if roles['vm']!=[family.root]:raise RuntimeError('original vm root missing')
    states={id(m):m.wait(0) for group in roles.values() for m in group}
    evidence=[{'role':'vm','identity':dict(family.root.initial),'original_root':True,
               'observation':dict(states[id(family.root)])}]
    for role in ('virtmgr','crosvm','console'):
        parent_role='virtmgr' if role=='crosvm' else 'vm'
        for m in roles[role]:
            if len(roles[parent_role])!=1:raise RuntimeError('missing original '+parent_role+' parent')
            parent=roles[parent_role][0];pid=parent_query(m)
            if pid!=parent.initial['pid'] or m.initial['creation_filetime']<parent.initial['creation_filetime']:
                raise RuntimeError('original '+role+' parent/generation mismatch')
            evidence.append({'role':role,'identity':dict(m.initial),'parent_identity':dict(parent.initial),
                             'observation':dict(states[id(m)])})
    ready=all(len(roles[k])==1 and states[id(roles[k][0])]['state']=='RUNNING' for k in ('vm','virtmgr','crosvm'))
    guest_ended=len(roles['crosvm'])==1 and states[id(roles['crosvm'][0])]['state']=='EXITED'
    if states[id(family.root)]['state']!='RUNNING' and not guest_ended:
        raise RuntimeError('Initial vm owner ended before Guest exit proof')
    if roles['crosvm'] and not guest_ended and states[id(roles['virtmgr'][0])]['state']!='RUNNING':
        raise RuntimeError('Initial service ended before Guest exit proof')
    return roles,{'contract':CONTRACT,'ready_roles_live':ready,'members':evidence}
