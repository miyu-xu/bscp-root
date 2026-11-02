"""Bounded recording, continuously drained anonymous pipe; never conceal loss."""
import os,threading,traceback
class PipeLog:
    def __init__(self,path,limit=64*1024*1024):
        self.file=open(path,'xb');self.read_fd,self.write_fd=os.pipe()
        self.limit=limit;self.kept=0;self.dropped=0;self.error=None;self.eof=False;self.finished=False;self.errors=[]
        self.thread=threading.Thread(target=self._read,daemon=True);self.thread.start()
    def _read(self):
        try:
            while True:
                data=os.read(self.read_fd,65536)
                if not data:self.eof=True;break
                n=min(len(data),max(0,self.limit-self.kept)) if not self.error else 0
                try:
                    if n:self.file.write(data[:n]);self.file.flush();self.kept+=n
                except BaseException as e:self.fault('write/flush',e);n=0
                self.dropped+=len(data)-n
        except BaseException as e:self.fault('read',e)
        finally:
            for name,close in (('close_read',lambda:os.close(self.read_fd)),('close_file',self.file.close)):
                try:close()
                except BaseException as e:self.fault(name,e)
            self.finished=True
    def fault(self,operation,error):
        if self.error is None:self.error=repr(error)
        self.errors.append({'operation':operation,'exception':repr(error),'traceback':traceback.format_exc()})
    def sent(self):
        if getattr(self,'write_close_attempted',False):raise RuntimeError('pipe write close already attempted; no replay')
        self.write_close_attempted=True
        try:os.close(self.write_fd)
        except BaseException as e:self.fault('close_write',e);raise
        self.write_fd=None
    def finish(self,timeout=5):
        if not 0<=timeout<=5:raise ValueError('bounded log join required')
        if self.write_fd is not None and not getattr(self,'write_close_attempted',False):
            try:self.sent()
            except BaseException:pass # fault is retained; never retry uncertain close.
        self.thread.join(timeout)
        return {'eof':self.eof,'finished':self.finished,'reader_alive':self.thread.is_alive(),'kept':self.kept,'dropped':self.dropped,'error':self.error,'errors':self.errors,'write_close_unknown':self.write_fd is not None}
