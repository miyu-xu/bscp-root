import copy
from bound_owner import BoundInstance as SharedBoundInstance

class BoundInstance(SharedBoundInstance):
    def __init__(self,expected,inventory,controller_pid,ops,*,retirement_ready,endpoint_identity,clock=__import__("time").monotonic,sleep=__import__("time").sleep,**kwargs):
        super().__init__(expected,inventory,controller_pid,ops=ops,handle_field="retained_audit_handles",clock=clock,sleep=sleep,**kwargs)
        self.retirement_ready=retirement_ready
        self.receipt['guest_rpc_endpoint']={'creation':copy.deepcopy(endpoint_identity),'retired':False}

    def endpoint_retired(self):
        try:ready=self.retirement_ready()
        except BaseException as exc:
            self.receipt['identity_observation_blocked']=True
            self.receipt['errors'].append(repr(exc))
            raise
        if ready is not True:return False
        self.receipt['guest_rpc_endpoint']['retired']=True
        return True

    def observe_once(self,*,logs_released,storage_snapshot):
        def logs_and_endpoint():
            return self.endpoint_retired() and logs_released() is True
        return super().observe_once(logs_released=logs_and_endpoint,storage_snapshot=storage_snapshot)

    def terminate_once(self,*,authorized,logs_released,storage_snapshot):
        if authorized is not True or not self.armed:raise RuntimeError('explicit scoped authorization and binding required')
        if self.cleanup_attempted:raise RuntimeError('cleanup not replayable')
        self.cleanup_attempted=True;deadline=self.clock()+5
        self.receipt['observation_budget_seconds']=5
        guest=next(p for p,r in self.expected.items() if r['role']=='guest')
        service=next(p for p,r in self.expected.items() if r['role']=='virtmgr')
        def signal_once(pid):
            if self.clock()>=deadline or not self.validate():return False
            if self.ops.exited(self.fds[pid]):return True
            self.attempts.add(pid)
            row={'pid':pid,'identity':self.expected[pid],'signal':'TERM','attempt':'RESERVED'}
            self.receipt['signals'].append(row)
            self.ops.term(self.fds[pid]);row['attempt']='RETURNED_NOT_EXIT_PROOF'
            return True
        try:
            if signal_once(guest):
                # The live creator must finish its own authenticated endpoint
                # destructor before it may be terminated. Never unlink here.
                while self.clock()<deadline:
                    ready=self.validate()
                    if ready and self.ops.exited(self.fds[guest]) and self.endpoint_retired():
                        signal_once(service);break
                    self.sleep(min(.01,max(0,deadline-self.clock())))
            while self.clock()<deadline:
                result=self.observe_once(logs_released=logs_released,storage_snapshot=storage_snapshot)
                if result['resource_state']=='REAPED' or result.get('observation_error'):return result
                self.sleep(min(.01,max(0,deadline-self.clock())))
            self.receipt['observation_expired']=True
        except BaseException as exc:
            self.receipt['errors'].append(repr(exc));self.receipt['identity_observation_blocked']=True
        self.receipt[self.handle_field]=sorted(self.fds)
        return copy.deepcopy(self.receipt)
