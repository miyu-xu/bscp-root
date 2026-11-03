#!/usr/bin/env python3
"""Select the published Microdroid release without adopting an existing process."""
from pathlib import Path
import argparse,json,platform,runpy,sys
ROOT=Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser(add_help=False)
    p.add_argument("--release",choices=["aosp17","r14"],default="aosp17")
    p.add_argument("--release-info",action="store_true")
    a,rest=p.parse_known_args()
    host={"Windows":"windows","Linux":"linux","Darwin":"macos"}.get(platform.system())
    if host is None:raise SystemExit("Unsupported host")
    state=json.loads((ROOT/"releases"/"default.json").read_text(encoding="utf-8"))
    if a.release_info:
        print(json.dumps({"selected":a.release,"host":host,"release":state},indent=2));return
    if a.release=="aosp17":
        entry=(ROOT/"releases"/"aosp17"/"windows-v1830"/"scripts"/"microdroid.py") if host=="windows" else (ROOT/"releases"/"aosp17"/"controllers"/host/"microdroid.py")
        if not state["published"] and not any(x in ("-h","--help") for x in rest):
            raise SystemExit("AOSP17 publication is not complete; no runtime was started")
    else:entry=ROOT/"releases"/"r14"/"controller"/"microdroid.py"
    sys.argv=[str(entry),*rest]
    runpy.run_path(str(entry),run_name="__main__")
if __name__=="__main__":main()
