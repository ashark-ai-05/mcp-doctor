# MCP Doctor Report

Trace: `.mcpdoctor/sessions/1781676180955-65027/trace.jsonl`

## Summary

- Events: 10
- RPC requests: 4
- RPC responses: 3
- Stderr lines: 0
- Validation issues: 0
- Contract issues: 0

## Executive summary

No protocol or contract errors were detected in this trace.

## Findings

No validation issues found.

## Timeline

### Request Some(1): `initialize`
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "capabilities": {},
    "clientInfo": {
      "name": "mcp-doctor",
      "version": "0.2.1"
    },
    "protocolVersion": "2025-06-18"
  }
}
```
### Response Some(1) (85ms)
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "capabilities": {
      "tools": {}
    },
    "protocolVersion": "2025-06-18",
    "serverInfo": {
      "name": "fake-mcp",
      "version": "0.1.0"
    }
  }
}
```
### Request None: `notifications/initialized`
```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized",
  "params": {}
}
```
### Request Some(2): `tools/list`
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "method": "tools/list",
  "params": {}
}
```
### Response Some(2) (0ms)
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "description": "Echo text",
        "inputSchema": {
          "properties": {
            "text": {
              "type": "string"
            }
          },
          "required": [
            "text"
          ],
          "type": "object"
        },
        "name": "echo"
      }
    ]
  }
}
```
### Request Some(3): `tools/call`
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "arguments": {
      "text": "hello MCP"
    },
    "name": "echo"
  }
}
```
### Response Some(3) (0ms)
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "text": "hello MCP",
        "type": "text"
      }
    ]
  }
}
```
