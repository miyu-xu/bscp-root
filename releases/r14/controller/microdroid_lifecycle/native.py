"""Thin native ownership adapters; imports Windows helper only on Windows."""
import hashlib,json,os,pathlib,re,signal,socket,subprocess,time,shutil,select,stat,errno

def sha(p):
    digest = hashlib.sha256()
    with pathlib.Path(p).open('rb') as f:
        while True:
            chunk = f.read(1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()
def require(ok,msg):
    if not ok:raise RuntimeError(msg)

class NativeOwner:
    def controller_release_ready(self):
        return self.owner_terminal
    def __init__(self,p,cfg,plan,run,state):
        self.p=p;self.cfg=cfg;self.plan=plan;self.run=pathlib.Path(run);self.state=pathlib.Path(state)
        self.owned={};self.sent_stop=False;self.transport_unknown=False;self.started=False;self.owner_terminal=False
    def retention_status(self):
        total=0;count=0;bounded=True
        for f in self.run.rglob('*'):
            if f.is_symlink():continue
            if f.is_file():total+=f.stat().st_size;count+=1
            if count>=4096:bounded=False;break
        return {'state':'UNKNOWN','owner_terminal':self.owner_terminal,'stop_sent':self.sent_stop,'transport_unknown':self.transport_unknown,'owned_processes':len(self.owned),'run_bytes_observed':total,'inventory_complete':bounded,'free_bytes':shutil.disk_usage(self.run).free,'growth_warning':total>128*1024**2 or not bounded,'disk_floor_warning':shutil.disk_usage(self.run).free<10*1024**3,'external_logs_hard_capped':False,'safe_stop_available':not self.sent_stop and not self.transport_unknown,'no_auto_kill_or_recovery':True}
    def request(self,command):
        require(not self.transport_unknown,'prior request outcome unknown')
        if command.startswith('VADD '):
            from compute_protocol import parse
            from entry import gpu_mode
            parse(command)
            if gpu_mode(self.cfg)=='off':return 'VADD ERR GPU_DISABLED\n'
        self.observe();require(self.guest_present(),'no verified Guest ownership')
        # prelaunch exclusive bind + scoped service/Guest ownership are required;
        # this is cooperative same-user isolation, not a hostile-local-peer defense.
        with socket.create_connection(('127.0.0.1',self.cfg['port']),timeout=3) as s:
            fault=self.p.get('request_fault_test')
            inject=fault is not None and command=='ADD 1'
            if inject:
                require(self.p['platform'] in ('windows','linux','macos') and fault.get('instance')==self.cfg['name'],'fault test instance mismatch')
                if self.p['platform'] in ('linux','macos'):self.bind_linux_fault_members()
                require(fault.get('mode') in ('timeout','disconnect') and fault.get('enabled') is True,'fault test disabled/invalid')
                require(not getattr(self,'fault_attempted',False),'fault request may not replay')
                self.fault_attempted=True
            s.settimeout(10);self.transport_unknown=True;s.sendall((command+'\n').encode());out=b''
            if inject:
                self.fault_receipt={'mode':fault['mode'],'instance':self.cfg['name'],'command':command,'send_returned':True,'response_delivered':False}
                if fault['mode']=='timeout':
                    until=time.monotonic()+10
                    while time.monotonic()<until:time.sleep(max(0,min(.05,until-time.monotonic())))
                    raise TimeoutError('test-only response withheld for10seconds; outcome unknown')
                s.shutdown(socket.SHUT_RDWR)
                raise ConnectionAbortedError('test-only own request socket disconnected; outcome unknown')
            until=time.monotonic()+10
            while not out.endswith(b'\n'):
                require(time.monotonic()<until,'response deadline')
                s.settimeout(max(.001,until-time.monotonic()))
                x=s.recv(4096);require(x,'response EOF');out+=x;require(len(out)<=34816,'response limit')
            require(out.count(b'\n')==1,'malformed response')
        response=out.decode() # Decode failures also retain the uncertain outcome.
        self.transport_unknown=False;return response
    def ready(self):
        self.observe()
        if not self.guest_present():return False
        trace=self.trace()
        if not re.search(r'notify(?:_payload_ready|PayloadReady) cid='+str(self.cfg['cid'])+r'\b',trace):return False
        require(self.request('INFO')=='HEADLESS_COUNTER_V1 GUEST_ENCRYPTED_STORAGE\n','unexpected payload contract')
        return True
    def stop(self):
        require(not self.sent_stop,'STOP must not be repeated');self.sent_stop=True
        return self.request('STOP')
    def trace(self):
        path=pathlib.Path(self.plan['environment']['VIRTMGR_TRACE_FILE'])
        require(not path.exists() or path.stat().st_size<=8*1024*1024,'trace limit exceeded')
        return path.read_text(errors='replace') if path.exists() else ''
    def guest_exit(self):
        require(self.guest_done(),'Guest still alive/unknown')
        trace=self.trace();cid=str(self.cfg['cid'])
        require(re.search(r'notifyPayloadFinished cid='+cid+r' exit_code=0(?:\s|$)',trace) and re.search(r'callback_on_died cid='+cid+r' reason=r#SHUTDOWN(?:\s|$)',trace),'missing exact CID exit/SHUTDOWN evidence')
        return 0
    def failure_cleanup(self):
        # Without fresh success proof do not send guessed signals. Keep claims frozen.
        if self.owner_terminal:return {'owner_terminal':True,'guest_success_not_implied':True}
        if self.started and self.guest_done():
            self.cleanup_services();return {'services_cleaned':True,'guest_success_not_implied':True,'owner_terminal':True}
        return {'state':'UNKNOWN','manual_live_owner_attention':True,'no_pid_reattach':True,'owner_terminal':not self.started}
    def finalize_if_terminal(self):
        if self.owner_terminal:return True
        if self.guest_done():self.cleanup_services()
        return self.owner_terminal

class WindowsOwner(NativeOwner):
    def terminal_resource_receipt(self):
        if not self.owner_terminal or not self.family.closed:return None
        log=getattr(self,'log_receipt',{})
        if (log.get('eof') is not True or log.get('finished') is not True or
                log.get('reader_alive') is not False or log.get('error') or
                log.get('dropped') or log.get('write_close_unknown')):return None
        snapshot=self.snapshot()
        require(isinstance(snapshot,dict) and bool(snapshot),'missing storage release evidence')
        return {'owner_terminal':True,'resource_state':'REAPED','storage_snapshot':snapshot,
                'forced_job_cleanup':getattr(self.family,'job_terminate_attempted',False) is True,
                'receipt':self.cleanup,'pipe_log':log,'no_repeat':True}
    def controller_release_ready(self):
        if not self.owner_terminal:return False
        if not hasattr(self,'log'):return True
        # Keep the controller until its reader terminates. A terminal read error
        # allows a failed result, never a fabricated EOF or REAPED receipt.
        self.log_receipt=self.log.finish(timeout=0)
        return (self.log_receipt.get('finished') is True
                and self.log_receipt.get('reader_alive') is False
                and self.log_receipt.get('write_close_unknown') is False)
    def finish_logs(self):
        try:self.log_receipt=self.log.finish()
        finally:self.null.close()
    def failure_cleanup(self):
        if self.owner_terminal:return {'owner_terminal':True,'guest_success_not_implied':True}
        if not self.started:
            if hasattr(self,'log'):
                if hasattr(self,'null'):self.finish_logs()
                else:self.log_receipt=self.log.finish()
            self.owner_terminal=True
            return {'owner_terminal':True,'pipe_log':getattr(self,'log_receipt',None)}
        if getattr(self.family,'job_terminate_attempted',False):
            # A failed normal service cleanup already owns this one attempt.
            # Preserve its exact receipt; do not replace it with a retry record.
            return {'owner_terminal':self.family.closed,'forced_job_cleanup':True,'receipt':getattr(self,'cleanup',None),'no_repeat':True,'pipe_log':getattr(self,'log_receipt',None)}
        self.cleanup={}
        if not self.family.resume_attempted:
            self.pre_resume_cleanup_attempted=True
            self.cleanup=self.family.finish_failed_start(authorization=True)
            self.owner_terminal=self.cleanup.get('owner_terminal') is True
            if self.owner_terminal:self.finish_logs()
            return {'owner_terminal':self.owner_terminal,'pre_resume_cleanup':True,'receipt':self.cleanup,'pipe_log':getattr(self,'log_receipt',None)}
        if self.p.get('request_fault_test') is not None:
            require(self.p['request_fault_test'].get('cleanup_authorized') is True,'fault cleanup authorization disabled; retain owner')
        self.forced_cleanup_requested=True
        self.m.finish_test_job(self.family,authorization=True,timeout_ms=5000,receipt=self.cleanup)
        confirmed=self.cleanup.get('termination_confirmed') is True and self.cleanup.get('handles_closed') is True
        if confirmed:
            self.owner_terminal=True;self.finish_logs()
        result={'owner_terminal':confirmed,'forced_job_cleanup':True,'receipt':self.cleanup,'pipe_log':getattr(self,'log_receipt',None),'resource_state':'UNKNOWN'}
        if confirmed:
            log=self.log_receipt
            if log.get('eof') is True and log.get('finished') is True and log.get('reader_alive') is False and not log.get('error') and not log.get('dropped') and not log.get('write_close_unknown'):
                try:result['storage_snapshot']=self.snapshot();result['resource_state']='REAPED'
                except BaseException as exc:result['storage_observation_error']=repr(exc)
        return result
    def finalize_if_terminal(self):
        if self.owner_terminal:return True
        if getattr(self.family,'terminal_close_attempted',False) and not self.family.closed:return False
        if not self.family.resume_attempted:
            # Observe the same root termination attempt; never resend TerminateProcess.
            if not self.family.abort_attempted and self.family._root_handle is not None:return False
            self.cleanup=self.family.finish_failed_start(authorization=True,timeout_ms=1)
            self.owner_terminal=self.cleanup.get('owner_terminal') is True
            if self.owner_terminal:self.finish_logs()
            return self.owner_terminal
        if self.family.closed:self.owner_terminal=True;return True
        # A pending TerminateJob is observed, never issued twice.
        if self.family.wait()['state']=='EMPTY' and all(m.wait(0)['state']=='EXITED' for m in self.family.members.values()):
            self.family.close_terminal();self.owner_terminal=True
            self.finish_logs()
        return self.owner_terminal
    def start(self):
        import importlib.util,sys,msvcrt
        path=pathlib.Path(__file__).parent/'vendor/owned_job.py'
        spec=importlib.util.spec_from_file_location('md_owned_job',path);m=importlib.util.module_from_spec(spec);sys.modules[spec.name]=m;spec.loader.exec_module(m);self.m=m
        allowed={x:sha(x) for x in (self.p['vm'],self.p['virtmgr'],self.p['crosvm'],'C:/Windows/System32/conhost.exe')}
        unset=sorted(set(k for k in os.environ if k not in self.plan['environment'])|{'VIRTMGR_DEBUG_POLICY_JSON','GFXSTREAM_KVM_COHERENT_CACHED_ONLY','VIRTMGR_CAPTURE_GUEST_CONSOLE','VIRTMGR_CAPTURE_CROSVM_STDIO'})
        doc={'version':1,'executable':self.plan['argv'][0],'args':self.plan['argv'][1:],'cwd':str(self.run),'allowed_images':allowed,'env_set':self.plan['environment'],'env_unset':unset}
        path=self.run/'command.json';path.write_text(json.dumps(doc,indent=2))
        from pipe_log import PipeLog
        self.log=PipeLog(self.run/'driver.log');self.null=open(os.devnull,'rb')
        try:
            self.family=m.OwnedFamily.spawn_reviewed(m.ReviewedCommand.from_file(path,sha(path)),stdio=(msvcrt.get_osfhandle(self.null.fileno()),msvcrt.get_osfhandle(self.log.write_fd),msvcrt.get_osfhandle(self.log.write_fd)))
            self.started=True # Own the returned family before closing parent pipe ends.
        except m.OwnedSpawnError as exc:self.family=exc.family;self.started=True;self.spawn_failure=getattr(exc.family,'spawn_failure',{'phase':exc.phase,'exception':repr(exc.original)});raise
        finally:self.log.sent()
        self.started=True
    def observe(self):
        if not self.started:return
        self.observation_evidence={}
        self.m.recorded_observe(self.family,self.observation_evidence)
        self.guests=[x for x in self.family.members.values() if x.initial['exe_path']==self.m.canon(self.p['crosvm'])]
    def guest_present(self):return bool(getattr(self,'guests',[]))
    def guest_done(self):
        # GPU-on run-mp owns six crosvm members; existing GPU-off run owns one.
        # Do not treat an unexpected topology as a disposable service.
        from entry import gpu_mode
        expected=1 if gpu_mode(self.cfg)=='off' else 6
        self.observe();return len(getattr(self,'guests',[]))==expected and all(x.wait(0)['state']=='EXITED' for x in self.guests)
    def guest_exit(self):
        require(self.guest_done(),'Guest not done')
        require(all(x.wait(0).get('exit_code')==0 and not x.wait(0).get('terminate_requested') for x in self.guests),'Guest forced/nonzero')
        return super().guest_exit()
    def cleanup_services(self):
        require(self.guest_done(),'do not terminate active Guest Job')
        root=self.family.root.wait(0)
        require(root['state']=='EXITED' and root.get('exit_code')==0,'vm client not naturally exited')
        self.cleanup={};self.m.finish_test_job(self.family,authorization=True,receipt=self.cleanup)
        require(self.cleanup.get('termination_confirmed') and self.cleanup.get('handles_closed'),'Job cleanup unknown')
        self.owner_terminal=True
        self.finish_logs()
        (self.run/'cleanup.json').write_text(json.dumps(self.cleanup,indent=2))
        require(self.log_receipt['eof'] and self.log_receipt['finished'] and not self.log_receipt['reader_alive'] and not self.log_receipt['error'] and not self.log_receipt['dropped'],'log incomplete; preserve UNKNOWN')
    def snapshot(self):
        import importlib.util
        path=pathlib.Path(__file__).parent/'vendor/windows_state.py';spec=importlib.util.spec_from_file_location('md_state',path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
        return m.snapshot(self.state)

class PosixOwner(NativeOwner):
    def release_service_handles(self):
        require(not getattr(self,'service_signal_unknown',False),'service signal outcome unknown')
        require(not getattr(self,'service_close_unknown',False),'service close outcome unknown')
        for pid,fd in list(getattr(self,'service_fds',{}).items()):
            if not select.select([fd],[],[],0)[0]:return False
            try:os.close(fd)
            except BaseException:
                self.service_close_unknown=True
                raise
            del self.service_fds[pid]
        return True
    def bind_linux_fault_members(self):
        # Explicit opt-in before a fault request; never called by normal start.
        require(self.p['platform']=='linux','Linux only')
        require(not hasattr(self,'fault_owner'),'fault binding not replayable')
        self.observe()
        require(self.child.poll()==0,'persistent launcher must finish successfully before arming')
        expected={pid:{'start':row['start'],'sid':row['sid'],'exe':row['exe'],
                       'cmd':row['cmd'],'role':'guest' if row['kind']=='crosvm' else row['kind']}
                  for pid,row in self.owned.items()}
        current=self.linux_fault_inventory()
        require(current==expected,'persistent runtime membership/identity mismatch')
        from linux_pidfd import BoundInstance
        self.fault_owner=BoundInstance(expected,self.linux_fault_inventory,os.getpid())
        self.fault_owner.bind()
    def linux_fault_inventory(self):
        self.verify_proc_view()
        out={}
        for q in pathlib.Path('/proc').iterdir():
            if not q.name.isdigit():continue
            try:first=self.linux_stat(q)
            except FileNotFoundError as exc:
                if exc.errno!=errno.ENOENT:raise
                self.verify_proc_view();continue
            if first['sid']!=self.child.pid:continue
            if first['state']=='Z':continue # pidfd readiness must independently agree
            pid=int(q.name)
            exe=str((q/'exe').resolve(strict=True));cmd=(q/'cmdline').read_bytes().replace(b'\0',b' ').decode()
            after=self.linux_stat(q)
            require(all(first[k]==after[k] for k in ('start','sid')),'inventory identity raced; no signal')
            role='launcher' if pid==self.child.pid else 'client' if pid==getattr(self,'fault_client_pid',None) else ('guest' if self.owned.get(pid,{}).get('kind')=='crosvm' else self.owned.get(pid,{}).get('kind','UNOWNED'))
            out[pid]={'start':after['start'],'sid':after['sid'],'exe':exe,'cmd':cmd,'role':role}
        self.verify_proc_view()
        return out
    def terminate_linux_fault_once(self,*,authorized):
        require(hasattr(self,'fault_owner'),'no prebound ownership')
        # Regular-file logging here is separate from the caller's SSH reader.
        # Do not close the file while any captured process/session remains live.
        def release_log():
            if not self.session_empty():return False
            if not self.log.closed:self.log.close()
            return self.log.closed
        self.forced_linux_cleanup_requested=True
        result=self.fault_owner.terminate_once(authorized=authorized,
                    logs_released=release_log,storage_snapshot=self.snapshot)
        self.cleanup=result
        self.owner_terminal=result.get('resource_state')=='REAPED'
        return result
    def linux_proc_view(self):
        root=pathlib.Path('/proc');info=root.lstat()
        require(stat.S_ISDIR(info.st_mode),'proc root is not a directory')
        entries=[]
        for line in (root/'self/mountinfo').read_text().splitlines():
            left,right=line.split(' - ',1)
            if left.split()[4]=='/proc':
                require(right.split()[0]=='proc','proc mount filesystem changed')
                entries.append(line)
        require(len(entries)==1,'proc mount identity unavailable/ambiguous')
        own=(root/'self/stat').read_text()
        require(int(own.split(' ',1)[0])==os.getpid(),'proc self identity changed')
        start=own.rsplit(')',1)[1].split()[19]
        return (info.st_dev,info.st_ino,entries[0],os.getpid(),start)
    def verify_proc_view(self):
        require(hasattr(self,'proc_view'),'no original proc ownership view')
        require(self.linux_proc_view()==self.proc_view,'proc ownership view changed')
    def linux_absence(self,q):
        # Path.exists may suppress EACCES/EIO depending on Python version.
        # Only exact directory ENOENT in the original proc view means gone.
        try:q.lstat()
        except FileNotFoundError as exc:
            require(exc.errno==errno.ENOENT,'non-ENOENT absence query')
            self.verify_proc_view()
            self.observation_evidence.update(state='GONE',absence_errno=exc.errno)
            return False
        raise RuntimeError('incomplete owned proc observation; PID directory remains')
    def same_identity(self,record,current):
        # Parent lineage establishes ownership once; reparenting is not exec or
        # PID reuse. Session, generation, executable and argv remain mandatory.
        return all(record[k]==current[k] for k in ('start','sid','exe','cmd'))
    def linux_stat(self,q):
        fields=(q/'stat').read_text().rsplit(')',1)[1].split()
        return {'ppid':int(fields[1]),'sid':int(fields[3]),'start':fields[19],'state':fields[0]}
    def linux_live(self,pid,row):
        q=pathlib.Path('/proc')/str(pid)
        self.observation_evidence={'pid':pid,'operation':'stat','expected':dict(row)}
        self.verify_proc_view()
        try:first=self.linux_stat(q)
        except FileNotFoundError as exc:
            require(exc.errno==errno.ENOENT,'non-ENOENT owned stat failure')
            return self.linux_absence(q)
        self.observation_evidence['stat']=first
        require(all(first[k]==row[k] for k in ('start','sid')),'PID generation/session changed; no action')
        try:
            self.observation_evidence['operation']='exe/cmdline'
            current={**first,'exe':str((q/'exe').resolve(strict=True)),'cmd':(q/'cmdline').read_bytes().replace(b'\0',b' ').decode()}
        except FileNotFoundError as exc:
            require(exc.errno==errno.ENOENT,'non-ENOENT owned identity failure')
            # ENOENT is not an exit receipt. A same-generation partial proc
            # record stays pending under the existing shutdown deadline.
            self.observation_evidence.update(operation='recheck after ENOENT',errno=exc.errno)
            try:after=self.linux_stat(q)
            except FileNotFoundError as exc:
                require(exc.errno==errno.ENOENT,'non-ENOENT owned recheck failure')
                return self.linux_absence(q)
            self.observation_evidence['recheck_stat']=after
            require(all(after[k]==row[k] for k in ('start','sid')),'PID generation/session changed during observation')
            self.observation_evidence['state']='EXIT_IDENTITY_PENDING'
            return True
        try:after=self.linux_stat(q)
        except FileNotFoundError as exc:
            require(exc.errno==errno.ENOENT,'non-ENOENT owned final stat failure')
            return self.linux_absence(q)
        require(all(after[k]==row[k] for k in ('start','sid')),'PID generation/session changed after identity')
        require(self.same_identity(row,current),'PID identity changed; no action')
        self.observation_evidence.update(state='LIVE',ppid_changed=first['ppid']!=row['ppid'])
        return True
    def session_empty(self):
        # Independent of launcher return code and Guest discovery. A failed or
        # very short-lived launch must be able to finish UNKNOWN without a VM.
        if self.child.poll() is None:return False
        cp=subprocess.run(['ps','-axo','pid='],capture_output=True,text=True,timeout=5,check=True)
        require(not cp.stderr.strip(),'process inventory incomplete')
        for word in cp.stdout.split():
            pid=int(word)
            try:
                if os.getsid(pid)==self.child.pid:return False
            except ProcessLookupError:continue
        return True
    def finalize_if_terminal(self):
        if self.owner_terminal:return True
        if hasattr(self,'fault_owner'):
            # Observe only after a completed binding and one-shot cleanup attempt.
            if not self.fault_owner.armed or not self.fault_owner.cleanup_attempted:return False
            if not hasattr(self,'fault_observe_deadline'):
                self.fault_observe_deadline=time.monotonic()+30;self.fault_observe_calls=0
            if time.monotonic()>=self.fault_observe_deadline or self.fault_observe_calls>=30:return False
            self.fault_observe_calls+=1
            def release_log():
                if not self.session_empty():return False
                if not self.log.closed:self.log.close()
                return self.log.closed
            self.cleanup=self.fault_owner.observe_once(logs_released=release_log,storage_snapshot=self.snapshot)
            self.owner_terminal=self.cleanup.get('resource_state')=='REAPED'
            return self.owner_terminal
        if self.session_empty():
            if not self.release_service_handles():return False
            self.log.close();self.owner_terminal=True;return True
        # Successful Guest observation can release only this run's idle service.
        if self.child.poll() in (None,0) and self.guest_done():self.cleanup_services()
        return self.owner_terminal
    def failure_cleanup(self):
        if hasattr(self,'fault_owner'):
            if getattr(self,'fault_attempted',False) and self.fault_owner.armed and not self.fault_owner.cleanup_attempted:
                self.terminate_linux_fault_once(authorized=self.p.get('request_fault_test',{}).get('cleanup_authorized') is True)
            return {'owner_terminal':self.owner_terminal,'resource_state':'REAPED' if self.owner_terminal else 'UNKNOWN',
                    'storage_snapshot':getattr(self,'cleanup',{}).get('storage_snapshot'),
                    'receipt':getattr(self,'cleanup',None),'no_repeat':True}
        if not self.started:return {'owner_terminal':True}
        terminal=self.finalize_if_terminal()
        return {'owner_terminal':terminal,'guest_success_not_implied':True,'retained_session':not terminal}
    def actual_mac_executable(self,pid):
        import ctypes
        lib=ctypes.CDLL('/usr/lib/libproc.dylib',use_errno=True)
        lib.proc_pidpath.argtypes=[ctypes.c_int,ctypes.c_void_p,ctypes.c_uint32];lib.proc_pidpath.restype=ctypes.c_int
        b=ctypes.create_string_buffer(4096);n=lib.proc_pidpath(pid,b,len(b))
        require(n>0,'owned executable query unavailable/denied')
        return str(pathlib.Path(os.fsdecode(b.value)).resolve())
    def table(self):
        out={}
        if self.p['platform']=='linux':
            current_view=self.linux_proc_view()
            if not hasattr(self,'proc_view'):self.proc_view=current_view
            require(current_view==self.proc_view,'proc ownership view changed during inventory')
            for q in pathlib.Path('/proc').iterdir():
                if not q.name.isdigit():continue
                try:
                    comm=(q/'comm').read_text().strip()
                    if comm not in ('crosvm','virtmgr','binder-rpc-host'):continue
                    fields=(q/'stat').read_text().rsplit(')',1)[1].split()
                    out[int(q.name)]={'ppid':int(fields[1]),'sid':int(fields[3]),'start':fields[19],'exe':str((q/'exe').resolve(strict=True)),'cmd':(q/'cmdline').read_bytes().replace(b'\0',b' ').decode()}
                except FileNotFoundError:continue
        else:
            cp=subprocess.run(['ps','-axo','pid=,ppid=,lstart=,command='],capture_output=True,text=True,timeout=5,check=True)
            for line in cp.stdout.splitlines():
                f=line.strip().split(None,7)
                if len(f)!=8:continue
                cmd=f[7]
                if '/crosvm ' not in cmd and '/virtmgr ' not in cmd:continue
                # Paths contain spaces: match full pinned binary prefix, not split argv0.
                kind=next((k for k in ('crosvm','virtmgr') if cmd.startswith(self.p['dist']+'/macos/bin/'+k+' ')),None)
                if kind:out[int(f[0])]={'ppid':int(f[1]),'sid':os.getsid(int(f[0])),'start':' '.join(f[2:7]),'exe':self.actual_mac_executable(int(f[0])),'cmd':cmd}
        return out
    def start(self):
        require(not any(re.search(r'--cid[ =]'+str(self.cfg['cid'])+r'\b',r['cmd']) for r in self.table().values()),'CID already in use')
        self.log=(self.run/'driver.log').open('xb')
        # No Python preexec after a control reader thread was created.
        # The pinned Mac launcher already sets its child's FD soft limit, bounded
        # by the existing hard limit. Do not duplicate it on the controller.
        self.child=subprocess.Popen(self.plan['argv'],cwd=self.p['repo'],env=self.plan['environment'],stdin=subprocess.DEVNULL,stdout=self.log,stderr=subprocess.STDOUT,start_new_session=True)
        self.started=True
    def observe(self):
        table=self.table()
        for pid,row in table.items():
            if row['exe']==str(pathlib.Path(self.p['virtmgr']).resolve()) and str(self.run/'service'/'virtmgr-service.sock') in row['cmd'] and os.getsid(pid)==self.child.pid:
                self.owned.setdefault(pid,{**row,'kind':'virtmgr'})
        for pid,row in table.items():
            parent=self.owned.get(row['ppid']);actual_parent=table.get(row['ppid'])
            if parent and actual_parent and self.same_identity(parent,actual_parent) and row['sid']==parent['sid'] and row['exe']==str(pathlib.Path(self.p['crosvm']).resolve()) and re.search(r'--cid[ =]'+str(self.cfg['cid'])+r'\b',row['cmd']):self.owned.setdefault(pid,{**row,'kind':'crosvm'})
        self.current=table
        (self.run/'owned.json').write_text(json.dumps(self.owned,indent=2))
        if hasattr(self,'child') and self.child.poll() not in (None,0):raise RuntimeError('launcher failed')
    def live(self,pid,row):
        # Owned PID lookup is unfiltered; renamed/execed process is UNKNOWN, not gone.
        if self.p['platform']=='linux':
            return self.linux_live(pid,row)
        else:
            cp=subprocess.run(['ps','-p',str(pid),'-o','pid=,ppid=,lstart=,command='],capture_output=True,text=True,timeout=5)
            if cp.returncode==1 and not cp.stdout.strip() and not cp.stderr.strip():return False
            require(cp.returncode==0 and not cp.stderr.strip(),'owned process query failed')
            f=cp.stdout.strip().split(None,7);require(len(f)==8 and int(f[0])==pid,'owned process parse failed')
            require(' '.join(f[2:7])==row['start'] and f[7]==row['cmd'],
                    'PID identity changed before session query; no action')
            try:
                sid=os.getsid(pid)
            except ProcessLookupError as exc:
                if exc.errno!=errno.ESRCH:raise
                # ps succeeded, but the owned process may have exited before
                # getsid. Only a new, successful absent-PID observation closes
                # this window; any visible PID or query error stays UNKNOWN.
                again=subprocess.run(['ps','-p',str(pid),'-o','pid=,ppid=,lstart=,command='],capture_output=True,text=True,timeout=5)
                if again.returncode==1 and not again.stdout.strip() and not again.stderr.strip():
                    self.observation_evidence={'state':'EXITED','reason':'getsid ESRCH followed by ps absence','pid':pid,'identity':row}
                    return False
                raise RuntimeError('session query raced; PID visible or absence unproven') from exc
            current={'ppid':int(f[1]),'sid':sid,'start':' '.join(f[2:7]),'exe':self.actual_mac_executable(pid),'cmd':f[7]}
        require(self.same_identity(row,current),'PID identity changed; no action')
        return True
    def guest_present(self):return any(v['kind']=='crosvm' for v in self.owned.values())
    def guest_done(self):
        self.observe();return self.guest_present() and not any(self.live(pid,row) for pid,row in self.owned.items() if row['kind']=='crosvm')
    def guest_exit(self):
        super().guest_exit()
        text=(self.run/'logs'/'vm-run-app.log').read_text(errors='replace')
        for pid,row in self.owned.items():
            if row['kind']=='crosvm':
                require(re.search(r'after SharedChild::spawn cid='+str(self.cfg['cid'])+r' pid='+str(pid)+r'\b',text),'spawn association missing')
                require(re.search(r'crosvm\('+str(pid)+r'\) exited with status exit status: 0(?:\s|$)',text),'matching child wait0 missing')
        return 0
    def cleanup_services(self):
        require(self.guest_done(),'Guest still alive')
        receipt=[]
        for pid,row in self.owned.items():
            if row['kind']=='virtmgr' and self.live(pid,row):
                if self.p['platform']=='linux':
                    require(self.observation_evidence.get('state')=='LIVE','service identity pending; no signal')
                    require(hasattr(os,'pidfd_open') and hasattr(signal,'pidfd_send_signal'),'stable Linux process handle unavailable')
                    if not hasattr(self,'service_fds'):self.service_fds={}
                    if not hasattr(self,'service_term_attempts'):self.service_term_attempts=set()
                    if not hasattr(self,'service_cleanup_deadline'):self.service_cleanup_deadline=time.monotonic()+5
                    require(not getattr(self,'service_signal_unknown',False),'service signal result unknown; retain')
                    if pid not in self.service_fds:self.service_fds[pid]=os.pidfd_open(pid,0)
                    fd=self.service_fds[pid]
                    if not select.select([fd],[],[],0)[0] and pid not in self.service_term_attempts:
                        require(self.live(pid,row) and self.observation_evidence.get('state')=='LIVE','pinned service identity pending')
                        self.service_term_attempts.add(pid)
                        try:signal.pidfd_send_signal(fd,signal.SIGTERM,None,0)
                        except BaseException:
                            self.service_signal_unknown=True
                            raise
                    remaining=max(0,self.service_cleanup_deadline-time.monotonic())
                    require(bool(select.select([fd],[],[],remaining)[0]),'service cleanup deadline; original handle retained')
                    require(self.release_service_handles(),'original service handle exit pending')
                    receipt.append({'pid':pid,'identity':row,'signal':'TERM','role':'idle_service','target':'verified_pidfd','exit_observed_on_same_handle':True})
                else:
                    raise RuntimeError('Mac requires the audit-token owner; no PID signal fallback')
        require(self.child.poll()==0,'launcher has not completed successfully')
        require(self.session_empty(),'unaccounted session process remains')
        require(self.release_service_handles(),'original service handle still pending')
        self.log.close();self.owner_terminal=True
        self.cleanup={'services':receipt,'session_empty':True,'regular_log_closed':True,'inner_pipe_eof':'NOT_APPLICABLE_REGULAR_FILE'}
        (self.run/'cleanup.json').write_text(json.dumps(self.cleanup,indent=2))
    def snapshot(self):
        out={}
        for name in ('instance.img','storage.img','app.idsig'):
            q=self.state/name;require(q.is_file() and not q.is_symlink(),'state missing/link')
            cmd=['fuser',str(q)] if self.p['platform']=='linux' else ['/usr/sbin/lsof','-t',str(q)]
            cp=subprocess.run(cmd,capture_output=True,timeout=5);require(cp.returncode==1 and not cp.stdout and not cp.stderr,'state holder/query failure/unknown')
            st=q.stat();out[name]={'dev':st.st_dev,'inode':st.st_ino,'bytes':st.st_size,'sha256':sha(q)}
        return out
