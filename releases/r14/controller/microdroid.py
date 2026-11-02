#!/usr/bin/env python3
"""Explicit foreground Microdroid lifecycle; no default-profile discovery."""
import pathlib,sys
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent/'microdroid_lifecycle'))
from entry import main
if __name__=='__main__':
 try:main()
 except Exception as e:
  import json
  print(json.dumps({'status':'BLOCKED_OR_FAILED','reason':str(e),'automatic_recovery':False}),file=sys.stderr)
  sys.exit(2)
