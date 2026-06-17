#!/usr/bin/env python3
import argparse
import json
import sys

parser = argparse.ArgumentParser()
parser.add_argument("--mode", choices=["ok", "invalid-json", "schema-change", "missing-tool", "stderr-auth"], default="ok")
args = parser.parse_args()

if args.mode == "stderr-auth":
    print("missing env var GITHUB_TOKEN", file=sys.stderr, flush=True)

def write(obj):
    sys.stdout.write(json.dumps(obj, separators=(",", ":")) + "\n")
    sys.stdout.flush()

for raw in sys.stdin:
    raw = raw.strip()
    if not raw:
        continue
    try:
        msg = json.loads(raw)
    except Exception:
        continue
    method = msg.get("method")
    msg_id = msg.get("id")
    if msg_id is None:
        continue
    if args.mode == "invalid-json" and method == "initialize":
        sys.stdout.write("this is not json\n")
        sys.stdout.flush()
        continue
    if method == "initialize":
        write({
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "fake-mcp", "version": "0.1.0"},
            },
        })
    elif method == "tools/list":
        tools = [] if args.mode == "missing-tool" else [{
            "name": "echo",
            "description": "Echo text",
            "inputSchema": {
                "type": "object",
                "properties": {"text": {"type": "string"}},
                "required": ["text"],
            },
        }]
        write({"jsonrpc": "2.0", "id": msg_id, "result": {"tools": tools}})
    elif method == "tools/call":
        params = msg.get("params", {})
        name = params.get("name")
        arguments = params.get("arguments", {})
        if name != "echo" or args.mode == "missing-tool":
            write({"jsonrpc": "2.0", "id": msg_id, "error": {"code": -32601, "message": "tool not found"}})
        else:
            text = arguments.get("text", "")
            if args.mode == "schema-change":
                result = {"content": [{"type": "text", "text": f"changed:{text}"}]}
            else:
                result = {"content": [{"type": "text", "text": text}]}
            write({"jsonrpc": "2.0", "id": msg_id, "result": result})
    else:
        write({"jsonrpc": "2.0", "id": msg_id, "error": {"code": -32601, "message": f"unknown method {method}"}})
