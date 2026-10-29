"""Opt-in, one instance's retained Linux pidfds. No PID-signal fallback."""
import os,sys,signal,select,time,copy

class NativeOps:
    def __init__(self):
        if sys.platform!='linux' or not hasattr(os,'pidfd_open') or not hasattr(signal,'pidfd_send_signal'):
            raise RuntimeError('Linux pidfd API unavailable; no fallback')
    def open(self,pid):return os.pidfd_open(pid,0)
    def exited(self,fd):
        p=select.poll();p.register(fd,select.POLLIN)
        rows=p.poll(0)
        if any(flags & (select.POLLERR|select.POLLNVAL) for _,flags in rows):raise OSError('pidfd poll error')
        return any(flags & (select.POLLIN|select.POLLHUP) for _,flags in rows)
    def term(self,fd):signal.pidfd_send_signal(fd,signal.SIGTERM,None,0)
    def close(self,fd):os.close(fd)

from bound_owner import BoundInstance as SharedBoundInstance

class BoundInstance(SharedBoundInstance):
    def __init__(self,expected,inventory,controller_pid,ops=None,clock=__import__("time").monotonic,sleep=__import__("time").sleep,**kwargs):
        super().__init__(expected,inventory,controller_pid,ops=ops or NativeOps(),handle_field="retained_pidfds",clock=clock,sleep=sleep,**kwargs)
