from pathlib import Path
import subprocess,json,os,hashlib,sys
p=Path(__file__).resolve().parent
out=Path('E:/bscp/out/aosp17-publication-v1827/libslirp-known-source-v2/abi-tests')
assert not out.exists();out.mkdir()
build=out.parent/'build';dll=build/'subprojects/libslirp/libslirp-0.dll'
receipt=json.loads((p/'SLIRP-SUPERBUILD-BUILD.json').read_text())
assert receipt['exit']==0 and hashlib.sha256(dll.read_bytes()).hexdigest()==receipt['artifact']['sha256']
source=p/'slirp-rebuild-sources/bscp-slirp-superbuild-v1/subprojects/libslirp'
crate=Path('C:/Users/miyu1/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libslirp-sys-4.2.1/src/lib.rs')
fields='version restricted in_enabled vnetwork vnetmask vhost in6_enabled vprefix_addr6 vprefix_len vhost6 vhostname tftp_server_name tftp_path bootfile vdhcp_start vnameserver vnameserver6 vdnssearch vdomainname if_mtu if_mru disable_host_loopback enable_emu outbound_addr outbound_addr6 disable_dns'.split()
c=r'''#include <stdio.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>
#include "libslirp.h"
static int timers=0, fds=0, packets=0;
static ssize_t send_packet(const void *b,size_t n,void *o){packets++; return (ssize_t)n;}
static void guest_error(const char *s,void *o){fprintf(stderr,"guest_error:%s\n",s);exit(21);}
static int64_t clock_ns(void *o){return 0;}
static void *timer_new(SlirpTimerCb cb,void *co,void *o){void *p=calloc(1,1);if(!p)exit(22);timers++;return p;}
static void timer_free(void *p,void *o){free(p);timers--;}
static void timer_mod(void *p,int64_t when,void *o){}
static void register_fd(int fd,void *o){fds++;}
static void unregister_fd(int fd,void *o){fds--;}
static void notify(void *o){}
int main(void){
 printf("size=%zu\n",sizeof(SlirpConfig));
 /*FIELD_OFFSETS*/
 WSADATA w;if(WSAStartup(MAKEWORD(2,2),&w))return 23;
 SlirpCb cb={send_packet,guest_error,clock_ns,timer_new,timer_free,timer_mod,register_fd,unregister_fd,notify};
 SlirpConfig cfg={0};cfg.restricted=1;cfg.disable_host_loopback=true;
 cfg.in_enabled=false;cfg.in6_enabled=false;cfg.disable_dns=true;
 if(slirp_new(&cfg,&cb,NULL)!=NULL)return 24;
 for(int i=0;i<100;i++){
  cfg.version=(i%2)?1:3;Slirp *s=slirp_new(&cfg,&cb,NULL);if(!s)return 25;
  slirp_cleanup(s);if(timers||fds||packets)return 26;
 }
 printf("lifecycle=PASS version=%s loops=100 timers=%d fds=%d packets=%d\n",slirp_version_string(),timers,fds,packets);
 WSACleanup();return 0;
}
'''
c=c.replace('/*FIELD_OFFSETS*/','\n'.join('printf("'+n+'=%zu\\n",offsetof(SlirpConfig,'+n+'));' for n in fields))
(out/'probe.c').write_text(c)
rs='use libslirp_sys::SlirpConfig;\nfn main(){\nprintln!("size={}",std::mem::size_of::<SlirpConfig>());\n'+'\n'.join('println!("'+n+'={}",std::mem::offset_of!(SlirpConfig,'+n+'));' for n in fields)+'\n}\n'
(out/'layout.rs').write_text(rs)
env=dict(os.environ);env['PATH']=str(dll.parent)+';'+str(build/'subprojects/glib/subprojects/proxy-libintl')+';C:/workspace/mingw64/bin;'+env['PATH']
results=[]
def run(name,cmd):
 proc=subprocess.Popen(cmd,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 try:a,b=proc.communicate(timeout=120)
 except subprocess.TimeoutExpired:
  results.append({'name':name,'pid':proc.pid,'state':'UNKNOWN_STILL_RUNNING','signal_sent':False});(p/'SLIRP-CANDIDATE-TESTS.json').write_text(json.dumps(results,indent=2));raise
 row={'name':name,'exit':proc.returncode,'real_stdout_stderr_eof':True,'stdout':a.decode(errors='replace'),'stderr':b.decode(errors='replace')};results.append(row)
 (p/'SLIRP-CANDIDATE-TESTS.json').write_text(json.dumps(results,indent=2));assert proc.returncode==0,row
 return row['stdout']
run('C compile',['C:/workspace/mingw64/bin/gcc.exe','-std=c11','-Werror=incompatible-pointer-types','-I'+str(source/'src'),'-I'+str(dll.parent),str(out/'probe.c'),'-L'+str(dll.parent),'-lslirp','-lws2_32','-o',str(out/'probe.exe')])
rust='C:/Users/miyu1/.rustup/toolchains/1.94.0-x86_64-pc-windows-gnu/bin/rustc.exe'
run('exact cached Rust FFI compile',[rust,'--edition=2018','--crate-name','libslirp_sys','--crate-type','rlib',str(crate),'-o',str(out/'libslirp_sys.rlib')])
run('Rust ABI probe compile',[rust,'--edition=2021',str(out/'layout.rs'),'--extern','libslirp_sys='+str(out/'libslirp_sys.rlib'),'-o',str(out/'layout.exe')])
actual=run('real candidate lifecycle',[str(out/'probe.exe')]);rustout=run('actual Rust ABI layout',[str(out/'layout.exe')])
left=dict(x.split('=',1) for x in actual.splitlines() if '=' in x and not x.startswith('lifecycle='));right=dict(x.split('=',1) for x in rustout.splitlines())
assert left==right,{'C':left,'Rust':right}
final={'status':'PASS','scope':'C/Rust shared-prefix ABI + disabled-network lifecycle only; not crosvm/VM/network forwarding acceptance','candidate_sha256':receipt['artifact']['sha256'],'ffi_source_sha256':hashlib.sha256(crate.read_bytes()).hexdigest(),'layout_fields':len(fields),'operations':results,'old_runtime_unchanged':True}
(p/'SLIRP-CANDIDATE-TESTS.json').write_text(json.dumps(final,indent=2));print(json.dumps({k:v for k,v in final.items() if k!='operations'}))
