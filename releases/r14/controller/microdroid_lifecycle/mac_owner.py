"""Mac normal and fault lifecycle using generation-bound original handles."""
import os, pathlib, subprocess, time, json
from native import PosixOwner, require
from mac_audit import AuditOps
from audit_owner import BoundInstance

class MacOwner(PosixOwner):
    def start(self):
        require(self.p['platform']=='macos','Mac only')
        self.audit_query=AuditOps()
        self.normal_ops=AuditOps()
        self.normal_handles={}
        super().start()

    def ps_rows(self):
        cp=subprocess.run(['ps','-axo','pid=,ppid=,stat=,lstart=,command='],
                          capture_output=True,text=True,timeout=5)
        require(cp.returncode==0 and not cp.stderr,'session inventory failed')
        out={}
        for line in cp.stdout.splitlines():
            f=line.split(None,8)
            require(len(f)==9,'malformed process inventory')
            pid=int(f[0])
            try:sid=os.getsid(pid)
            except ProcessLookupError:continue # no exit proof inferred
            if sid!=self.child.pid:continue
            if f[2].startswith('Z'):continue # original kqueue still required
            out[pid]={'ppid':int(f[1]),'sid':sid,'start':' '.join(f[3:8]),'cmd':f[8]}
        return out

    def table(self,all_members=False):
        if not hasattr(self,'child'):
            # Keep the frozen global CID conflict preflight before Popen. Only
            # the post-launch view is restricted to our dedicated session.
            return PosixOwner.table(self)
        out={}
        for pid,row in self.ps_rows().items():
            if not all_members:
                # Discovery only: transient shell/client processes are not
                # owners. The complete session is checked separately at arm.
                service=str(self.run/'service'/'virtmgr-service.sock') in row['cmd']
                guest='--cid' in row['cmd'] and '/crosvm ' in row['cmd']
                if not service and not guest:continue
            owner=getattr(self,'fault_owner',None)
            handle=owner.fds.get(pid) if owner else None
            if handle and owner.ops.exited(handle) and pid not in self.ps_rows():continue
            normal=getattr(self,'normal_handles',{}).get(pid)
            if normal and self.normal_ops.exited(normal):
                original=self.owned[pid]
                require(all(row[k]==original[k] for k in ('start','sid','cmd')),
                        'exited PID reused or session identity replaced')
                continue
            # Do not erase permission, I/O or uncertain port-release errors,
            # even if a subsequent observation happens to find the PID absent.
            identity=self.audit_query.identity(pid)
            # Re-read association while generation is fixed by the token checks.
            after=self.ps_rows().get(pid)
            require(after is not None and all(after[k]==row[k] for k in ('start','sid','cmd')),
                    'session identity changed during token query')
            identity2=self.audit_query.identity(pid)
            require(identity==identity2,'audit generation changed during inventory')
            out[pid]={**after,**identity}
        return out

    def same_identity(self,a,b):
        return super().same_identity(a,b) and a['audit_token']==b['audit_token']

    def live(self,pid,row):
        handle=getattr(self,'normal_handles',{}).get(pid)
        if handle and self.normal_ops.exited(handle):return False
        now=self.table()
        if pid not in now:return False # never enough for fault resource completion
        require(self.same_identity(row,now[pid]),'audit generation changed')
        return True

    def observe(self):
        require(not getattr(self,'normal_binding_blocked',False),'partial normal binding retained; no rebind')
        super().observe()
        if self.p.get('request_fault_test') is not None:return
        for pid,row in self.owned.items():
            if pid in self.normal_handles:continue
            self.normal_ops.expected[pid]=row
            try:handle=self.normal_ops.open(pid)
            except BaseException:
                self.normal_binding_blocked=True
                raise
            self.normal_handles[pid]=handle
            try:
                current=self.table().get(pid)
                require(current is not None and self.same_identity(row,current),
                        'normal ownership changed while acquiring original handle')
            except BaseException:
                self.normal_binding_blocked=True
                raise

    def guest_done(self):
        if self.p.get('request_fault_test') is not None:return super().guest_done()
        self.observe()
        guests=[pid for pid,row in self.owned.items() if row['kind']=='crosvm']
        return bool(guests) and all(pid in self.normal_handles and
                    self.normal_ops.exited(self.normal_handles[pid]) for pid in guests)

    def bind_linux_fault_members(self):
        # Name retained for the existing request hook; semantics are Mac only.
        require(not hasattr(self,'fault_owner'),'binding cannot replay')
        self.observe()
        require(self.child.poll()==0,'launcher/client not naturally complete')
        expected={pid:{k:row[k] for k in ('start','sid','exe','cmd','audit_token')}
                  for pid,row in self.owned.items()}
        for pid in expected:expected[pid]['role']='guest' if self.owned[pid]['kind']=='crosvm' else self.owned[pid]['kind']
        require(self.mac_inventory()==expected,'persistent members differ')
        self.fault_owner=BoundInstance(expected,self.mac_inventory,os.getpid(),ops=AuditOps(expected))
        self.fault_owner.bind()

    def mac_inventory(self):
        out={}
        for pid,row in self.table(all_members=True).items():
            out[pid]={k:row[k] for k in ('start','sid','exe','cmd','audit_token')}
            kind=self.owned.get(pid,{}).get('kind','UNOWNED')
            out[pid]['role']='guest' if kind=='crosvm' else kind
        return out

    def session_empty(self):
        return (self.child.poll() is not None and not self.ps_rows()
                and not any(not h.get('port_closed') for h in self.audit_query.retained))

    def terminate_linux_fault_once(self,*,authorized):
        require(hasattr(self,'fault_owner'),'original audit ownership missing')
        self.forced_macos_cleanup_requested=True
        def logs():
            if not self.session_empty():return False
            if not self.log.closed:self.log.close()
            return self.log.closed
        self.cleanup=self.fault_owner.terminate_once(authorized=authorized,
                          logs_released=logs,storage_snapshot=self.snapshot)
        self.cleanup['audit_handles']=self.fault_owner.ops.evidence()
        self.owner_terminal=self.cleanup.get('resource_state')=='REAPED'
        return self.cleanup

    def finalize_if_terminal(self):
        if any(getattr(self,k,False) for k in ('normal_binding_blocked','normal_close_blocked','normal_receipt_blocked')):return False
        if self.owner_terminal:return True
        if not hasattr(self,'fault_owner'):
            if self.p.get('request_fault_test') is None and self.guest_done():
                self.cleanup_services();return self.owner_terminal
            if self.session_empty():
                if any(not h.get('port_closed') or not h.get('kq_closed')
                       for h in getattr(getattr(self,'normal_ops',None),'retained',[])):return False
                self.log.close();self.owner_terminal=True;return True
            return False # no automatic bare-PID idle-service path
        result=super().finalize_if_terminal()
        if hasattr(self,'cleanup'):self.cleanup['audit_handles']=self.fault_owner.ops.evidence()
        return result

    def cleanup_services(self):
        require(not hasattr(self,'fault_owner'),'fault owner prohibits normal service cleanup')
        require(not getattr(self,'normal_binding_blocked',False),'partial normal binding retained')
        require(not getattr(self,'normal_close_blocked',False),'partial close retained; no replay')
        require(not getattr(self,'normal_receipt_blocked',False),'cleanup receipt outcome unknown')
        require(self.guest_done(),'original Guest exit not proven')
        require(self.child.poll()==0,'launcher not naturally complete')
        if not hasattr(self,'normal_cleanup_deadline'):
            self.normal_cleanup_deadline=time.monotonic()+5
        for pid,row in self.owned.items():
            h=self.normal_handles[pid]
            if row['kind']!='virtmgr' or self.normal_ops.exited(h):continue
            if not h['signal_attempted']:
                current=self.table().get(pid)
                require(current is not None and self.same_identity(row,current),
                        'service identity changed before normal cleanup')
                self.normal_ops.term(h)
        while not all(self.normal_ops.exited(h) for h in self.normal_handles.values()):
            require(time.monotonic()<self.normal_cleanup_deadline,'normal service observation deadline')
            time.sleep(.01)
        require(self.session_empty(),'unknown session member remains')
        for h in self.normal_handles.values():
            if not h.get('port_closed'):
                try:self.normal_ops.close(h)
                except BaseException:
                    self.normal_close_blocked=True
                    raise
        self.log.close()
        self.cleanup={'services':self.normal_ops.evidence(),'session_empty':True,
                      'regular_log_closed':True,'inner_pipe_eof':'NOT_APPLICABLE_REGULAR_FILE'}
        try:(self.run/'cleanup.json').write_text(json.dumps(self.cleanup,indent=2))
        except BaseException:
            self.normal_receipt_blocked=True
            raise
        self.owner_terminal=True

    def failure_cleanup(self):
        result=super().failure_cleanup()
        if hasattr(self,'fault_owner'):
            result['audit_handles']=self.fault_owner.ops.evidence()
        if hasattr(self,'audit_query'):
            result['partial_query_handles']=self.audit_query.evidence()
        return result
