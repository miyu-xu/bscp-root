"""Identity of the ID allocated by the official VM service; never allocates IDs."""
import os,stat,hashlib,pathlib,importlib.util

def read_identity(state,platform):
    path=pathlib.Path(state)/'instance_id'
    for q in (path,*path.parents):
        if q.is_symlink() or (hasattr(q,'is_junction') and q.is_junction()):
            raise RuntimeError('instance ID path link/reparse point')
    if platform=='windows':
        source=pathlib.Path(__file__).parent/'vendor/windows_state.py'
        spec=importlib.util.spec_from_file_location('md_id_state',source)
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        return module.snapshot(state,names=('instance_id',))['instance_id']
    fd=os.open(path,os.O_RDONLY|os.O_CLOEXEC|os.O_NOFOLLOW)
    try:
        before=os.fstat(fd)
        if not stat.S_ISREG(before.st_mode) or before.st_size!=64 or before.st_nlink!=1:
            raise RuntimeError('instance ID must be an exclusive regular64-byte file')
        data=b''
        while len(data)<65:
            part=os.read(fd,65-len(data))
            if not part:break
            data+=part
        after=os.fstat(fd);current=path.lstat()
        fields=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns,s.st_nlink,s.st_mode)
        if len(data)!=64 or fields(before)!=fields(after) or fields(after)!=fields(current):
            raise RuntimeError('instance ID changed during observation')
        return {'dev':after.st_dev,'inode':after.st_ino,'bytes':64,'sha256':hashlib.sha256(data).hexdigest()}
    finally:os.close(fd)
