import ctypes as C,hashlib
from ctypes import wintypes as W
from pathlib import Path
K=C.WinDLL('kernel32',use_last_error=True)
K.CreateFileW.argtypes=[W.LPCWSTR,W.DWORD,W.DWORD,W.LPVOID,W.DWORD,W.DWORD,W.HANDLE];K.CreateFileW.restype=W.HANDLE
K.CloseHandle.argtypes=[W.HANDLE];K.CloseHandle.restype=W.BOOL
class Info(C.Structure):
 _fields_=[('attributes',W.DWORD),('created',W.FILETIME),('accessed',W.FILETIME),('written',W.FILETIME),('volume',W.DWORD),('size_hi',W.DWORD),('size_lo',W.DWORD),('links',W.DWORD),('index_hi',W.DWORD),('index_lo',W.DWORD)]
K.GetFileInformationByHandle.argtypes=[W.HANDLE,C.POINTER(Info)];K.GetFileInformationByHandle.restype=W.BOOL
K.ReadFile.argtypes=[W.HANDLE,W.LPVOID,W.DWORD,C.POINTER(W.DWORD),W.LPVOID];K.ReadFile.restype=W.BOOL
def snapshot(root):
 out={}
 for name in ('instance.img','storage.img','payload.idsig'):
  p=Path(root)/name;h=K.CreateFileW(str(p),0x80000000,0,None,3,0,None)
  if h==C.c_void_p(-1).value:raise C.WinError(C.get_last_error())
  try:
   info=Info()
   if not K.GetFileInformationByHandle(h,C.byref(info)):raise C.WinError(C.get_last_error())
   digest=hashlib.sha256();buf=C.create_string_buffer(1024*1024);n=W.DWORD()
   while True:
    if not K.ReadFile(h,buf,len(buf),C.byref(n),None):raise C.WinError(C.get_last_error())
    if not n.value:break
    digest.update(buf.raw[:n.value])
   out[name]={'path':str(p.resolve()),'volume':info.volume,'file_id':(info.index_hi<<32)|info.index_lo,'creation_filetime':(info.created.dwHighDateTime<<32)|info.created.dwLowDateTime,'bytes':(info.size_hi<<32)|info.size_lo,'sha256':digest.hexdigest(),'exclusive_read':True}
  finally:
   if not K.CloseHandle(h):raise C.WinError(C.get_last_error())
 return out
