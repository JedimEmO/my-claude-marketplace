# Tracing Patterns — Formats and Debugging Walkthroughs

## Trace Format

A trace captures the full lifecycle of a multi-agent task. Each entry is a span.

### Span Schema

```
span:
  trace_id: "task-2024-abc123"       # Shared across all agents in this task
  span_id: "coord-001"               # Unique to this span
  parent_span_id: null                # null for root, parent's ID for children
  agent: "coordinator"               # Which agent produced this span
  type: "agent_invocation"           # agent_invocation | tool_call | delegation
  start_time: "2024-01-15T10:00:00Z"
  end_time: "2024-01-15T10:00:12Z"
  tokens_in: 2400
  tokens_out: 350
  status: "success"                  # success | error | timeout | cancelled
  context_size_at_start: 3200        # Tokens of context when span began
  metadata:
    decision: "Delegating to research-agent because task requires web search"
    input_summary: "User asked for competitive analysis of 3 products"
    output_summary: "Delegated research for each product to specialist"
```

### Tool Call Span

```
span:
  trace_id: "task-2024-abc123"
  span_id: "research-tool-001"
  parent_span_id: "research-001"     # Child of the research agent span
  agent: "research-agent"
  type: "tool_call"
  tool_name: "web_search"
  tool_input:
    query: "product X market share 2024"
    max_results: 5
  tool_output_tokens: 1200
  status: "success"
  latency_ms: 2300
```

## Example: Full Trace of a Multi-Agent Task

Task: "Analyze the authentication module and suggest improvements"

```
TRACE: task-2024-auth-review
│
├─ [coordinator] 10:00:00 - 10:00:45  (tokens: 2400→350)
│  Decision: "Auth analysis needs code reading + security expertise.
│             Delegating code exploration to code-agent, security
│             review to security-agent, then synthesizing."
│
├─ [code-agent] 10:00:02 - 10:00:18  (tokens: 1800→900)
│  │  Decision: "Need to find auth module files, read implementation,
│  │            understand the flow"
│  ├─ tool: glob("**/auth/**")           2ms    → 8 files found
│  ├─ tool: read("src/auth/middleware.rs") 1ms   → 120 lines
│  ├─ tool: read("src/auth/jwt.rs")       1ms   → 85 lines
│  └─ tool: grep("session|token|cookie")  3ms   → 14 matches
│     Output: "Auth uses JWT with refresh tokens, sessions stored
│              in Redis, no CSRF protection on token endpoint"
│
├─ [security-agent] 10:00:02 - 10:00:25  (tokens: 2200→600)
│  │  Decision: "Reviewing auth patterns against OWASP checklist"
│  ├─ tool: read("src/auth/middleware.rs") 1ms   → 120 lines
│  ├─ tool: read("src/auth/jwt.rs")       1ms   → 85 lines
│  └─ tool: grep("verify|validate|check") 3ms   → 9 matches
│     Output: "3 findings: missing CSRF on /token, JWT secret
│              from env without rotation, no rate limit on /login"
│
└─ [coordinator] 10:00:26 - 10:00:45  (tokens: 3800→1200)
   Decision: "Both agents returned successfully. Synthesizing
              code understanding with security findings."
   Output: Final analysis with 3 prioritized recommendations
```

**Total: 8.2K input tokens, 3.1K output tokens, 45 seconds, 7 tool calls**

## Debugging Walkthrough: Agent Stuck in a Loop

**Symptom**: task taking 3x longer than usual, token spend climbing.

**Trace reveals**:
```
├─ [code-agent] 10:00:02 - 10:02:45  ⚠ LONG SPAN
│  ├─ tool: grep("handleAuth")          → 0 results
│  ├─ tool: grep("handle_auth")         → 0 results
│  ├─ tool: grep("authHandler")         → 0 results
│  ├─ tool: grep("auth_handler")        → 0 results
│  ├─ tool: grep("AuthHandler")         → 0 results
│  ├─ tool: glob("**/auth*handler*")    → 0 results
│  ├─ tool: grep("authenticate")        → 3 results  ← finally
│  ...
```

**Diagnosis**: the agent is searching for a function name that doesn't exist in the codebase. It's trying variations but not finding it.

**Fix options**:
1. Better handoff — the coordinator should have included the actual function/file names
2. Better tools — a "find relevant code" tool that does fuzzy matching
3. Loop detection — after 4 failed searches with similar inputs, surface to coordinator or human

## Debugging Walkthrough: Context Degradation

**Symptom**: final output is missing information that was found by a specialist.

**Trace reveals**:
```
├─ [research-agent] output: 2400 tokens
│  "Found 5 competitors. Detailed analysis of pricing,
│   features, market share for each..."
│
├─ [coordinator] receives research output
│  context_size_at_start: 3200 tokens
│  context_size_after_receiving: 5600 tokens  ← research output added
│  context_size_when_delegating_to_writer: 5600 tokens
│
├─ [writer-agent] receives: 800 tokens of context  ⚠ LOSSY
│  "Write a report about competitors. Key findings:
│   5 competitors identified."                     ← detail lost!
```

**Diagnosis**: the coordinator summarized the research output too aggressively when creating the handoff for the writer agent.

**Fix**: use structured handoffs with explicit fields (competitors list, pricing table, feature matrix) so the coordinator can't accidentally drop structured data during summarization.

## Debugging Walkthrough: Silent Tool Failure

**Symptom**: output is correct but incomplete. No errors in the trace.

**Trace reveals**:
```
├─ [data-agent]
│  ├─ tool: query_database("SELECT * FROM users WHERE active")
│  │  status: success
│  │  output: {"rows": [], "count": 0}     ← empty, not an error
│  │
│  Decision: "No active users found. Proceeding with empty dataset."
```

**Diagnosis**: the tool returned an empty result which is technically a success. The agent treated "no data" as "no active users" when the real issue was a permissions problem — the tool's database credentials didn't have access to the users table, so it returned empty rather than an error.

**Fix**: tools should distinguish between "no results" and "cannot access." The error contract should include: `{"rows": [], "count": 0, "accessible_tables": ["logs"], "requested_table": "users", "warning": "table not in accessible set"}`.

## Key Metrics Dashboard

What to track in a monitoring dashboard:

```
┌─────────────────────────────────────────────────────┐
│ AGENT SYSTEM HEALTH                                  │
├──────────────────┬──────────────────────────────────┤
│ Active tasks     │ 12                               │
│ Avg completion   │ 34s                              │
│ Error rate       │ 2.1%                             │
│ Total token/hr   │ 1.2M                             │
├──────────────────┴──────────────────────────────────┤
│ PER-AGENT BREAKDOWN                                  │
│                  calls  err%  avg_tokens  avg_time   │
│ coordinator       48    0.0%    1.2K       4.2s     │
│ research-agent    35    5.7%    3.4K      12.1s     │
│ code-agent        41    2.4%    2.1K       8.3s     │
│ review-agent      22    0.0%    1.8K       6.7s     │
├──────────────────────────────────────────────────────┤
│ TOOL HEALTH                                          │
│                  calls  err%  avg_latency  tokens    │
│ web_search        62    8.1%    2.3s        800     │
│ read_file        145    0.7%    12ms        450     │
│ grep              98    0.0%    8ms         200     │
│ edit_file         34    2.9%    15ms        300     │
├──────────────────────────────────────────────────────┤
│ ALERTS                                               │
│ ⚠ web_search error rate above 5% threshold          │
│ ⚠ research-agent avg tokens trending up (+15%/day)  │
└──────────────────────────────────────────────────────┘
```
