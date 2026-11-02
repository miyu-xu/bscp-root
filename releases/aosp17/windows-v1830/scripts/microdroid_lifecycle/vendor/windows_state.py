import ctypes as C,hashlib
from contextlib import contextmanager
from ctypes import wintypes as W
from pathlib import Path
K=C.WinDLL('kernel32',use_last_error=True)
K.CreateFileW.argtypes=[W.LPCWSTR,W.DWORD,W.DWORD,W.LPVOID,W.DWORD,W.DWORD,W.HANDLE];K.CreateFileW.restype=W.HANDLE
K.CloseHandle.argtypes=[W.HANDLE];K.CloseHandle.restype=W.BOOL
class Info(C.Structure):
 _fields_=[('attributes',W.DWORD),('created',W.FILETIME),('accessed',W.FILETIME),('written',W.FILETIME),('volume',W.DWORD),('size_hi',W.DWORD),('size_lo',W.DWORD),('links',W.DWORD),('index_hi',W.DWORD),('index_lo',W.DWORD)]
K.GetFileInformationByHandle.argtypes=[W.HANDLE,C.POINTER(Info)];K.GetFileInformationByHandle.restype=W.BOOL
K.ReadFile.argtypes=[W.HANDLE,W.LPVOID,W.DWORD,C.POINTER(W.DWORD),W.LPVOID];K.ReadFile.restype=W.BOOL
DEFAULT_NAMES=('instance.img','storage.img','payload.idsig')

def _local_path(p):
 for q in (p,*p.parents):
  st=q.lstat()
  if getattr(st,'st_file_attributes',0)&0x400:
   raise RuntimeError('state path reparse point')

def _identity(info):
 return (info.attributes,info.volume,info.index_hi,info.index_lo,
         info.created.dwHighDateTime,info.created.dwLowDateTime,
         info.written.dwHighDateTime,info.written.dwLowDateTime,
         info.size_hi,info.size_lo,info.links)

@contextmanager
def _pinned_root(root):
 p=Path(root).absolute()
 if '..' in p.parts:raise ValueError('state root must not contain parent traversal')
 handles=[]
 try:
  # Pin ancestors in traversal order before opening their descendants. Sharing
  # read only denies directory write/reparse and rename/delete while observed.
  for q in reversed((p,*p.parents)):
   # Metadata-only access does not participate in Windows sharing checks.
   # Request directory read/list access so the original directory is pinned.
   h=K.CreateFileW(str(q),0x80000000,1,None,3,0x02200000,None)
   if h==C.c_void_p(-1).value:raise C.WinError(C.get_last_error())
   handles.append(h);info=Info()
   if not K.GetFileInformationByHandle(h,C.byref(info)):raise C.WinError(C.get_last_error())
   if not info.attributes&0x10 or info.attributes&0x400:
    raise RuntimeError('state ancestor must be a non-reparse directory')
  yield p
 finally:
  failure=None
  for h in reversed(handles):
   if not K.CloseHandle(h):failure=C.WinError(C.get_last_error())
  if failure is not None:raise failure

def snapshot(root,*,include_instance_id=False,names=None):
 if type(include_instance_id) is not bool:raise TypeError('include_instance_id must be bool')
 selected=DEFAULT_NAMES if names is None else names
 if not isinstance(selected,(tuple,list)) or not selected or any(type(n) is not str or n not in (*DEFAULT_NAMES,'instance_id') for n in selected) or len(set(selected))!=len(selected):
  raise ValueError('invalid state snapshot names')
 selected=tuple(selected)
 if include_instance_id and 'instance_id' not in selected:selected+=('instance_id',)
 with _pinned_root(root) as pinned:
  return _snapshot_files(pinned,selected)

def _snapshot_files(root,selected):
 out={}
 for name in selected:
  p=Path(root).absolute()/name;_local_path(p)
  # Reject a final-component reparse point rather than following it. Sharing zero
  # retains the existing exclusive-read contract, including delete/rename denial.
  h=K.CreateFileW(str(p),0x80000000,0,None,3,0x00200000,None)
  if h==C.c_void_p(-1).value:raise C.WinError(C.get_last_error())
  try:
   info=Info()
   if not K.GetFileInformationByHandle(h,C.byref(info)):raise C.WinError(C.get_last_error())
   size=(info.size_hi<<32)|info.size_lo
   if info.attributes&(0x10|0x400):raise RuntimeError('state must be a regular non-reparse file')
   if name=='instance_id' and (size!=64 or info.links!=1):raise RuntimeError('instance ID must be an exclusive regular64-byte file')
   digest=hashlib.sha256();buf=C.create_string_buffer(1024*1024);n=W.DWORD();total=0
   while True:
    if not K.ReadFile(h,buf,len(buf),C.byref(n),None):raise C.WinError(C.get_last_error())
    if not n.value:break
    total+=n.value;digest.update(buf.raw[:n.value])
   after=Info()
   if not K.GetFileInformationByHandle(h,C.byref(after)):raise C.WinError(C.get_last_error())
   _local_path(p)
   if _identity(info)!=_identity(after) or total!=size:raise RuntimeError('state identity changed during observation')
   out[name]={'path':str(p.resolve()),'volume':info.volume,'file_id':(info.index_hi<<32)|info.index_lo,'creation_filetime':(info.created.dwHighDateTime<<32)|info.created.dwLowDateTime,'bytes':(info.size_hi<<32)|info.size_lo,'sha256':digest.hexdigest(),'exclusive_read':True}
  finally:
   if not K.CloseHandle(h):raise C.WinError(C.get_last_error())
 return out
