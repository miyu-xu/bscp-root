from pathlib import Path
import ast,hashlib,json,subprocess,sys,xml.etree.ElementTree as ET
root=Path(__file__).resolve().parents[2]
assert (root/"docs/README.md").is_file() and (root/"docs/README.zh-CN.md").is_file()
assert not (root/"doc").exists(), "docs is the sole first-party documentation directory"
index=json.loads((root/"releases/aosp17/CONTROLLER-AND-DOC-SOURCE.json").read_bytes())
for name,row in index["files"].items():
    path=root/name
    assert path.resolve().is_relative_to(root.resolve()) and not path.is_symlink()
    data=path.read_bytes()
    assert len(data)==row["bytes"] and hashlib.sha256(data).hexdigest()==row["sha256"],name
for path in (root/"releases").rglob("*.py"):
    ast.parse(path.read_text(encoding="utf-8-sig"),filename=str(path))
old=json.loads((root/"releases/r14/SOURCE.json").read_bytes())
for name,h in old["files"].items():
    assert hashlib.sha256((root/"releases/r14/controller"/name).read_bytes()).hexdigest()==h
m=ET.parse(root/"docs/aosp17/frozen-delivery/android17-locked.xml")
projects=m.findall("project")
assert len(projects)==1087 and len({p.get("path") for p in projects})==1087
assert all(len(p.get("revision",""))==40 and all(c in "0123456789abcdef" for c in p.get("revision","")) for p in projects)
for args in (["--release-info"],["--help"],["--release","r14","--help"]):
    subprocess.run([sys.executable,"-B",str(root/"scripts/microdroid.py"),*args],check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
print(json.dumps({"frozen_files":len(index["files"]),"r14_files":len(old["files"]),"pinned_projects":1087,"entry_help":"PASS","VM_started":False,"runtime_acceptance":"not repeated by CI"}))
