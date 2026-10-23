#!/usr/bin/env python3
"""lcb.py FILE.jsonl — run control requests from a JSON-lines file ({"method":..., "params":...} or {"sleep": s}).

The app's control token comes from LIGHTCRAFT_CONTROL_TOKEN_FILE (or LIGHTCRAFT_CONTROL_TOKEN): start the app
with the same `--control-token-file` (docs/control-protocol.md)."""
import json, os, socket, sys, time
def token():
    path = os.environ.get("LIGHTCRAFT_CONTROL_TOKEN_FILE")
    t = open(path).read().strip() if path else os.environ.get("LIGHTCRAFT_CONTROL_TOKEN")
    if not t: sys.exit("set LIGHTCRAFT_CONTROL_TOKEN_FILE to the app's --control-token-file")
    return t
TOKEN = token()
def call(method, params):
    s = socket.create_connection(("127.0.0.1", 7980), timeout=180)
    f = s.makefile("rwb")
    for req in ({"id": "auth", "method": "auth", "params": {"token": TOKEN}}, {"id": 1, "method": method, "params": params}):
        f.write((json.dumps(req) + "\n").encode()); f.flush()
        reply = json.loads(f.readline() or b"null")
        if req["method"] == "auth" and not (reply or {}).get("ok"): sys.exit(f"control authentication failed: {reply}")
    s.close()
    return reply
for line in open(sys.argv[1]):
    line = line.strip()
    if not line or line.startswith("#"): continue
    r = json.loads(line)
    if "sleep" in r:
        time.sleep(r["sleep"]); continue
    out = call(r["method"], r.get("params", {}))
    print(r["method"], json.dumps(out)[:160])
