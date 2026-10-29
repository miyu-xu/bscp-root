import pathlib,sys,unittest,copy,io,tempfile,json
HERE=pathlib.Path(__file__).parent;sys.path.insert(0,str(HERE));import accept as d
class Oracle(unittest.TestCase):
 def proof(self):
  members={str(p):dict(start='s',sid=2,exe='x',cmd='x',role=r,audit_token=[0,0,0,0,0,p,1,99]) for p,r in ((11,'virtmgr'),(12,'guest'))}
  signals=[dict(pid=int(p),identity=m,signal='TERM',attempt='RETURNED_NOT_EXIT_PROOF') for p,m in members.items()]
  handles=[dict(pid=int(p),token=m['audit_token'],registration_verified=True,signal_attempted=True,signal_return=0,exited=True,kq_closed=True,port_closed=True,events=[dict(ident=int(p),filter=-5,flags=16,fflags=0x80000000)]) for p,m in members.items()]
  disk={k:dict(dev=1,inode=i+1,bytes=8,sha256='a'*64) for i,k in enumerate(('instance.img','storage.img','app.idsig'))}
  c=dict(members=members,signals=signals,audit_handles=handles,processes_exited=True,retained_audit_handles=[],errors=[])
  v=dict(state='UNKNOWN',failure_cleanup=dict(receipt=c),forced_macos_cleanup=True,normal_guest_exit_proven=False,request=dict(id=5,command='ADD 1',state='UNKNOWN'),fault_receipt=dict(instance='t',mode='timeout',command='ADD 1',send_returned=True,response_delivered=False),task_result='UNKNOWN',resource_state='REAPED',needsRecovery=True,storage_state='RELEASED',storage_released=True,storage_snapshot=disk)
  record=dict(state='UNKNOWN',needsRecovery=True,task_result='UNKNOWN',resource_state='REAPED',exit=v,storage_snapshot=disk)
  return v,record,dict(eof=True,finished=True,error=None,alive=False),0,disk
 def test_valid_and_types(self):
  args=self.proof();self.assertTrue(d.eligible(*args))
  for key,val in [('eof',1),('finished',1),('alive',0)]:
   a=list(self.proof());a[2][key]=val;self.assertFalse(d.eligible(*a))
  a=list(self.proof());a[3]=False;self.assertFalse(d.eligible(*a))
 def test_signal_identity_and_handle(self):
  for field,value in [('signal_attempted',False),('signal_return',False),('signal_return',1),('token',[0]*8)]:
   a=self.proof();a[0]['failure_cleanup']['receipt']['audit_handles'][0][field]=value
   self.assertFalse(d.eligible(*a))
  a=self.proof();a[0]['failure_cleanup']['receipt']['signals'][0]['identity']={};self.assertFalse(d.eligible(*a))
 def test_duplicate_and_invalid_tail_reader_drains(self):
  root=HERE/'reader-evidence';root.mkdir(exist_ok=True)
  import uuid
  for data in (b'{"state":"UNKNOWN","state":"STOPPED"}\n',b'{"state":"UNKNOWN"}\nJUNK',b'{"state":'):
   stream=io.BufferedReader(io.BytesIO(data));r=d.Reader(stream,root/(uuid.uuid4().hex+'.log'));r.thread.join(3)
   self.assertTrue(r.eof);self.assertTrue(r.finished);self.assertFalse(r.thread.is_alive());self.assertIsNotNone(r.error)
 def test_multiline_and_null(self):
  with self.assertRaises(ValueError):d.loads('{"a":1,"a":2}')
  self.assertEqual(d.loads('{\n"fault_receipt":null\n}'),{'fault_receipt':None})
  a=self.proof();a[0]['fault_receipt']=None;self.assertFalse(d.eligible(*a))
if __name__=='__main__':unittest.main()
