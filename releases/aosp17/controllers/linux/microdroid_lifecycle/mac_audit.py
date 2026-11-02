"""macOS generation-bound signal and original NOTE_EXIT ownership. No PID kill."""
import ctypes, os, select, sys

class AuditOps:
    def __init__(self, expected=None):
        if sys.platform != 'darwin': raise RuntimeError('Darwin required; no fallback')
        self.expected = expected or {}
        self.u = ctypes.c_uint32
        self.lib = ctypes.CDLL('/usr/lib/libSystem.B.dylib', use_errno=True)
        self.proc = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
        u=self.u
        self.own=u.in_dll(self.lib,'mach_task_self_').value
        self.lib.task_name_for_pid.argtypes=[u,ctypes.c_int,ctypes.POINTER(u)]
        self.lib.task_info.argtypes=[u,ctypes.c_int,ctypes.POINTER(u),ctypes.POINTER(u)]
        self.lib.mach_port_deallocate.argtypes=[u,u]
        self.proc.proc_signal_with_audittoken.argtypes=[ctypes.POINTER(u),ctypes.c_int]
        self.proc.proc_signal_with_audittoken.restype=ctypes.c_int
        self.proc.proc_pidpath_audittoken.argtypes=[ctypes.POINTER(u),ctypes.c_void_p,u]
        self.proc.proc_pidpath_audittoken.restype=ctypes.c_int
        self.retained=[]

    def check(self, rc, operation):
        if rc: raise RuntimeError('Mach result '+str(rc)+': '+operation)

    def token(self, h):
        t=(self.u*8)();n=self.u(8)
        self.check(self.lib.task_info(h['port'],15,t,ctypes.byref(n)),'task_info')
        if n.value!=8 or t[5]!=h['pid']:raise RuntimeError('token count/PID mismatch')
        return list(t)

    def path(self, token):
        t=(self.u*8)(*token);b=ctypes.create_string_buffer(4096)
        if self.proc.proc_pidpath_audittoken(t,b,4096)<=0:
            raise OSError(ctypes.get_errno(),'token executable query')
        return os.path.realpath(os.fsdecode(b.value))

    def acquire(self,pid):
        port=self.u()
        self.check(self.lib.task_name_for_pid(self.own,pid,ctypes.byref(port)),'task_name_for_pid')
        h={'pid':pid,'port':port.value,'kq':None,'exited':False,'signal_attempted':False,
           'port_close_attempted':False,'kq_close_attempted':False,'events':[]}
        self.retained.append(h) # even partial initialization stays owned
        try:h['token']=self.token(h);h['exe']=self.path(h['token'])
        except BaseException:
            self.release_port(h)
            self.retained.remove(h)
            raise
        return h

    def identity(self,pid):
        h=self.acquire(pid)
        try:
            if self.token(h)!=h['token']:raise RuntimeError('token changed during identity query')
            return {'audit_token':h['token'],'exe':h['exe']}
        finally:
            self.release_port(h)
            self.retained.remove(h)

    def open(self,pid):
        h=self.acquire(pid)
        expected=self.expected[pid]
        if h['token']!=expected['audit_token'] or h['exe']!=expected['exe']:
            raise RuntimeError('original generation/executable differs before registration')
        h['kq']=select.kqueue()
        h['kq'].control([select.kevent(pid,filter=select.KQ_FILTER_PROC,
            flags=select.KQ_EV_ADD|select.KQ_EV_ONESHOT,fflags=select.KQ_NOTE_EXIT)],0,0)
        if self.token(h)!=h['token'] or self.path(h['token'])!=h['exe']:
            raise RuntimeError('generation/executable differs after registration')
        h['registration_verified']=True
        return h

    def exited(self,h):
        if h['exited']:return True
        if h['kq'] is None:raise RuntimeError('no original exit registration')
        for e in h['kq'].control(None,1,0):
            row={'ident':e.ident,'filter':e.filter,'flags':e.flags,'fflags':e.fflags}
            h['events'].append(row)
            if e.ident!=h['pid'] or e.filter!=select.KQ_FILTER_PROC or e.flags&select.KQ_EV_ERROR or not e.fflags&select.KQ_NOTE_EXIT:
                raise RuntimeError('invalid original exit event')
            h['exited']=True
        return h['exited']

    def term(self,h):
        if h['signal_attempted']:raise RuntimeError('signal replay prohibited')
        if not h.get('registration_verified') or self.exited(h):raise RuntimeError('not verified live')
        if self.token(h)!=h['token'] or self.path(h['token'])!=h['exe']:
            raise RuntimeError('identity changed before token signal')
        h['signal_attempted']=True
        rc=self.proc.proc_signal_with_audittoken((self.u*8)(*h['token']),15)
        h['signal_return']=rc
        if rc:raise OSError(rc,'proc_signal_with_audittoken TERM')

    def release_port(self,h):
        if h.get('port_closed'):return
        if h['port_close_attempted']:raise RuntimeError('port close outcome unknown')
        h['port_close_attempted']=True
        self.check(self.lib.mach_port_deallocate(self.own,h['port']),'mach_port_deallocate')
        h['port_closed']=True

    def close(self,h):
        if not self.exited(h):raise RuntimeError('exit not proven; keep name port/kqueue')
        if h['kq_close_attempted']:raise RuntimeError('kqueue close outcome unknown')
        h['kq_close_attempted']=True;h['kq'].close();h['kq_closed']=True
        self.release_port(h)

    def evidence(self):
        return [{k:v for k,v in h.items() if k!='kq'} for h in self.retained]
