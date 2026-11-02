// Desktop authority for the inherited, single-client RPC bootstrap.
// This is local same-user authorization, not Android PermissionManager or SELinux.
// The retained endpoint authenticates its creator, not arbitrary Binder object delegates.
use std::io::{self, Error, ErrorKind};
use std::os::fd::{AsRawFd, BorrowedFd, OwnedFd};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Peer { pub pid: libc::pid_t, pub uid: libc::uid_t, pub gid: libc::gid_t }

fn denied(message: &str) -> Error { Error::new(ErrorKind::PermissionDenied, message) }
fn getsock<T: Copy>(fd: BorrowedFd<'_>, level: i32, option: i32) -> io::Result<T> {
    let mut value = std::mem::MaybeUninit::<T>::uninit();
    let mut size = std::mem::size_of::<T>() as libc::socklen_t;
    // Kernel writes at most size bytes; require the complete typed result.
    if unsafe { libc::getsockopt(fd.as_raw_fd(), level, option, value.as_mut_ptr().cast(), &mut size) } != 0 {
        return Err(Error::last_os_error());
    }
    if size as usize != std::mem::size_of::<T>() { return Err(denied("invalid socket credential size")); }
    Ok(unsafe { value.assume_init() })
}

fn peer(fd: BorrowedFd<'_>) -> io::Result<Peer> {
    if getsock::<i32>(fd, libc::SOL_SOCKET, libc::SO_TYPE)? != libc::SOCK_STREAM {
        return Err(denied("RPC bootstrap is not a stream socket"));
    }
    let mut addr = std::mem::MaybeUninit::<libc::sockaddr_storage>::zeroed();
    let mut len = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    if unsafe { libc::getpeername(fd.as_raw_fd(), addr.as_mut_ptr().cast(), &mut len) } != 0 {
        return Err(Error::last_os_error());
    }
    if len < std::mem::size_of::<libc::sa_family_t>() as _ || unsafe { addr.assume_init() }.ss_family as i32 != libc::AF_UNIX {
        return Err(denied("RPC bootstrap must be a connected Unix socket"));
    }
    #[cfg(target_os="macos")]
    {
        let (mut uid, mut gid) = (0, 0);
        if unsafe { libc::getpeereid(fd.as_raw_fd(), &mut uid, &mut gid) } != 0 { return Err(Error::last_os_error()); }
        let pid = getsock::<libc::pid_t>(fd, 0, libc::LOCAL_PEERPID)?;
        Ok(Peer { pid, uid, gid })
    }
    #[cfg(target_os="linux")]
    {
        let cred = getsock::<libc::ucred>(fd, libc::SOL_SOCKET, libc::SO_PEERCRED)?;
        Ok(Peer { pid: cred.pid, uid: cred.uid, gid: cred.gid })
    }
}

#[derive(Debug)]
pub struct Authority { endpoint: Option<OwnedFd>, peer: Peer, rpc_required: bool }
static BOOTSTRAP: std::sync::OnceLock<Authority> = std::sync::OnceLock::new();
pub fn initialize(fd: BorrowedFd<'_>, creator: libc::pid_t) -> io::Result<()> {
    let owner=Authority::bind(fd,creator)?;
    BOOTSTRAP.set(owner).map_err(|_|denied("desktop authority already initialized"))
}
/// Listener authority names only the owning user; every external operation
/// additionally requires a real authenticated RPC dispatch identity.
pub fn initialize_listener() -> io::Result<()> {
    let uid=unsafe{libc::getuid()};let gid=unsafe{libc::getgid()};
    if uid==0 || uid!=unsafe{libc::geteuid()} || gid!=unsafe{libc::getegid()}{return Err(denied("invalid desktop listener owner"));}
    BOOTSTRAP.set(Authority{endpoint:None,peer:Peer{pid:-1,uid,gid},rpc_required:true}).map_err(|_|denied("desktop authority already initialized"))
}
pub fn current() -> io::Result<&'static Authority> {
    BOOTSTRAP.get().ok_or_else(||denied("desktop bootstrap is not authenticated"))
}
impl Authority {
    pub fn bind(fd: BorrowedFd<'_>, expected_creator: libc::pid_t) -> io::Result<Self> {
        let endpoint = fd.try_clone_to_owned()?;
        let actual = peer(std::os::fd::AsFd::as_fd(&endpoint))?;
        let uid = unsafe { libc::getuid() };
        if uid != unsafe { libc::geteuid() } || unsafe { libc::getgid() } != unsafe { libc::getegid() } {
            return Err(denied("privilege-changing desktop launch is unsupported"));
        }
        if uid == 0 || expected_creator <= 0 || actual.pid != expected_creator || actual.uid != uid || actual.gid != unsafe {libc::getgid()} {
            return Err(denied("bootstrap peer does not match the invoking user/process"));
        }
        Ok(Self { endpoint: Some(endpoint), peer: actual, rpc_required: false })
    }
    pub fn identity(&self) -> Peer { self.peer }
    pub fn request_identity(&self) -> io::Result<Peer> {
        if !self.rpc_required { return Ok(self.peer); }
        #[cfg(target_os="macos")]
        {
            #[link(name="binder_sdk")]
            extern "C" { fn BSCP_RpcCallingAuditToken(token:*mut u32,count:usize)->i32; }
            let mut token=[0u32;8];let status=unsafe{BSCP_RpcCallingAuditToken(token.as_mut_ptr(),token.len())};
            if status!=0{return Err(denied("authenticated RPC caller context is absent"));}
            let actual=Peer{pid:token[5] as libc::pid_t,uid:token[1],gid:token[2]};
            if actual.pid<=0 || actual.uid!=self.peer.uid || actual.gid!=self.peer.gid || token[3]!=self.peer.uid || token[4]!=self.peer.gid{return Err(denied("RPC caller is not the instance owner"));}
            Ok(actual)
        }
        #[cfg(target_os="linux")]
        {
            #[link(name="binder_sdk")]
            extern "C" { fn BSCP_RpcCallingLinuxCredentials(pid:*mut libc::pid_t,uid:*mut libc::uid_t,gid:*mut libc::gid_t)->i32; }
            let mut actual=Peer{pid:-1,uid:0,gid:0};
            let status=unsafe{BSCP_RpcCallingLinuxCredentials(&mut actual.pid,&mut actual.uid,&mut actual.gid)};
            if status!=0 || actual.pid<=0 || actual.uid!=self.peer.uid || actual.gid!=self.peer.gid {
                return Err(denied("live authenticated RPC caller context is absent or foreign"));
            }
            Ok(actual)
        }
        #[cfg(not(any(target_os="macos",target_os="linux")))]
        {Err(denied("authenticated desktop listener unsupported on this platform"))}
    }

    pub fn check(&self, permission: &str) -> io::Result<()> {
        // Keep a descriptor reference so closing/reusing the original numeric FD cannot replace it.
        let _caller=self.request_identity()?;
        if let Some(endpoint)=self.endpoint.as_ref(){
            if peer(std::os::fd::AsFd::as_fd(endpoint))? != self.peer{return Err(denied("desktop bootstrap identity changed"));}
        }
        if unsafe { libc::geteuid() } != self.peer.uid || unsafe { libc::getuid() } != self.peer.uid || unsafe {libc::getgid()} != self.peer.gid || unsafe {libc::getegid()} != self.peer.gid {
            return Err(denied("desktop authority changed"));
        }
        match permission {
            "android.permission.DUMP" | "android.permission.MANAGE_VIRTUAL_MACHINE" |
            "android.permission.USE_CUSTOM_VIRTUAL_MACHINE" => Ok(()),
            _ => Err(denied("unsupported desktop permission")),
        }
    }
    pub fn check_image(&self, fd: BorrowedFd<'_>) -> io::Result<()> {
        self.check("android.permission.MANAGE_VIRTUAL_MACHINE")?;
        check_image(fd, self.peer.uid)
    }
}

/// Actual opened image descriptor checks. No path lookup or synthetic SELinux label.
/// This does not make mutable same-user input immutable; AVB/APK verification remains required.
pub fn check_image(fd: BorrowedFd<'_>, uid: libc::uid_t) -> io::Result<()> {
    let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
    if unsafe { libc::fstat(fd.as_raw_fd(), st.as_mut_ptr()) } != 0 { return Err(Error::last_os_error()); }
    let st = unsafe { st.assume_init() };
    if st.st_mode & libc::S_IFMT != libc::S_IFREG { return Err(denied("desktop image must be a regular file")); }
    if st.st_uid != uid && st.st_uid != 0 { return Err(denied("image owned by another user")); }
    if st.st_mode & 0o022 != 0 { return Err(denied("image writable by group or other users")); }
    let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
    if flags == -1 { return Err(Error::last_os_error()); }
    #[cfg(target_os="linux")]
    if flags & libc::O_PATH != 0 { return Err(denied("O_PATH is not a readable image descriptor")); }
    if flags & libc::O_ACCMODE == libc::O_WRONLY { return Err(denied("image descriptor is not readable")); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::AsFd;
    use std::os::unix::net::UnixStream;
    use std::os::unix::fs::PermissionsExt;
    #[test] fn inherited_bootstrap_child() {
        use std::os::fd::FromRawFd;
        let Ok(raw)=std::env::var("TEST_BOOTSTRAP_FD") else {return;};
        let fd=unsafe {OwnedFd::from_raw_fd(raw.parse().unwrap())};
        let expected=std::env::var("TEST_CREATOR_PID").unwrap().parse().unwrap();
        let owner=Authority::bind(fd.as_fd(),expected).unwrap();
        owner.check("android.permission.MANAGE_VIRTUAL_MACHINE").unwrap();
        println!("verified inherited creator {} uid {}",owner.identity().pid,owner.identity().uid);
    }
    #[test] fn actual_spawn_exec_bootstrap() {
        use std::os::unix::process::CommandExt;
        let (a,_b)=UnixStream::pair().unwrap();let fd=a.as_raw_fd();
        let mut command=std::process::Command::new(std::env::current_exe().unwrap());
        let child_test=format!("{}::inherited_bootstrap_child",module_path!().split_once("::").unwrap().1);
        command.args(["--exact",child_test.as_str(),"--nocapture"])
            .env("TEST_BOOTSTRAP_FD",fd.to_string()).env("TEST_CREATOR_PID",std::process::id().to_string());
        unsafe {command.pre_exec(move || {
            let flags=libc::fcntl(fd,libc::F_GETFD);
            if flags<0 || libc::fcntl(fd,libc::F_SETFD,flags & !libc::FD_CLOEXEC)<0{return Err(Error::last_os_error());} Ok(())
        });}
        let result=command.output().unwrap();
        assert!(result.status.success(),"{} {}",String::from_utf8_lossy(&result.stdout),String::from_utf8_lossy(&result.stderr));
        assert!(String::from_utf8_lossy(&result.stdout).contains("verified inherited creator"));
    }
    #[test] fn real_peer_and_known_permission() {
        let (a, _b) = UnixStream::pair().unwrap();
        let owner = Authority::bind(a.as_fd(), unsafe { libc::getpid() }).unwrap();
        owner.check("android.permission.MANAGE_VIRTUAL_MACHINE").unwrap();
        assert!(owner.check("android.permission.UNKNOWN").is_err());
        assert!(owner.check("android.permission.USE_RELAXED_MICRODROID_ROLLBACK_PROTECTION").is_err());
    }
    #[test] fn wrong_creator_rejected() {
        let (a, _b) = UnixStream::pair().unwrap();
        assert!(Authority::bind(a.as_fd(), -1).is_err());
        assert!(Authority::bind(a.as_fd(), unsafe {libc::getpid()}+1).is_err());
    }
    #[test] fn original_descriptor_close_does_not_replace_authority() {
        let (a, _b) = UnixStream::pair().unwrap();
        let owner = Authority::bind(a.as_fd(), unsafe { libc::getpid() }).unwrap();
        drop(a);
        let unrelated = std::fs::File::open("/dev/null").unwrap();
        owner.check("android.permission.DUMP").unwrap();
        assert!(Authority::bind(unrelated.as_fd(), unsafe { libc::getpid() }).is_err());
    }
    #[test] fn tcp_bootstrap_rejected() {
        let l=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let c=std::net::TcpStream::connect(l.local_addr().unwrap()).unwrap();
        let (_s,_)=l.accept().unwrap();
        assert!(Authority::bind(c.as_fd(), unsafe { libc::getpid() }).is_err());
    }
    #[test] fn real_file_modes_and_type() {
        let p=std::env::temp_dir().join(format!("desktop-authority-{}",std::process::id()));
        let f=std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(&p).unwrap();
        f.set_permissions(std::fs::Permissions::from_mode(0o600)).unwrap();
        check_image(f.as_fd(),unsafe {libc::getuid()}).unwrap();
        f.set_permissions(std::fs::Permissions::from_mode(0o620)).unwrap();
        assert!(check_image(f.as_fd(),unsafe {libc::getuid()}).is_err());
        assert!(check_image(std::fs::File::open("/dev/null").unwrap().as_fd(),unsafe {libc::getuid()}).is_err());
        // Retain test artifacts; no deletion of old/user files.
    }
}

/// Obtain the path of this actual open file, and reject stale path replacement.
/// The path is used only by the existing partition check, never to reopen data.
#[cfg(target_os="macos")]
pub fn image_descriptor_path(fd:BorrowedFd<'_>)->io::Result<std::path::PathBuf>{
    use std::os::unix::{ffi::OsStrExt,fs::MetadataExt};
    let mut buf=[0u8;libc::PATH_MAX as usize];
    if unsafe{libc::fcntl(fd.as_raw_fd(),libc::F_GETPATH,buf.as_mut_ptr())}<0{return Err(Error::last_os_error());}
    let end=buf.iter().position(|b|*b==0).ok_or_else(||denied("unterminated descriptor path"))?;
    let path=std::path::PathBuf::from(std::ffi::OsStr::from_bytes(&buf[..end]));
    if !path.is_absolute(){return Err(denied("descriptor path is not absolute"));}
    let mut st=std::mem::MaybeUninit::<libc::stat>::uninit();
    if unsafe{libc::fstat(fd.as_raw_fd(),st.as_mut_ptr())}!=0{return Err(Error::last_os_error());}
    let st=unsafe{st.assume_init()};let metadata=std::fs::metadata(&path)?;
    if metadata.dev()!=st.st_dev as u64 || metadata.ino()!=st.st_ino as u64 {
        return Err(denied("descriptor pathname identity changed"));
    }
    Ok(path)
}
#[cfg(all(test,target_os="macos"))]
mod descriptor_path_tests {
 use super::*;use std::os::fd::AsFd;
 #[test] fn actual_file_and_rename_keep_original_identity(){
  let dir=tempfile::tempdir().unwrap();let original=dir.path().join("one");let renamed=dir.path().join("two");
  let file=std::fs::File::create(&original).unwrap();
  assert_eq!(image_descriptor_path(file.as_fd()).unwrap(),original.canonicalize().unwrap());
  std::fs::rename(&original,&renamed).unwrap();std::fs::write(&original,b"replacement").unwrap();
  assert_eq!(image_descriptor_path(file.as_fd()).unwrap(),renamed.canonicalize().unwrap());
 }
 #[test] fn socket_is_not_a_file_path(){let (a,_b)=std::os::unix::net::UnixStream::pair().unwrap();assert!(image_descriptor_path(a.as_fd()).is_err());}
}
