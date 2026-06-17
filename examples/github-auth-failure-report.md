# MCP Doctor Report

Trace: `.mcpdoctor/sessions/1781679752339-67664/trace.jsonl`

## Summary

- Events: 24
- RPC requests: 4
- RPC responses: 3
- Stderr lines: 14
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
      "version": "0.2.2"
    },
    "protocolVersion": "2025-06-18"
  }
}
```
### Response Some(1) (1767ms)
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "capabilities": {
      "tools": {}
    },
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "github-mcp-server",
      "version": "0.6.2"
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
GitHub MCP Server running on stdio
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
### Response Some(2) (8ms)
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "description": "Create or update a single file in a GitHub repository",
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "additionalProperties": false,
          "properties": {
            "branch": {
              "description": "Branch to create/update the file in",
              "type": "string"
            },
            "content": {
              "description": "Content of the file",
              "type": "string"
            },
            "message": {
              "description": "Commit message",
              "type": "string"
            },
            "owner": {
              "description": "Repository owner (username or organization)",
              "type": "string"
            },
            "path": {
              "description": "Path where to create/update the file",
              "type": "string"
            },
            "repo": {
              "description": "Repository name",
              "type": "string"
            },
            "sha": {
              "description": "SHA of the file being replaced (required when updating existing files)",
              "type": "string"
            }
          },
          "required": [
            "owner",
            "repo",
            "path",
            "content",
            "message",
            "branch"
          ],
          "type": "object"
        },
        "name": "create_or_update_file"
      },
      {
        "description": "Search for GitHub repositories",
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "additionalProperties": false,
          "properties": {
            "page": {
              "description": "Page number for pagination (default: 1)",
              "type": "number"
            },
            "perPage": {
              "description": "Number of results per page (default: 30, max: 100)",
              "type": "number"
            },
            "query": {
              "description": "Search query (see GitHub search syntax)",
              "type": "string"
            }
          },
          "required": [
            "query"
          ],
          "type": "object"
        },
        "name": "search_repositories"
      },
      {
        "description": "Create a new GitHub repository in your account",
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "additionalProperties": false,
          "properties": {
            "autoInit": {
              "description": "Initialize with README.md",
              "type": "boolean"
            },
            "description": {
              "description": "Repository description",
              "type": "string"
            },
            "name": {
              "description": "Repository name",
              "type": "string"
            },
            "private": {
              "description": "Whether the repository should be private",
              "type": "boolean"
            }
          },
          "required": [
            "name"
          ],
          "type": "object"
        },
        "name": "create_repository"
      },
      {
        "description": "Get the contents of a file or directory from a GitHub repository",
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "additionalProperties": false,
          "properties": {
            "branch": {
              "description": "Branch to get contents from",
              "type": "string"
            },
            "owner": {
              "description": "Repository owner (username or organization)",
              "type": "string"
            },
            "path": {
              "description": "Path to the file or directory",
              "type": "string"
            },
            "repo": {
              "description": "Repository name",
              "type": "string"
  …[truncated 26270 bytes]
```
### Request Some(3): `tools/call`
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "arguments": {
      "body": "This should not be created",
      "owner": "mcp-doctor-no-such-owner",
      "repo": "mcp-doctor-no-such-repo",
      "title": "mcp-doctor auth dogfood"
    },
    "name": "create_issue"
  }
}
```
### Response Some(3) (361ms)
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to create issue: Requires authentication\nStack: GitHubAuthenticationError: Requires authentication\n    at createGitHubError (file:///Users/indibc/.npm/_npx/3dfbf5a9eea4a1b3/node_modules/@modelcontextprotocol/server-github/dist/common/errors.js:55:20)\n    at githubRequest (file:///Users/indibc/.npm/_npx/3dfbf5a9eea4a1b3/node_modules/@modelcontextprotocol/server-github/dist/common/utils.js:38:15)\n    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)\n    at async file:///Users/indibc/.npm/_npx/3dfbf5a9eea4a1b3/node_modules/@modelcontextprotocol/server-github/dist/index.js:251:35"
  },
  "id": 3,
  "jsonrpc": "2.0"
}
```
### Stderr
```text
[DEBUG] Attempting to create issue in mcp-doctor-no-such-owner/mcp-doctor-no-such-repo
```
### Stderr
```text
[DEBUG] Issue options: {
```
### Stderr
```text
  "title": "mcp-doctor auth dogfood",
```
### Stderr
```text
  "body": "This should not be created"
```
### Stderr
```text
}
```
### Stderr
```text
[ERROR] Failed to create issue: GitHubAuthenticationError: Requires authentication
```
### Stderr
```text
    at createGitHubError (file:///Users/indibc/.npm/_npx/3dfbf5a9eea4a1b3/node_modules/@modelcontextprotocol/server-github/dist/common/errors.js:55:20)
```
### Stderr
```text
    at githubRequest (file:///Users/indibc/.npm/_npx/3dfbf5a9eea4a1b3/node_modules/@modelcontextprotocol/server-github/dist/common/utils.js:38:15)
```
### Stderr
```text
    at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
```
### Stderr
```text
    at async file:///Users/indibc/.npm/_npx/3dfbf5a9eea4a1b3/node_modules/@modelcontextprotocol/server-github/dist/index.js:251:35 {
```
### Stderr
```text
  status: 401,
```
### Stderr
```text
  response: { message: 'Requires authentication' }
```
### Stderr
```text
}
```
