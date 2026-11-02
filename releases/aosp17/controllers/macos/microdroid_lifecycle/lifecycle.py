"""One foreground owner; no socket listener, PID reattach, or background scheduler."""
import queue,threading,time,re,traceback,copy

class Unknown(RuntimeError):pass

def supervise(owner,commands,timeout,emit,clock=time.monotonic,sleep=time.sleep):
    result={'state':'STARTING','stop_reply':None,'guest_exit':None,'service_cleaned':False,
            'storage_released':None,'storage_state':'UNVERIFIED','forced_guest':False,'events':[],
            'task_result':'NOT_REQUESTED','resource_state':'UNKNOWN','needsRecovery':False}
    def note(event):
        result['events'].append(event)
        if len(result['events'])>64:del result['events'][:-64]
        emit(dict(result))
    deadline=clock()+timeout
    last_request=0
    try:
        owner.start();note('owner_started')
        while not owner.ready():
            if clock()>=deadline:raise TimeoutError('startup deadline')
            sleep(.05)
        result['state']='RUNNING';result['resource_state']='LIVE';note('owned_payload_ready')
        while True:
            owner.observe()
            if owner.guest_done():raise Unknown('Guest ended without this controller STOP')
            if clock()>=deadline:
                result['deadline_reached']=True;break
            try:command=commands.get_nowait()
            except queue.Empty:sleep(.05);continue
            if command=='status':note('live_status');continue
            if command=='__INPUT_ERROR__':raise Unknown('control input reader failed')
            if command.startswith('request '):
                match=re.fullmatch(r'request ([1-9][0-9]{0,8}) (INFO|PROPERTIES|PROBE|VKENUM|PIXEL|COHERENT|GET|SET -?[0-9]{1,19}|ADD -?[0-9]{1,19}|VADD [^\r\n]+)',command)
                if not match or int(match[1])<=last_request:
                    note('rejected_request');continue
                if match[2].startswith('VADD '):
                    try:
                        from compute_protocol import parse
                        parse(match[2])
                    except ValueError:
                        note('rejected_request');continue
                last_request=int(match[1])
                result['task_result']='UNKNOWN'
                result['request']={'id':last_request,'command':match[2],'state':'PENDING'}
                # The supervisor itself owns the only request operation. Queued
                # stop/EOF and deadlines cannot run concurrently with this call.
                owner.transport_unknown=True
                try:
                    note('request_pending')
                    owner.transport_unknown=False
                    response=owner.request(match[2])
                    if not isinstance(response,str) or not response.endswith('\n') or response.count('\n')!=1:
                        raise Unknown('incomplete request response')
                    if match[2].startswith('VADD '):
                        from compute_protocol import validate_response
                        result['request']['task']=validate_response(match[2],response)
                    result['request'].update(state='COMPLETE',response=response)
                    result['task_result']=result['request'].get('task',{}).get('outcome','UNKNOWN')
                    if match[2] in ('GET',) or match[2].startswith(('SET ','ADD ')):
                        if re.fullmatch(r'VALUE -?(?:0|[1-9][0-9]*)\n',response):
                            value=int(response[6:-1])
                            if -(1<<63)<=value<(1<<63):result['task_result']='SUCCESS'
                    elif match[2]=='INFO' and response=='HEADLESS_COUNTER_V1 GUEST_ENCRYPTED_STORAGE\n':
                        result['task_result']='SUCCESS'
                    note('request_complete')
                except BaseException:
                    owner.transport_unknown=True
                    result['request']['state']='UNKNOWN'
                    raise
                continue
            if command in ('stop','__EOF__'):break
            note('rejected_control_command')
        result['state']='STOPPING';note('stop_requested')
        # The owner sends only existing sample-payload STOP; no new Guest protocol.
        reply=owner.stop();result['stop_reply']=reply.strip()
        if reply!='BYE\n':raise Unknown('STOP response missing/malformed; never retry')
        end=clock()+30
        while not owner.guest_done():
            if clock()>=end:raise TimeoutError('Guest shutdown deadline')
            sleep(.05)
        code=owner.guest_exit()
        if type(code)is not int or code!=0:raise Unknown('Guest did not exit naturally with zero')
        result['guest_exit']=code;note('guest_natural_exit')
        owner.cleanup_services();result['service_cleaned']=True;note('service_cleanup_separate')
        result['pipe_log']=getattr(owner,'log_receipt',None)
        result['cleanup_receipt']=getattr(owner,'cleanup',None)
        result['storage_snapshot']=owner.snapshot()
        if not isinstance(result['storage_snapshot'],dict) or not result['storage_snapshot']:raise Unknown('missing storage release evidence')
        result['storage_released']=True;result['storage_state']='RELEASED'
        result['state']='STOPPED';result['resource_state']='REAPED';note('healthy_stopped')
    except BaseException as exc:
        result['state']='UNKNOWN';result['resource_state']='UNKNOWN';result['needsRecovery']=True;result['error']=repr(exc)
        result['forced_guest']=None;result['normal_guest_exit_proven']=False
        result['first_exception']={'exception':repr(exc),'traceback':traceback.format_exc(),
            'operation':copy.deepcopy(getattr(getattr(owner,'family',None),'last_operation',None)),
            'observation':copy.deepcopy(getattr(owner,'observation_evidence',None))}
        result['spawn_failure']=copy.deepcopy(getattr(owner,'spawn_failure',None))
        def safe_note(event):
            try:note(event)
            except BaseException as logging_error:result['recording_error']=repr(logging_error)
        safe_note('failure_preserved')
        result['fault_receipt']=copy.deepcopy(getattr(owner,'fault_receipt',None))
        # No success inferred from cleanup; adapter never attaches to historical PID.
        try:
            result['failure_cleanup']=owner.failure_cleanup()
            if result['failure_cleanup'].get('resource_state')=='REAPED':
                snapshot=result['failure_cleanup'].get('storage_snapshot')
                if not isinstance(snapshot,dict) or not snapshot:raise Unknown('REAPED cleanup lacks storage release evidence')
                result['resource_state']='REAPED'
                result['storage_snapshot']=snapshot
                result['storage_released']=True;result['storage_state']='RELEASED'
        except BaseException as cleanup:result['cleanup_error']=repr(cleanup)
        if getattr(owner,'forced_macos_cleanup_requested',False):
            result['forced_macos_cleanup']=True;result['forced_guest']=None;result['normal_guest_exit_proven']=False
        if getattr(owner,'forced_linux_cleanup_requested',False):
            result['forced_linux_cleanup']=True;result['forced_guest']=None;result['normal_guest_exit_proven']=False
        if result.get('failure_cleanup',{}).get('forced_job_cleanup') or getattr(owner,'forced_cleanup_requested',False):
            result['forced_job_cleanup']=True
            result['forced_guest']=None # Which Guest was still live at Job termination is not inferred.
            result['normal_guest_exit_proven']=False
        if getattr(owner,'cleanup',None):result['cleanup_receipt']=owner.cleanup
        if getattr(owner,'pre_resume_cleanup_attempted',False):
            result['pre_resume_cleanup_attempted']=True
            result['suspended_root_termination_attempted']=bool(getattr(owner.family,'abort_attempted',False))
        result['pipe_log']=getattr(owner,'log_receipt',None)
        result['owner_terminal']=result.get('failure_cleanup',{}).get('owner_terminal') is True
        safe_note('terminal_unknown' if result['owner_terminal'] else 'owner_must_be_retained')
    return result

def stdin_commands(stream):
    q=queue.Queue(maxsize=32)
    def reader():
        try:
            while True:
                line=stream.readline(128)
                if not line:q.put('__EOF__');return
                command=line.strip()
                if len(line)>=128:
                    while line and not line.endswith('\n'):line=stream.readline(128)
                    command='invalid'
                q.put(command)
        except Exception:q.put('__INPUT_ERROR__')
    threading.Thread(target=reader,daemon=True).start()
    return q
