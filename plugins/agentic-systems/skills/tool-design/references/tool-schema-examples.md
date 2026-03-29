# Tool Schema Examples

Bad vs good tool definitions showing how schema design, descriptions, and error handling affect agent behavior.

## 1. Search Tool — Description and Output Structure

### Bad

```json
{ "name": "search", "description": "Search utility.", "parameters": { "input": { "type": "string" } } }
```

The agent has no idea whether this searches names, contents, or both. `input` gives no format hint. Empty return is ambiguous — no matches or error?

### Good

```json
{
  "name": "search_files",
  "description": "Search for files by name pattern in the project. Returns paths sorted by modification time. Do not use for content search — use grep_files instead.",
  "parameters": {
    "pattern": { "type": "string", "description": "Glob pattern. Examples: '**/*.ts', 'src/**/index.js'" },
    "max_results": { "type": "integer", "default": 50 }
  },
  "output": {
    "status": { "type": "string", "enum": ["ok", "error"] },
    "files": ["string"],
    "total_matches": "integer",
    "truncated": "boolean"
  }
}
```

Clear purpose, boundary with `grep_files`, structured output with `truncated` flag. No ambiguity.

## 2. Mutation Tool — Kitchen-Sink vs Separate Actions

### Bad

```json
{
  "name": "manage_user",
  "description": "Manage users in the system.",
  "parameters": {
    "action": { "type": "string", "enum": ["create", "update", "delete", "deactivate"] },
    "user_id": { "type": "string" },
    "data": { "type": "object" }
  }
}
```

Safety is invisible — `delete` and `create` share a description. The opaque `data` object gives no schema guidance per action.

### Good

```json
{
  "name": "create_user",
  "description": "Create a new user. If email exists, returns existing user (idempotent).",
  "parameters": {
    "email": { "type": "string" },
    "display_name": { "type": "string" },
    "role": { "type": "string", "enum": ["viewer", "editor", "admin"], "default": "viewer" }
  }
}
```

```json
{
  "name": "delete_user",
  "description": "Permanently delete a user and all data. Cannot be undone. Use deactivate_user to disable without data loss.",
  "parameters": {
    "user_id": { "type": "string" },
    "confirm": { "type": "boolean", "description": "Must be true. Safety check." }
  }
}
```

Separate tools, separate safety profiles. `create_user` is idempotent, `delete_user` is destructive with a confirmation gate and a pointer to a safer alternative.

## 3. Data Retrieval — Unfiltered Dump vs Paginated and Filtered

### Bad

```json
{ "name": "get_logs", "description": "Get logs.", "parameters": { "source": { "type": "string" } } }
```

Returns the entire log as a single string — could be 5 lines or 500,000. No scope control, no filtering, no metadata.

### Good

```json
{
  "name": "get_logs",
  "description": "Retrieve log entries filtered by severity and time. Newest first. For full export use export_logs_to_file.",
  "parameters": {
    "service": { "type": "string" },
    "severity": { "type": "string", "enum": ["debug","info","warn","error","fatal"], "default": "info" },
    "since_minutes": { "type": "integer", "default": 60 },
    "max_entries": { "type": "integer", "default": 100 },
    "contains": { "type": "string", "description": "Substring filter" }
  },
  "output": {
    "status": "string",
    "entries": [{"timestamp": "...", "severity": "...", "message": "..."}],
    "total_matching": "integer",
    "returned": "integer"
  }
}
```

Agent controls scope with five parameters. `total_matching` vs `returned` tells the agent if there are more results.

## 4. Error Handling — Raw Strings vs Structured Errors

### Bad

The tool returns error information as plain strings:

```
"Error: ECONNREFUSED 10.0.0.5:5432 - connection refused"
```

Or stack traces:

```
"Traceback (most recent call last):\n  File \"db.py\", line 42...\npsycopg2.OperationalError: could not connect to server"
```

The agent cannot distinguish connection errors from permission errors from syntax errors. Stack traces waste tokens.

### Good

```json
{
  "status": "error",
  "error": {
    "code": "connection_failed",
    "message": "Cannot connect to database at 10.0.0.5:5432",
    "category": "transient",
    "recoverable": true,
    "retry_after_seconds": 5,
    "suggestion": "Database may be starting up. Retry in a few seconds."
  }
}
```

`recoverable` and `category` let the agent decide programmatically. `retry_after_seconds` gives a specific wait. `suggestion` provides a fallback strategy. No stack traces, no implementation internals.
