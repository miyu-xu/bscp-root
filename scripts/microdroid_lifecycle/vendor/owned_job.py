"""Small Windows owned-process-family primitive. No PID attach or automatic kill.

Manifest hash is a caller-supplied review pin, NOT an authorization mechanism.
Unknown outcomes retain handles in ACTIVE; there is no destructor cleanup.
"""
import ctypes as C
from ctypes import wintypes as W
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import subprocess
import _winapi
import threading
from functools import wraps

K=C.WinDLL('kernel32',use_last_error=True)
def api(n,args,ret):
    f=getattr(K,n);f.argtypes=args;f.restype=ret;return f
api('CreateJobObjectW',[W.LPVOID,W.LPCWSTR],W.HANDLE)
api('AssignProcessToJobObject',[W.HANDLE,W.HANDLE],W.BOOL)
api('IsProcessInJob',[W.HANDLE,W.HANDLE,C.POINTER(W.BOOL)],W.BOOL)
api('QueryInformationJobObject',[W.HANDLE,C.c_int,W.LPVOID,W.DWORD,C.POINTER(W.DWORD)],W.BOOL)
api('OpenProcess',[W.DWORD,W.BOOL,W.DWORD],W.HANDLE)
api('CloseHandle',[W.HANDLE],W.BOOL)
api('ResumeThread',[W.HANDLE],W.DWORD)
api('GetProcessId',[W.HANDLE],W.DWORD)
api('GetProcessTimes',[W.HANDLE]+[C.POINTER(W.FILETIME)]*4,W.BOOL)
api('QueryFullProcessImageNameW',[W.HANDLE,W.DWORD,W.LPWSTR,C.POINTER(W.DWORD)],W.BOOL)
api('WaitForSingleObject',[W.HANDLE,W.DWORD],W.DWORD)
api('GetExitCodeProcess',[W.HANDLE,C.POINTER(W.DWORD)],W.BOOL)
api('TerminateProcess',[W.HANDLE,W.UINT],W.BOOL)
api('GetCurrentProcess',[],W.HANDLE)
api('GetFileType',[W.HANDLE],W.DWORD)
api('DuplicateHandle',[W.HANDLE,W.HANDLE,W.HANDLE,C.POINTER(W.HANDLE),W.DWORD,W.BOOL,W.DWORD],W.BOOL)
api('K32EnumProcessModulesEx',[W.HANDLE,C.POINTER(W.HMODULE),W.DWORD,C.POINTER(W.DWORD),W.DWORD],W.BOOL)
api('K32GetModuleFileNameExW',[W.HANDLE,W.HMODULE,W.LPWSTR,W.DWORD],W.DWORD)

ACTIVE={}
ATTEMPTS={}
CREATE_LOCK=threading.Lock()
MAX_MEMBERS=128
def serialized(method):
    @wraps(method)
    def invoke(self,*a,**kw):
        family=getattr(self,'_family',self)
        with family._lock:return method(self,*a,**kw)
    return invoke
def check(v):
    if not v:raise C.WinError(C.get_last_error())
    return v
def sha(p):
    with open(p,'rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def canon(p):return os.path.normcase(str(Path(p).resolve(strict=True)))


class JOB_BASIC_LIMIT(C.Structure):
    _fields_=[('per_process',C.c_longlong),('per_job',C.c_longlong),('flags',W.DWORD),('min_ws',C.c_size_t),('max_ws',C.c_size_t),('active',W.DWORD),('affinity',C.c_size_t),('priority',W.DWORD),('scheduling',W.DWORD)]
class JOB_IO(C.Structure):
    _fields_=[(n,C.c_ulonglong) for n in ['read_ops','write_ops','other_ops','read_bytes','write_bytes','other_bytes']]
class JOB_LIMIT(C.Structure):
    _fields_=[('basic',JOB_BASIC_LIMIT),('io',JOB_IO),('process_memory',C.c_size_t),('job_memory',C.c_size_t),('peak_process',C.c_size_t),('peak_job',C.c_size_t)]
api('SetInformationJobObject',[W.HANDLE,C.c_int,W.LPVOID,W.DWORD],W.BOOL)
api('TerminateJobObject',[W.HANDLE,W.UINT],W.BOOL)
def set_test_job_containment(job):
    limit=JOB_LIMIT();limit.basic.flags=0x2000 # KILL_ON_JOB_CLOSE, no BREAKAWAY flags
    check(K.SetInformationJobObject(job,9,C.byref(limit),C.sizeof(limit)))
    actual=JOB_LIMIT();returned=W.DWORD()
    check(K.QueryInformationJobObject(job,9,C.byref(actual),C.sizeof(actual),C.byref(returned)))
    if actual.basic.flags!=0x2000:raise RuntimeError('unexpected test Job limits')

class OwnedSpawnError(RuntimeError):
    """Original failure plus exact retained attempt, never a PID reconstruction."""
    def __init__(self,original,family,phase):
        self.original=original;self.family=family;self.phase=phase
        self.pid=family.created_pid;self.tid=family.created_tid
        self.resume_attempted=family.resume_attempted
        super().__init__('owned spawn failed at '+phase+'; retained; no retry: '+repr(original))

@dataclass(frozen=True)
class ReviewedCommand:
    manifest_sha256:str
    executable:str
    args:tuple
    cwd:str
    allowed_images:tuple
    env_set:tuple
    env_unset:tuple
    @classmethod
    def from_file(cls,path,expected_sha256):
        raw=Path(path).read_bytes()
        if hashlib.sha256(raw).hexdigest()!=expected_sha256:raise ValueError('manifest pin mismatch')
        from strict_json import loads
        d=loads(raw)
        if set(d)!={'version','executable','args','cwd','allowed_images','env_set','env_unset'} or d['version']!=1:raise ValueError('manifest schema')
        if not Path(d['executable']).is_absolute() or not Path(d['cwd']).is_absolute():raise ValueError('absolute paths required')
        if not isinstance(d['args'],list) or any(not isinstance(x,str) or '\0' in x for x in d['args']):raise ValueError('args')
        if not isinstance(d['env_set'],dict) or not isinstance(d['env_unset'],list):raise ValueError('environment')
        for k,v in d['env_set'].items():
            if not isinstance(k,str) or not isinstance(v,str) or not k or '=' in k or '\0' in k+v:raise ValueError('environment')
        for k in d['env_unset']:
            if not isinstance(k,str) or not k or '=' in k or '\0' in k:raise ValueError('environment')
        images=[]
        for p,h in d['allowed_images'].items():
            if not Path(p).is_absolute() or sha(p)!=h:raise ValueError('image pin mismatch')
            images.append((canon(p),h))
        exe=canon(d['executable']);cwd=canon(d['cwd'])
        if not Path(cwd).is_dir() or exe not in dict(images):raise ValueError('root not allowed')
        return cls(expected_sha256,exe,tuple(d['args']),cwd,tuple(images),tuple(d['env_set'].items()),tuple(d['env_unset']))

def identity(h,record=None):
    def mark(name):
        if record:record(name)
    t=[W.FILETIME() for _ in range(4)]
    mark('GetProcessTimes')
    check(K.GetProcessTimes(h,*(C.byref(x) for x in t)))
    b=C.create_unicode_buffer(32768);n=W.DWORD(len(b))
    mark('QueryFullProcessImageNameW')
    check(K.QueryFullProcessImageNameW(h,0,b,C.byref(n)))
    mark('GetProcessId')
    pid=check(K.GetProcessId(h))
    mark('filesystem:canonicalize_image');image=canon(b.value)
    mark('filesystem:hash_image');image_sha=sha(b.value)
    return {'pid':int(pid),'creation_filetime':(t[0].dwHighDateTime<<32)|t[0].dwLowDateTime,
            'exe_path':image,'exe_sha256':image_sha}

class OwnedMember:
    def __init__(self,family,h,initial):
        self._family=family;self._h=h;self.initial=initial;self.terminate_attempted=False
    @serialized
    def verify(self,expected):
        if self._family.closed or expected!=self.initial:raise ValueError('exact retained identity required')
        now=identity(self._h)
        if now!=self.initial:raise RuntimeError('retained handle identity changed')
        self._family._require_member(self._h)
        return now
    @serialized
    def wait(self,ms=0):
        if type(ms)is not int or not 0<=ms<=10000:raise ValueError('bounded wait 0..10000')
        if self._family.closed or self._h is None:raise RuntimeError('closed process handle')
        self._family.last_operation={'api':'WaitForSingleObject','target':dict(self.initial),'retained_handle':True,'timeout_ms':ms}
        r=K.WaitForSingleObject(self._h,ms)
        if r==258:return {'state':'RUNNING'}
        if r!=0:raise C.WinError(C.get_last_error())
        self._family.last_operation={'api':'GetExitCodeProcess','target':dict(self.initial),'retained_handle':True}
        code=W.DWORD();check(K.GetExitCodeProcess(self._h,C.byref(code)))
        return {'state':'EXITED','exit_code':code.value,'terminate_requested':self.terminate_attempted}
    @serialized
    def modules(self):
        before=self.verify(self.initial)
        a=(W.HMODULE*512)();needed=W.DWORD()
        check(K.K32EnumProcessModulesEx(self._h,a,C.sizeof(a),C.byref(needed),3))
        if needed.value>C.sizeof(a) or needed.value%C.sizeof(W.HMODULE):raise RuntimeError('module capacity')
        rows=[]
        for i in range(needed.value//C.sizeof(W.HMODULE)):
            b=C.create_unicode_buffer(32768);n=check(K.K32GetModuleFileNameExW(self._h,a[i],b,len(b)))
            if n>=len(b)-1:raise RuntimeError('module path truncated')
            rows.append({'base':int(a[i]),'path':b.value,'file_sha256':sha(b.value)})
        if before!=self.verify(self.initial):raise RuntimeError('identity changed')
        return {'process':before,'modules':rows,'scope':'loaded module paths; file hashes, NOT mapped-memory hashes; non-atomic snapshot'}
    @serialized
    def terminate(self,expected,*,authorization,reason):
        """Explicit per-member scoped decision, never inferred from a timeout."""
        if authorization is not True or not isinstance(reason,str) or not reason.strip():raise ValueError('explicit authorization and reason required')
        self.verify(expected)
        if self.terminate_attempted:raise RuntimeError('termination outcome reserved; no replay')
        if self.wait()['state']=='EXITED':return {'state':'ALREADY_EXITED'}
        self.terminate_attempted=True
        self._family.events.append({'action':'TerminateProcess','identity':self.initial,'reason':reason,'state':'ATTEMPT_RESERVED'})
        check(K.TerminateProcess(self._h,71))
        return {'state':'REQUESTED_NOT_EXIT_PROOF'}

class OwnedFamily:
    def __init__(self):raise TypeError('use spawn_reviewed')
    @classmethod
    def spawn_reviewed(cls,command,*,stdio=None):
        if type(command)is not ReviewedCommand:raise TypeError('ReviewedCommand required')
        for p,h in command.allowed_images:
            if sha(p)!=h:raise ValueError('image changed before spawn')
        self=object.__new__(cls);self.command=command;self.closed=False;self.state='PREPARED'
        self._lock=threading.RLock()
        self._job=None;self._thread=None;self._root_handle=None;self.members={};self.events=[]
        self.created_pid=None;self.created_tid=None;self.created_identity=None
        self.create_attempted=False;self.create_failed=False;self.creation_receipt=None;self.assigned=False
        self.resume_attempted=False;self.abort_attempted=False
        with CREATE_LOCK:
            if command.manifest_sha256 in ATTEMPTS:raise RuntimeError('manifest already attempted; inspect original owner, never replay')
            ATTEMPTS[command.manifest_sha256]=self
        # Dedicated test Job: kill-on-last-handle-close, without breakaway.
        ACTIVE[id(self)]=self
        env=os.environ.copy()
        for k in command.env_unset:env.pop(k,None)
        env.update(command.env_set)
        inherited=[];self.inherited_handles=inherited;self.inherited_close_errors=[]
        try:
            self.state='JOB_CREATE_ATTEMPTED'
            self._job=check(K.CreateJobObjectW(None,None))
            set_test_job_containment(self._job)
            startup=subprocess.STARTUPINFO()
            if stdio is not None:
                # Caller owns bounded pipe readers. Only duplicate the three exact
                # supplied standard handles, never all inheritable parent handles.
                if type(stdio)is not tuple or len(stdio)!=3:raise ValueError('stdio=(stdin,stdout,stderr) handles')
                current=K.GetCurrentProcess()
                for handle in stdio:
                    if type(handle)is not int or K.GetFileType(handle) not in (2,3):raise ValueError('stdio requires valid character/pipe handles')
                    duplicate=W.HANDLE();check(K.DuplicateHandle(current,handle,current,C.byref(duplicate),0,True,2))
                    inherited.append(duplicate.value)
                startup.dwFlags|=subprocess.STARTF_USESTDHANDLES
                startup.hStdInput,startup.hStdOutput,startup.hStdError=inherited
                startup.lpAttributeList={'handle_list':inherited}
            self.create_attempted=True;self.state='CREATE_ATTEMPTED'
            try:
                h,t,pid,tid=_winapi.CreateProcess(command.executable,subprocess.list2cmdline([command.executable,*command.args]),None,None,bool(inherited),
                    0x4|0x08000000,env,command.cwd,startup)
            except OSError:
                self.create_failed=True;raise # Win32 call returned failure, no child handles.
            self._root_handle=h;self._thread=t;self.state='SUSPENDED_RETAINED'
            self.created_pid=pid;self.created_tid=tid
            self.creation_receipt=(h,t,pid,tid,command.manifest_sha256)
            init=identity(h)
            if init['pid']!=pid or init['exe_path']!=command.executable or init['exe_sha256']!=dict(command.allowed_images)[command.executable]:raise RuntimeError('created root mismatch')
            self.created_identity=dict(init)
            # Nested-job incompatibility is terminal for this attempt; no unjobbed fallback.
            self.state='ASSIGN_ATTEMPTED'
            check(K.AssignProcessToJobObject(self._job,h));self.assigned=True;self._require_member(h)
            self.root=OwnedMember(self,h,init);self.members[(pid,init['creation_filetime'])]=self.root
            self.state='ASSIGNED_SUSPENDED'
            self.resume_attempted=True;self.state='RESUME_ATTEMPTED'
            previous=K.ResumeThread(t)
            if previous!=1:raise RuntimeError('resume unknown/unexpected suspend count: '+str(previous))
            self.state='RUNNING';return self
        except BaseException as exc:
            import traceback
            self.spawn_failure={'phase':self.state,'exception':repr(exc),'traceback':traceback.format_exc()}
            phase=self.state
            self.state='UNKNOWN_RETAINED_NO_RETRY'
            raise OwnedSpawnError(exc,self,phase) from exc
        finally:
            for index,h in enumerate(inherited):
                if not K.CloseHandle(h):
                    exc=C.WinError(C.get_last_error())
                    self.inherited_close_errors.append({'handle':int(h),'error':repr(exc)})
                else:inherited[index]=None
            if self.inherited_close_errors:
                self.state='UNKNOWN_RETAINED_NO_RETRY'
                raise OwnedSpawnError(RuntimeError(repr(self.inherited_close_errors)),self,'STDIO_DUPLICATE_CLOSE')
    @serialized
    def abort_created_suspended(self,expected,*,authorization,reason):
        """Narrow pre-Resume cleanup of this exact successful CreateProcess.

        Not an unjobbed runtime fallback: never allowed after Resume was attempted.
        Can only terminate the still-suspended root through its original HANDLE.
        """
        if authorization is not True or not isinstance(reason,str) or not reason.strip():raise ValueError('explicit suspended-root cleanup authorization required')
        if self.closed or self.resume_attempted or self.abort_attempted:raise RuntimeError('not a fresh known pre-Resume root; retain')
        receipt=getattr(self,'creation_receipt',None)
        if receipt is None or expected!=receipt or receipt!=(self._root_handle,self._thread,self.created_pid,self.created_tid,self.command.manifest_sha256):raise ValueError('original successful suspended CreateProcess receipt required')
        r=K.WaitForSingleObject(self._root_handle,0)
        if r==0:return {'state':'ALREADY_EXITED'}
        if r!=258:raise RuntimeError('root wait unknown; retain')
        self.abort_attempted=True
        self.events.append({'action':'AbortCreatedSuspended','creation_receipt':receipt,'reason':reason,'state':'ATTEMPT_RESERVED'})
        check(K.TerminateProcess(self._root_handle,72))
        return {'state':'REQUESTED_NOT_EXIT_PROOF'}
    @serialized
    def close_failed_terminal(self):
        """Close a failed pre-Resume attempt only after original-root terminal proof."""
        if self.closed or getattr(self,'terminal_close_attempted',False):raise RuntimeError('closed/close already attempted; retain')
        if self.resume_attempted:raise RuntimeError('post-Resume use ordinary family terminal handling')
        if self.create_attempted and self._root_handle is None and not getattr(self,'create_failed',False):raise RuntimeError('creation outcome unknown without handle; retain')
        if self._root_handle is not None and K.WaitForSingleObject(self._root_handle,0)!=0:raise RuntimeError('root not proven exited; retain')
        if self._job is not None and self._member_pids():raise RuntimeError('job nonempty; retain')
        if any(getattr(self,'inherited_handles',[])):raise RuntimeError('stdio close outcome unknown; retain')
        self.terminal_close_attempted=True;self.closed_handle_records=[]
        for attr in ('_root_handle','_thread','_job'):
            h=getattr(self,attr)
            if h is not None:
                self.last_operation={'api':'CloseHandle','kind':attr,'handle':int(h)}
                check(K.CloseHandle(h));setattr(self,attr,None)
                for member in self.members.values():
                    if member._h==h:member._h=None
                self.closed_handle_records.append({'kind':attr})
        self.closed=True;self.state='CLOSED_FAILED_TERMINAL';ACTIVE.pop(id(self),None)
    @serialized
    def finish_failed_start(self,*,authorization,timeout_ms=5000):
        if authorization is not True or self.resume_attempted:raise ValueError('pre-Resume cleanup only')
        if not 1<=timeout_ms<=10000:raise ValueError('bounded timeout')
        if self.closed:return {'owner_terminal':True,'normal_guest_exit':False}
        if getattr(self,'terminal_close_attempted',False):raise RuntimeError('partial close retained; no replay')
        if self._root_handle is not None:
            if not self.abort_attempted:
                self.abort_created_suspended(self.creation_receipt,authorization=True,reason='failed reviewed launch; original suspended root only')
            self.last_operation={'api':'WaitForSingleObject','target':'original CreateProcess root','timeout_ms':timeout_ms}
            if K.WaitForSingleObject(self._root_handle,timeout_ms)!=0:raise RuntimeError('suspended-root termination not proven; retain')
        self.close_failed_terminal()
        return {'owner_terminal':self.closed,'normal_guest_exit':False,'pre_resume_cleanup':True,'root_termination_requested':self.abort_attempted,'closed_handle_records':self.closed_handle_records}
    def _require_member(self,h):
        b=W.BOOL();check(K.IsProcessInJob(h,self._job,C.byref(b)))
        if not b.value:raise RuntimeError('not in owned job')
    def _member_pids(self):
        class Pids(C.Structure):
            _fields_=[('assigned',W.DWORD),('count',W.DWORD),('ids',C.c_size_t*MAX_MEMBERS)]
        self.last_operation={'api':'QueryInformationJobObject','target':'retained dedicated Job','information_class':3}
        x=Pids();ret=W.DWORD();check(K.QueryInformationJobObject(self._job,3,C.byref(x),C.sizeof(x),C.byref(ret)))
        if x.count>MAX_MEMBERS or x.assigned>x.count:raise RuntimeError('job enumeration capacity/race; retain')
        return list(x.ids[:x.count])
    @serialized
    def observe_members(self):
        if self.closed or getattr(self,'terminal_close_attempted',False):raise RuntimeError('closed/closing owner; no rediscovery')
        rows=[]
        try:
            retained={}
            for key,member in self.members.items():
                pid=member.initial['pid']
                if key!=(pid,member.initial['creation_filetime']) or member._family is not self or member._h is None or pid in retained:
                    raise RuntimeError('retained identity/handle invariant failed; no PID recovery')
                retained[pid]=member
            for pid in self._member_pids():
                if pid in retained:
                    # A continuously held process HANDLE keeps that process
                    # object/ID alive. No reopen/image query during shutdown.
                    # All close paths serialize on this same family lock.
                    member=retained[pid]
                    member.wait(0) # Failure is UNKNOWN, never inferred as exit.
                    rows.append(member)
                    continue
                # PIDs only originate from the private Job. Open+membership validation
                # handles exit/reuse races; no caller-supplied PID entry point exists.
                self.last_operation={'api':'OpenProcess','pid_from_owned_job':int(pid),'requested_mask':'0x101410'}
                h=K.OpenProcess(0x1000|0x0400|0x0010|0x100000,False,pid)
                check(h)
                try:
                    def mark(api):self.last_operation={'api':api,'pid_from_owned_job':int(pid),'requested_mask':'0x101410'}
                    mark('IsProcessInJob');self._require_member(h);i=identity(h,mark)
                    if i['pid']!=pid or dict(self.command.allowed_images).get(i['exe_path'])!=i['exe_sha256']:
                        self.events.append({'observation':'unreviewed job image','identity':i})
                        raise RuntimeError('unreviewed job image; retain: '+repr(i))
                    mark('IsProcessInJob');self._require_member(h)
                    key=(pid,i['creation_filetime'])
                    if key not in self.members:
                        self.members[key]=OwnedMember(self,h,i);h=None
                    rows.append(self.members[key])
                except BaseException as exc:
                    import traceback
                    self.first_observation_error={'exception':repr(exc),'traceback':traceback.format_exc(),'last_operation':dict(self.last_operation)}
                    raise
                finally:
                    if h is not None:
                        self.last_operation={'api':'CloseHandle','pid_from_owned_job':int(pid),'kind':'temporary observation handle','requested_mask':'0x101410'}
                        check(K.CloseHandle(h))
            return rows
        except BaseException as exc:
            if not hasattr(self,'first_observation_error'):
                import traceback
                self.first_observation_error={'exception':repr(exc),'traceback':traceback.format_exc(),'last_operation':dict(getattr(self,'last_operation',{}))}
            self.state='OBSERVATION_UNKNOWN_RETAINED';raise
    @serialized
    def wait(self,ms=0):
        if type(ms)is not int or not 0<=ms<=10000:raise ValueError('bounded wait')
        r=K.WaitForSingleObject(self._job,ms)
        if r not in (0,258):raise C.WinError(C.get_last_error())
        pids=self._member_pids()
        return {'state':'EMPTY' if not pids else 'RUNNING','pids':pids,'root':self.root.wait(0)}
    def active_count(self):
        class Accounting(C.Structure):
            _fields_=[('user',C.c_longlong),('kernel',C.c_longlong),('period_user',C.c_longlong),('period_kernel',C.c_longlong),('page_faults',W.DWORD),('total',W.DWORD),('active',W.DWORD),('terminated',W.DWORD)]
        a=Accounting();ret=W.DWORD()
        check(K.QueryInformationJobObject(self._job,1,C.byref(a),C.sizeof(a),C.byref(ret)))
        return int(a.active)
    @serialized
    def close_terminal(self):
        if getattr(self,'terminal_close_attempted',False):raise RuntimeError('terminal close already attempted; do not replay')
        if any(getattr(self,'inherited_handles',[])):raise RuntimeError('stdio close outcome unknown; retain')
        if self.wait()['state']!='EMPTY':raise RuntimeError('live job retained')
        if any(m.wait()['state']!='EXITED' for m in self.members.values()):raise RuntimeError('live retained member')
        self.terminal_close_attempted=True
        if not hasattr(self,'closed_handle_records'):self.closed_handle_records=[]
        for m in self.members.values():
            self.last_operation={'api':'CloseHandle','identity':dict(m.initial)}
            check(K.CloseHandle(m._h))
            if m._h==self._root_handle:self._root_handle=None
            m._h=None;self.closed_handle_records.append({'kind':'process','identity':dict(m.initial)})
        self.last_operation={'api':'CloseHandle','kind':'thread'}
        check(K.CloseHandle(self._thread));self._thread=None;self.closed_handle_records.append({'kind':'thread'})
        self.last_operation={'api':'CloseHandle','kind':'job'}
        check(K.CloseHandle(self._job));self._job=None;self.closed_handle_records.append({'kind':'job'})
        self.closed=True;self.state='CLOSED_TERMINAL';ACTIVE.pop(id(self),None)

def finish_test_job(family, *, authorization, timeout_ms=5000, receipt=None):
    """One termination request, then both Job-empty AND all retained HANDLEs signaled.

    Unknown outcomes retain original handles in ACTIVE. Receipt survives all exceptions.
    This helper never claims normal Guest exit. No reopening or new PID access here.
    """
    import time,traceback
    if authorization is not True:raise ValueError('scoped termination authorization required')
    if type(timeout_ms) is not int or not 1<=timeout_ms<=10000:raise ValueError('bounded cleanup deadline required')
    if family.closed or family._job is None:raise RuntimeError('live original Job handle required')
    if not family.resume_attempted or not family.assigned or not hasattr(family,'root') or family.root._h!=family._root_handle:
        raise RuntimeError('ordinary Job cleanup requires assigned resumed original root; use failed-start cleanup')
    r=receipt if receipt is not None else {}
    r.update(status='CLEANUP_IN_PROGRESS',normal_guest_exit=False,members=[],errors=[],job_empty_seen=False)
    end=time.monotonic()+timeout_ms/1000
    def fault(api,exc,target=None):
        r['errors'].append({'api':api,'target':target,'exception':repr(exc),'traceback':traceback.format_exc()})
    try:
        if getattr(family,'job_terminate_attempted',False):
            r['status']='CLEANUP_UNKNOWN_PRIOR_REQUEST';return r
        family.job_terminate_attempted=True
        r['terminate_request']='ATTEMPT_RESERVED'
        try:
            check(K.TerminateJobObject(family._job,73));r['terminate_request']='RETURNED_SUCCESS_NOT_EXIT_PROOF'
        except BaseException as exc:
            fault('TerminateJobObject',exc,'original dedicated Job');r['status']='CLEANUP_UNKNOWN_TERMINATE';return r
        while True:
            r['members']=[]
            try:
                pids=family._member_pids();r['job_member_pids_last']=pids
                active=family.active_count();r['job_active_count_last']=active
                r['job_empty_seen']|=not pids
            except BaseException as exc:
                fault('QueryInformationJobObject',exc,'original dedicated Job');r['status']='CLEANUP_UNKNOWN_QUERY';return r
            all_exited=True
            for member in family.members.values():
                row={'identity':dict(member.initial),'retained_handle':True}
                try:row['observation']=member.wait(0)
                except BaseException as exc:
                    row['observation']={'state':'UNKNOWN'};fault('WaitForSingleObject/GetExitCodeProcess',exc,member.initial)
                r['members'].append(row)
                all_exited &= row['observation']['state']=='EXITED'
            if r['errors']:
                r['status']='CLEANUP_UNKNOWN_MEMBER';return r
            if not pids and active==0 and all_exited:
                r['termination_confirmed']=True
                try:family.close_terminal()
                except BaseException as exc:
                    fault('close_terminal',exc,'original retained handles');r['status']='TERMINATED_CLOSE_FAILED';return r
                r['status']='FORCED_CLEANUP_CONFIRMED';return r
            if time.monotonic()>=end:
                r['status']='CLEANUP_UNKNOWN_DEADLINE';return r
            time.sleep(min(.01,max(0,end-time.monotonic())))
    finally:
        r['handles_closed']=family.closed
        r['deadline_seconds']=timeout_ms/1000
        r['closed_handle_records']=getattr(family,'closed_handle_records',[])
        r['last_operation']=getattr(family,'last_operation',None)


def recorded_observe(family, evidence):
    """Persist operation, requested rights and initial error; do not hide access denial."""
    import traceback
    evidence['phase']='observe_members'
    evidence['requested_openprocess_mask']='0x101410'
    try:return family.observe_members()
    except BaseException as exc:
        evidence['initial_error']=getattr(family,'first_observation_error',{'exception':repr(exc),'traceback':traceback.format_exc(),'last_operation':getattr(family,'last_operation',None)})
        evidence['last_known_identities']=[dict(m.initial) for m in family.members.values()]
        raise
