"""Explicit Mac two-case runner. Run only after exact transfer/signal approval."""
import pathlib,subprocess,threading,queue,json,time,sys,hashlib,os,argparse,re
ROOT=pathlib.Path(__file__).resolve().parent

def unique_pairs(pairs):
    result={}
    for key,value in pairs:
        if key in result:raise ValueError('duplicate JSON key: '+key)
        result[key]=value
    return result

def loads(text):return json.loads(text,object_pairs_hook=unique_pairs)
def save(path,value):
    with path.open('x',encoding='utf8') as f:json.dump(value,f,indent=2);f.flush();os.fsync(f.fileno())
def verify():
    m=loads((ROOT/'transfer-manifest.json').read_text())
    for n,v in m.items():
        b=(ROOT/n).read_bytes()
        if len(b)!=v['bytes'] or hashlib.sha256(b).hexdigest()!=v['sha256']:raise RuntimeError('pin mismatch '+n)
class Reader:
    def __init__(self,stream,path):
        self.q=queue.Queue(maxsize=1024);self.eof=False;self.error=None;self.finished=False
        self.stream=stream;self.file=None
        try:self.file=path.open('xb')
        except BaseException as exc:self.error=repr(exc)
        self.thread=threading.Thread(target=self.read,daemon=False)
        try:self.thread.start()
        except BaseException as exc:self.error=repr(exc)
    def read(self):
        decoder=json.JSONDecoder(object_pairs_hook=unique_pairs);buf='';kept=0
        try:
            while True:
                chunk=self.stream.read1(4096)
                if not chunk:self.eof=True;break
                kept+=len(chunk)
                if kept>4*1024*1024:self.error='output cap';continue # still drain
                if self.file is not None and not self.error:
                    try:self.file.write(chunk);self.file.flush()
                    except BaseException as exc:self.error=repr(exc)
                if self.error:continue
                try:buf+=chunk.decode('utf8')
                except UnicodeDecodeError as exc:self.error=repr(exc);continue
                while buf.strip():
                    buf=buf.lstrip()
                    try:value,end=decoder.raw_decode(buf)
                    except json.JSONDecodeError:break
                    except ValueError as exc:self.error=repr(exc);buf='';break
                    try:self.q.put_nowait(value)
                    except queue.Full:self.error='event queue full';break
                    buf=buf[end:]
            if buf.strip() and not self.error:self.error='truncated/invalid JSON tail'
        except BaseException as e:self.error=repr(e)
        finally:
            try:
                if self.file is not None:self.file.close()
                if self.eof:self.stream.close()
            except BaseException as e:self.error=repr(e)
            self.finished=True
    def receipt(self):return dict(eof=self.eof,finished=self.finished,error=self.error,alive=self.thread.is_alive())
def await_event(reader,predicate,seconds=120):
    end=time.monotonic()+seconds
    while time.monotonic()<end:
        if reader.error:raise RuntimeError(reader.error)
        try:v=reader.q.get(timeout=.1)
        except queue.Empty:
            if reader.finished:raise RuntimeError('EOF before required event')
            continue
        if predicate(v):return v
    raise TimeoutError('event deadline; not completion')
def completed(v,rid,response):return v.get('request',{}).get('id')==rid and v['request'].get('state')=='COMPLETE' and v['request'].get('response')==response
def audit_proven(cleanup,members):
    handles=cleanup.get('audit_handles',[])
    signals={str(s.get('pid')):s for s in cleanup.get('signals',[])}
    return (len(handles)==2 and {str(h.get('pid')) for h in handles}==set(members)
      and all(h.get('token')==members[str(h['pid'])]['audit_token'] and h.get('registration_verified') is True
        and type(h.get('pid')) is int
        and ((h.get('signal_attempted') is True and type(h.get('signal_return')) is int and h['signal_return']==0 and str(h['pid']) in signals and signals[str(h['pid'])].get('identity')==members[str(h['pid'])])
             or (h.get('signal_attempted') is False and str(h['pid']) not in signals))
        and h.get('exited') is True and h.get('kq_closed') is True and h.get('port_closed') is True
        and len(h.get('events',[]))==1 and h['events'][0]['ident']==h['pid']
        and h['events'][0]['filter']==-5 and h['events'][0]['fflags'] & 0x80000000
        and not h['events'][0]['flags'] & 0x4000
        for h in handles))

def eligible(v,record,reader,exitcode,disk,instance="t",mode="timeout"):
    def exact(a,b):return json.dumps(a,sort_keys=True)==json.dumps(b,sort_keys=True)
    cleanup=v.get('failure_cleanup',{}).get('receipt',{})
    members=cleanup.get('members',{})
    roles=sorted(x.get('role','') for x in members.values())
    signals=cleanup.get('signals',[])
    ids=[str(x.get('pid')) for x in signals]
    proof=(roles==['guest','virtmgr'] and cleanup.get('processes_exited') is True
        and cleanup.get('retained_audit_handles')==[] and not cleanup.get('errors')
        and 1<=len(ids)<=2 and len(ids)==len(set(ids)) and set(ids)<=set(members)
        and all(set(row)=={'start','sid','exe','cmd','role','audit_token'} for row in members.values())
        and all(x.get('signal')=='TERM' and x.get('attempt')=='RETURNED_NOT_EXIT_PROOF' for x in signals)
        and audit_proven(cleanup,members)
        and v.get('normal_guest_exit_proven') is False and v.get('forced_macos_cleanup') is True
        and exact(record.get('exit'),v) and not v.get('cleanup_error')
        and v.get('request',{}).get('command')=='ADD 1' and type(v.get('request',{}).get('id')) is int and v['request']['id']==5 and v['request'].get('state')=='UNKNOWN'
        and (v.get('fault_receipt') or {}).get('send_returned') is True and v['fault_receipt'].get('response_delivered') is False
        and v['fault_receipt'].get('instance')==instance and v['fault_receipt'].get('mode')==mode and v['fault_receipt'].get('command')=='ADD 1'
        and set(disk)=={'instance.img','storage.img','app.idsig'}
        and all(set(row)=={'dev','inode','bytes','sha256'} and all(type(row[k]) is int for k in ('dev','inode','bytes')) and row['bytes']>0 and isinstance(row['sha256'],str) and len(row['sha256'])==64 and all(c in '0123456789abcdef' for c in row['sha256']) for row in disk.values()))
    return (proof and v.get('task_result')=='UNKNOWN' and v.get('resource_state')=='REAPED' and v.get('needsRecovery') is True
        and v.get('storage_state')=='RELEASED' and v.get('storage_released') is True
        and record.get('state')=='UNKNOWN' and record.get('needsRecovery') is True and record.get('task_result')=='UNKNOWN' and record.get('resource_state')=='REAPED'
        and set(reader)=={'eof','finished','error','alive'} and reader['eof'] is True and reader['finished'] is True and reader['error'] is None and reader['alive'] is False and type(exitcode) is int and exitcode==0
        and bool(disk) and exact(disk,v.get('storage_snapshot')) and exact(disk,record.get('storage_snapshot')))
def retain(process,reader,path,error):
    try:save(path,dict(state='UNKNOWN',error=repr(error),reader=reader.receipt(),controller_exit=process.poll(),no_retry=True))
    except BaseException:pass # never let recording failure discard a live owner/reader
    # Never cancel the reader/controller or close their live streams on error.
    while process.poll() is None or reader.thread.is_alive():time.sleep(1)
    if process.stdin is not None and not process.stdin.closed:process.stdin.close()
    raise RuntimeError('failed case retained; second case forbidden') from error
def case(key,cid,port):
    pkg=ROOT/key;name=key;out=ROOT/'evidence'/key;out.mkdir(parents=True,exist_ok=False)
    entry=[sys.executable,str(pkg/'entry.py'),'--platform','macos']
    for args in (['check','--gpu','off'],['create','--name',name,'--cid',str(cid),'--port',str(port),'--gpu','off']):
        cp=subprocess.run(entry+args,capture_output=True,timeout=120)
        save(out/(args[0]+'.json'),dict(code=cp.returncode,out=cp.stdout.decode(),err=cp.stderr.decode()))
        if cp.returncode:raise RuntimeError('prelaunch check/create failed; no retry')
    proc=subprocess.Popen(entry+['start','--name',name,'--gpu','off'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    reader=Reader(proc.stdout,out/'controller.log')
    try:
        await_event(reader,lambda v:v.get('state')=='RUNNING')
        for rid,cmd,reply in [(1,'INFO','HEADLESS_COUNTER_V1 GUEST_ENCRYPTED_STORAGE\n'),(2,'GET','VALUE 0\n'),(3,'SET 118','VALUE 118\n'),(4,'GET','VALUE 118\n')]:
            proc.stdin.write(f'request {rid} {cmd}\n'.encode());proc.stdin.flush()
            await_event(reader,lambda v:completed(v,rid,reply))
        proc.stdin.write(b'request 5 ADD 1\n');proc.stdin.flush()
        v=await_event(reader,lambda v:(v.get('fault_receipt') or {}).get('send_returned') is True and v.get('resource_state')=='REAPED')
        code=proc.wait(timeout=30);reader.thread.join(30)
        if reader.thread.is_alive():raise RuntimeError('outer reader not finished')
        tail=[]
        while not reader.q.empty():tail.append(reader.q.get_nowait())
        terminal=[v]+tail
        if any(not isinstance(x,dict) or x.get('state')!='UNKNOWN' or x.get('owner_terminal') is not True for x in terminal):raise RuntimeError('unexpected tail after terminal receipt')
        if not terminal:raise RuntimeError('missing terminal receipt')
        for x in terminal:
            if any(x.get(k)!=terminal[-1].get(k) for k in ('task_result','resource_state','storage_snapshot','failure_cleanup','fault_receipt','needsRecovery')):raise RuntimeError('conflicting terminal receipts')
        v=terminal[-1]
        record=loads((pkg/'prepared'/name/'record.json').read_text())
        # config records live under entry ROOT/prepared; persistent disks under profile instance_root.
        sys.path.insert(0,str(pkg));from native import PosixOwner
        owner=PosixOwner.__new__(PosixOwner);owner.p={'platform':'macos'};owner.state=ROOT/'i'/name/'state'
        disk=owner.snapshot()
        if not (pkg/'prepared'/name/'owner.claim').is_file():raise RuntimeError('failed claim unexpectedly removed')
        if not eligible(v,record,reader.receipt(),code,disk,name,'timeout' if key=='t' else 'disconnect'):raise RuntimeError('incomplete cleanup proof')
        proc.stdin.close()
        save(out/'result.json',dict(result=v,record=record,outer=reader.receipt(),controller_exit=code,disk=disk))
        return True
    except BaseException as exc:retain(proc,reader,out/'unknown.json',exc)
def probe_ready(code,probe,manifest):
    return type(code) is int and code==0 and probe.get('pass') is True and probe.get('manifest_sha256')==manifest
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def prior_t(expected_hash,execution_id,manifest):
    receipt_path=ROOT/'t-complete.json'
    if digest(receipt_path)!=expected_hash:raise RuntimeError('first receipt hash mismatch')
    receipt=loads(receipt_path.read_text())
    if receipt.get('case')!='t' or receipt.get('execution_id')!=execution_id or receipt.get('manifest_sha256')!=manifest or receipt.get('root')!=str(ROOT.resolve()):raise RuntimeError('first execution identity mismatch')
    for name,h in receipt['evidence_hashes'].items():
        if name not in T_EVIDENCE or digest(ROOT/name)!=h:raise RuntimeError('first evidence changed')
    if set(receipt['evidence_hashes'])!=set(T_EVIDENCE):raise RuntimeError('first evidence incomplete')
    first=loads((ROOT/'evidence/t/result.json').read_text())
    record=loads((ROOT/'t/prepared/t/record.json').read_text())
    sys.path.insert(0,str(ROOT/'t'));from native import PosixOwner
    owner=PosixOwner.__new__(PosixOwner);owner.p={'platform':'macos'};owner.state=ROOT/'i/t/state'
    if not (ROOT/'t/prepared/t/owner.claim').is_file():raise RuntimeError('first abnormal claim missing')
    if not eligible(first['result'],record,first['outer'],first['controller_exit'],owner.snapshot(),'t','timeout'):raise RuntimeError('first case no longer eligible')
    start=loads((ROOT/'t-execution.json').read_text())
    if start!=dict(case='t',execution_id=execution_id,manifest_sha256=manifest,root=str(ROOT.resolve())):raise RuntimeError('first reservation mismatch')
    return receipt

T_EVIDENCE=('evidence/t/result.json','evidence/t/controller.log','t/prepared/t/record.json','t/prepared/t/launch-plan.json','t-execution.json','harmless-result.json','harmless-execution.json')

def execute_case(key,execution_id,after_t=None,t_execution_id=None):
    if not re.fullmatch(r'[0-9a-f]{32}',execution_id):raise RuntimeError('execution id requires32 lowercase hex')
    verify();manifest=digest(ROOT/'transfer-manifest.json')
    if key=='d':
        if not after_t or not re.fullmatch(r'[0-9a-f]{64}',after_t) or not t_execution_id:raise RuntimeError('explicit prior first-case receipt required')
        if execution_id==t_execution_id:raise RuntimeError('case execution ids must differ')
        prior_t(after_t,t_execution_id,manifest)
    elif key!='t' or after_t is not None or t_execution_id is not None:raise RuntimeError('invalid first-case invocation')
    # This exclusive reservation prevents retry even when an earlier launch failed.
    save(ROOT/(key+'-execution.json'),dict(case=key,execution_id=execution_id,manifest_sha256=manifest,root=str(ROOT.resolve())))
    if key=='t':
        fresh_harmless(manifest)
    if case(key,64320 if key=='t' else 64321,45971 if key=='t' else 45972) is not True:raise RuntimeError('case not complete')
    evidence=T_EVIDENCE if key=='t' else ('evidence/d/result.json','evidence/d/controller.log','d/prepared/d/record.json','d/prepared/d/launch-plan.json','d-execution.json')
    receipt=dict(case=key,execution_id=execution_id,manifest_sha256=manifest,root=str(ROOT.resolve()),evidence_hashes={n:digest(ROOT/n) for n in evidence},task_result='UNKNOWN',needsRecovery=True,resource_state='REAPED',outer_session_eof='PARENT_MUST_OBSERVE_AFTER_EXIT')
    path=ROOT/(key+'-complete.json');save(path,receipt)
    print(json.dumps(dict(case=key,execution_id=execution_id,manifest_sha256=manifest,receipt_sha256=digest(path),remote_evidence_complete=True)),flush=True)
    return receipt

def fresh_harmless(manifest):
    # Fresh helper exit is mandatory: a JSON write may have succeeded before a
    # later fsync/close failure. Never trust an old success-looking file alone.
    code=subprocess.run([sys.executable,str(ROOT/'harmless.py'),'--approved-own-child']).returncode
    probe=loads((ROOT/'harmless-result.json').read_text())
    if not probe_ready(code,probe,manifest):raise RuntimeError('fresh harmless exit0 and matching proof required')
    save(ROOT/'harmless-execution.json',dict(exit_code=code,proof=probe))

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--approved-case',choices=('t','d'),required=True);parser.add_argument('--execution-id',required=True);parser.add_argument('--after-t');parser.add_argument('--t-execution-id')
    args=parser.parse_args()
    if sys.platform!='darwin':raise RuntimeError('Mac execution required')
    execute_case(args.approved_case,args.execution_id,args.after_t,args.t_execution_id)
if __name__=='__main__':main()
