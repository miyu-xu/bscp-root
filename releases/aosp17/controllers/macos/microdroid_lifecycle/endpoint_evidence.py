"""Read-only proof of the original Host-owned Guest RPC endpoint retirement."""
import pathlib,os,stat
from strict_json import loads

class EndpointEvidence:
    def __init__(self,trace,cid,service_pid,identity_check,lstat=None):
        self.trace=trace;self.cid=cid;self.service_pid=service_pid
        self.identity_check=identity_check;self.path=pathlib.Path('/tmp/binder_rpc_vsock_%d_%d.sock'%(cid,cid))
        self.lstat=lstat or self.path.lstat;self.original=None
    def events(self):
        rows=[]
        text=self.trace()
        self.pending_tail=bool(text) and not text.endswith('\n')
        # A newline is the Host's event commit boundary. A concurrent reader
        # can see a prefix of an otherwise valid append; it is pending, not
        # successful retirement and not a permanent malformed-event verdict.
        committed=text[:text.rfind('\n')+1]
        for line in committed.splitlines():
            if not line.startswith('guestRpcEndpoint '):continue
            row=loads(line[len('guestRpcEndpoint '):])
            if not isinstance(row,dict) or set(row)!={'phase','cid','pid','dev','ino','uid'}:raise RuntimeError('endpoint event shape')
            if row['phase'] not in ('bound','retired') or any(type(row[k]) is not int or row[k]<0 for k in ('cid','pid','dev','ino','uid')):raise RuntimeError('endpoint event type')
            if row['cid']!=self.cid or row['pid']!=self.service_pid:raise RuntimeError('endpoint event creator mismatch')
            rows.append(row)
        return rows
    def bind(self):
        if self.original is not None:raise RuntimeError('endpoint binding replay')
        self.identity_check()
        rows=self.events()
        if self.pending_tail or len(rows)!=1 or rows[0]['phase']!='bound':raise RuntimeError('unique complete live endpoint creation evidence required')
        row=rows[0];m=self.lstat()
        if not stat.S_ISSOCK(m.st_mode) or stat.S_IMODE(m.st_mode)!=0o600 or (m.st_dev,m.st_ino,m.st_uid)!=(row['dev'],row['ino'],row['uid']) or row['uid']!=os.getuid():raise RuntimeError('endpoint identity differs from creator evidence')
        self.identity_check();self.original=dict(row)
    def retired(self):
        if self.original is None:raise RuntimeError('endpoint was not bound')
        rows=self.events();expected=dict(self.original,phase='retired')
        if not rows or rows[0]!=self.original or len(rows)>2 or (len(rows)==2 and rows[1]!=expected):raise RuntimeError('endpoint evidence replaced/duplicated')
        try:m=self.lstat()
        except FileNotFoundError:
            return len(rows)==2 and not self.pending_tail
        if not stat.S_ISSOCK(m.st_mode) or (m.st_dev,m.st_ino,m.st_uid)!=(self.original['dev'],self.original['ino'],self.original['uid']):raise RuntimeError('endpoint path replaced; not owned')
        if len(rows)==2:raise RuntimeError('retired endpoint path exists')
        return False
