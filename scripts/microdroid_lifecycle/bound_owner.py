import copy,time

class BoundInstance:
    """inventory returns every live member of the original dedicated session.

    Expected identities come from the caller's verified launch association, not
    arbitrary user PID input. Identity includes generation, session, executable,
    argv and role. A audit/kqueue pins a process, not a complete process family.
    """
    def __init__(self,expected,inventory,controller_pid,ops,handle_field="retained_handles",clock=time.monotonic,sleep=time.sleep):
        self.expected=copy.deepcopy(expected);self.inventory=inventory
        self.ops=ops;self.handle_field=handle_field;self.clock=clock;self.sleep=sleep
        self.fds={};self.attempts=set();self.close_attempts=set();self.armed=False;self.cleanup_attempted=False
        self.receipt={'task_result':'UNKNOWN','resource_state':'UNKNOWN','needsRecovery':True,
                      'storage_state':'UNVERIFIED','storage_released':None,'signals':[],'errors':[],
                      'members':{str(pid):copy.deepcopy(row) for pid,row in expected.items()}}
        if not expected or controller_pid in expected:raise ValueError('invalid member set')
        roles=[v.get('role') for v in expected.values()]
        if sorted(roles)!=['guest','virtmgr']:
            raise ValueError('this candidate requires exactly the persistent service and GPU-off Guest')
        if len(set(v['sid'] for v in expected.values()))!=1:raise ValueError('mixed sessions')
    def bind(self):
        if self.fds or self.armed:raise RuntimeError('binding not replayable')
        try:
            if self.inventory()!=self.expected:raise RuntimeError('initial membership/identity mismatch')
            for pid in sorted(self.expected):
                self.fds[pid]=self.ops.open(pid)
                if self.ops.exited(self.fds[pid]):raise RuntimeError('member exited during binding')
                if self.inventory()!=self.expected:raise RuntimeError('identity changed during binding')
            self.armed=True
        except BaseException as exc:
            self.receipt['errors'].append(repr(exc))
            # Partial handles remain owned, no speculative close/rebind or signal.
            raise
    def validate(self):
        now=self.inventory()
        pending=False
        if set(now)-set(self.expected):raise RuntimeError('new unowned session member')
        for pid,identity in self.expected.items():
            exited=self.ops.exited(self.fds[pid])
            if pid in now and now[pid]!=identity:raise RuntimeError('generation/identity changed')
            if (pid not in now and not exited) or (pid in now and exited):pending=True
        return not pending
    def terminate_once(self,*,authorized,logs_released,storage_snapshot):
        if authorized is not True or not self.armed:raise RuntimeError('explicit scoped authorization and binding required')
        if self.cleanup_attempted:raise RuntimeError('cleanup not replayable')
        self.cleanup_attempted=True
        deadline=self.clock()+5
        self.receipt['observation_budget_seconds']=5
        try:
            ready=self.validate() # all members pass before the first signal
            for pid in sorted(self.fds) if ready else ():
                if self.clock()>=deadline:break
                if not self.validate():break # no later signal retry on a pending snapshot
                fd=self.fds[pid]
                if self.ops.exited(fd):continue
                self.attempts.add(pid)
                row={'pid':pid,'identity':self.expected[pid],'signal':'TERM','attempt':'RESERVED'}
                self.receipt['signals'].append(row)
                self.ops.term(fd);row['attempt']='RETURNED_NOT_EXIT_PROOF'
            while self.clock()<deadline:
                result=self.observe_once(logs_released=logs_released,storage_snapshot=storage_snapshot)
                if result['resource_state']=='REAPED' or result.get('observation_error'):return result
                self.sleep(min(.01,max(0,deadline-self.clock())))
            self.receipt['observation_expired']=True
        except BaseException as exc:
            self.receipt['errors'].append(repr(exc))
            self.receipt['identity_observation_blocked']=True
        self.receipt[self.handle_field]=sorted(self.fds)
        return copy.deepcopy(self.receipt)

    def observe_once(self,*,logs_released,storage_snapshot):
        """One signal-free observation, never replays uncertain closes.
        Calls may block in external OS/query callbacks; no wall-time guarantee.
        """
        if not self.armed or not self.cleanup_attempted:raise RuntimeError('no completed binding/cleanup attempt')
        if self.receipt['resource_state']=='REAPED':return copy.deepcopy(self.receipt)
        if self.receipt.get('identity_observation_blocked'):return copy.deepcopy(self.receipt)
        self.receipt.pop('observation_error',None)
        try:
            if self.close_attempts:raise RuntimeError('close outcome uncertain; no replay')
            try:
                now=self.inventory()
                if set(now)-set(self.expected):raise RuntimeError('new unowned session member')
                for pid,row in now.items():
                    if row!=self.expected[pid]:raise RuntimeError('generation/identity changed')
                exited=all(self.ops.exited(fd) for fd in self.fds.values())
            except BaseException as exc:
                self.receipt['identity_observation_blocked']=True
                self.receipt['errors'].append(repr(exc))
                raise
            # Inventory and readiness are not an atomic snapshot. Either side
            # pending means retain, not success and not an irreversible failure.
            if now or not exited:
                self.receipt['observation_pending']=True
            else:
                self.receipt['processes_exited']=True
                if logs_released() is not True:raise RuntimeError('reader/log release unproven')
                snapshot=storage_snapshot()
                if not isinstance(snapshot,dict) or not snapshot:raise RuntimeError('storage release unproven')
                for pid,fd in list(self.fds.items()):
                    self.close_attempts.add(pid);self.ops.close(fd);del self.fds[pid]
                self.receipt.update(storage_snapshot=snapshot,storage_state='RELEASED',storage_released=True,resource_state='REAPED',observation_pending=False)
        except BaseException as exc:self.receipt['observation_error']=repr(exc)
        self.receipt[self.handle_field]=sorted(self.fds)
        return copy.deepcopy(self.receipt)
