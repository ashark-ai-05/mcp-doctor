# MCP Doctor Report

Trace: `.mcpdoctor/sessions/1781678247199-65789/trace.jsonl`

## Summary

- Events: 12
- RPC requests: 4
- RPC responses: 3
- Stderr lines: 2
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
### Response Some(1) (1896ms)
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "capabilities": {
      "tools": {
        "listChanged": true
      }
    },
    "protocolVersion": "2025-06-18",
    "serverInfo": {
      "name": "secure-filesystem-server",
      "version": "0.2.0"
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
### Stderr
```text
Secure MCP Filesystem Server running on stdio
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
### Response Some(2) (12ms)
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "annotations": {
          "readOnlyHint": true
        },
        "description": "Read the complete contents of a file as text. DEPRECATED: Use read_text_file instead.",
        "execution": {
          "taskSupport": "forbidden"
        },
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "properties": {
            "head": {
              "description": "If provided, returns only the first N lines of the file",
              "type": "number"
            },
            "path": {
              "type": "string"
            },
            "tail": {
              "description": "If provided, returns only the last N lines of the file",
              "type": "number"
            }
          },
          "required": [
            "path"
          ],
          "type": "object"
        },
        "name": "read_file",
        "outputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "additionalProperties": false,
          "properties": {
            "content": {
              "type": "string"
            }
          },
          "required": [
            "content"
          ],
          "type": "object"
        },
        "title": "Read File (Deprecated)"
      },
      {
        "annotations": {
          "readOnlyHint": true
        },
        "description": "Read the complete contents of a file from the file system as text. Handles various text encodings and provides detailed error messages if the file cannot be read. Use this tool when you need to examine the contents of a single file. Use the 'head' parameter to read only the first N lines of a file, or the 'tail' parameter to read only the last N lines of a file. Operates on the file as text regardless of extension. Only works within allowed directories.",
        "execution": {
          "taskSupport": "forbidden"
        },
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "properties": {
            "head": {
              "description": "If provided, returns only the first N lines of the file",
              "type": "number"
            },
            "path": {
              "type": "string"
            },
            "tail": {
              "description": "If provided, returns only the last N lines of the file",
              "type": "number"
            }
          },
          "required": [
            "path"
          ],
          "type": "object"
        },
        "name": "read_text_file",
        "outputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "additionalProperties": false,
          "properties": {
            "content": {
              "type": "string"
            }
          },
          "required": [
            "content"
          ],
          "type": "object"
        },
        "title": "Read Text File"
      },
      {
        "annotations": {
          "readOnlyHint": true
        },
        "description": "Read an image or audio file. Returns the base64 encoded data and MIME type. Only works within allowed directories.",
        "execution": {
          "taskSupport": "forbidden"
        },
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "properties": {
            "path": {
              "type": "string"
            }
          },
          "required": [
            "path"
          ],
          "type": "object"
        },
        "name": "read_media_file",
        "outputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "additionalProperties": false,
          "properties": {
            "content": {
              "items": {
                "additionalProperties": false,
                "properties": {
                  "data": {
                    "type": "string"
                  },
                  "mimeType": {
    …[truncated 15800 bytes]
```
### Stderr
```text
Client does not support MCP Roots, using allowed directories set from server args: [ '/private/tmp/mcp-doctor-real-calls/fs-root' ]
```
### Request Some(3): `tools/call`
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "arguments": {
      "path": "/private/tmp/mcp-doctor-real-calls/fs-root"
    },
    "name": "list_directory"
  }
}
```
### Response Some(3) (5ms)
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "text": "[FILE] hello.txt",
        "type": "text"
      }
    ],
    "structuredContent": {
      "content": "[FILE] hello.txt"
    }
  }
}
```
