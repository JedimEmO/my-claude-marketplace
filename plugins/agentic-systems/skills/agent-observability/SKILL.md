---
name: agent-observability
description: Use when the user asks about tracing agent decisions, debugging multi-agent flows, monitoring tool usage, error handling in agent systems, resilience patterns for agents, circuit breakers, retry strategies, cost tracking, or human-in-the-loop observability.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Agent Observability — Tracing, Resilience, and Cost

In traditional systems, you debug with stack traces and logs. In agent systems, there are no stack traces — decisions emerge from reasoning over context. Observability means capturing not just what happened, but why the agent chose it. Error handling means designing for nondeterminism: the same input can produce different outputs, tools fail in new ways, and agents can confidently produce wrong answers. This skill combines observability and error handling because in agent systems, you debug through traces and build resilience through monitoring.

## Decision Tracing

The most important thing to trace is not WHAT the agent did, but WHY.

**What to capture at each decision point:**

- What the agent saw (relevant context at decision time)
- What it considered (which tools or options were evaluated)
- What it chose (the action taken)
- What happened (the result)

**Trace structure:**

- Each agent invocation = a span
- Each tool call within an agent = a child span
- Delegation to another agent = a linked span with the same correlation ID
- The full trace tells the story: "coordinator decided to delegate research, research agent searched 3 sources, found 2 relevant results, returned a summary, coordinator used the summary to generate the final response"

**Correlation IDs** — every multi-agent task gets a single trace ID. Pass it through every delegation. Without this, you cannot reconstruct what happened across agents. This is non-negotiable. If you do nothing else for observability, do this.

**Trace storage** — traces are only useful if you can query them later. Store them in a structured format (JSON lines, a database, or an observability platform). At minimum, you need to answer: "show me everything that happened for task X" and "show me all tasks where agent Y failed in the last hour."

**Decision logging** — beyond structured traces, log the agent's reasoning in a parseable format. If the agent explains its choice before acting, capture that explanation as metadata on the span. This is the difference between "agent called grep" and "agent called grep because the user asked about error handling and the agent decided to search for try/catch patterns first."

## Multi-Agent Flow Visualization

When something goes wrong in a 4-agent flow, you need to see the whole picture.

**Span-based tracing** (borrowed from distributed systems):

```
[Coordinator]──────────────────────────────────
  ├─[Research Agent]────────────
  │   ├─ tool: web_search ──
  │   └─ tool: web_fetch ────
  ├─[Code Agent]──────────────────
  │   ├─ tool: read_file ──
  │   ├─ tool: grep ───
  │   └─ tool: edit_file ─────
  └─[Review Agent]──────
      └─ tool: read_file ──
```

Each span records: start time, end time, token count, tool calls, success/failure, and any context passed in. The visualization does not need to be fancy — even a structured log that you can grep through is better than nothing.

**What to look for in traces:**

- Long spans — an agent stuck in a reasoning loop
- Deep nesting — too many delegation levels, context is degrading at each hop
- Repeated tool calls — agent retrying the same thing expecting different results
- Context bloat — context size growing across spans as too much is passed along
- Silent failures — an agent returned a result but skipped part of the task

## Health Signals

Borrowed from container orchestration: before routing work to an agent, know whether it can handle it.

**Liveness** — is the agent responding at all? In agent systems, this means: can the model be reached, do the tools work, does the system prompt load without errors? A non-live agent should not receive work.

**Readiness** — can the agent accept new work right now? An agent might be live but not ready: its context window is near capacity from a previous task, a critical tool is in a circuit-breaker open state, or it's in the middle of a long operation. A non-ready agent should be skipped in favor of another instance or a fallback.

**Resource signals to monitor:**
- Context utilization — how full is the agent's working window? Above 70% and it's constrained.
- Error rate trending — a rising error rate means something is degrading, even if individual errors are handled.
- Latency trending — increasing response times signal a problem before failures appear.
- Tool availability — if a critical tool is down, the agent is effectively degraded even if it's "live."

These signals feed into routing decisions. A coordinator that blindly delegates to a specialist without checking health will sometimes route work into a black hole.

## Bulkhead Pattern

Isolate failures so one misbehaving workflow doesn't take down the whole system. Named after ship bulkheads that prevent one flooded compartment from sinking the ship.

**Token budget isolation** — each task type gets its own token pool. A research task running wild and consuming 10x its expected tokens should exhaust the research budget, not the budget for code review or deployment tasks.

**Model instance isolation** — if possible, route different task types to different model instances. A stuck agent consuming rate limits on one instance doesn't affect agents on another.

**Tool concurrency isolation** — cap how many concurrent calls each agent type can make to shared tools. If the research agent is hammering the web search API, it should hit its own concurrency limit before affecting the code agent's ability to use the same API.

**The principle**: a failure in one part of the system should degrade that part, not cascade. Without bulkheads, one runaway agent can exhaust shared resources (tokens, API rate limits, model capacity) and starve every other agent in the system.

## Tool Usage Monitoring

Tools are the observable actions of an agent. Monitor them.

**Metrics to track:**

- Call frequency per tool, per agent — which tools are hot?
- Success/failure rate — a tool with >10% failure rate needs attention
- Latency distribution — slow tools bottleneck the whole flow
- Token consumption — tool descriptions in context + tool output tokens
- Misuse rate — agent calling a tool that returns errors or empty results repeatedly

**Signals that something is wrong:**

- An agent calls the same tool 3+ times with similar inputs — it is not getting what it needs. The tool interface is wrong, the tool output is unhelpful, or the agent's prompt does not teach it how to use the tool effectively.
- An agent never calls a tool it has — remove it. It is burning context space for nothing.
- Tool output tokens dominate the context window — the tool is returning too much. Add filtering, pagination, or summarization to the tool output.
- Tool calls cluster at the start then stop — the agent front-loads tool use and then reasons from stale information. Consider whether it should re-check state before concluding.

## Error Handling Patterns

Agent errors are different from service errors. Services fail with exceptions. Agents fail by producing wrong outputs, making poor decisions, or getting stuck in loops. Design for these failure modes explicitly.

### Retry with Backoff

For tool failures (network errors, rate limits, transient issues). Standard pattern: retry 2-3 times with exponential backoff. The agent should understand this is automatic, not a decision point. Do not surface transient tool failures to the agent's reasoning loop — handle them in the tool layer.

### Fallback Agents

If a specialist fails, route to a more capable (but more expensive) agent. Or to a generalist that can attempt the task with less precision.

- Code review agent fails → fall back to general-purpose agent with code review instructions
- Specialist with Haiku fails → retry with Sonnet
- The key design constraint: the fallback agent must be able to pick up from where the failed agent left off. This means the failed agent's partial work must be accessible — store intermediate results, not just final output.

### Graceful Degradation

Return partial results rather than nothing. If 3 of 4 research queries succeed, report those 3 and note the failure rather than failing the entire task. The coordinator should be designed to synthesize incomplete inputs. This requires the response schema to support partial results — a list of findings with a status field per item, not a single monolithic answer.

### Circuit Breakers

If a tool or agent is consistently failing, stop calling it temporarily. Especially important for external tools (APIs, databases). Pattern:

- **Closed** — normal operation, requests flow through
- **Open** — failures exceeded threshold, all requests fail fast without attempting the call
- **Half-open** — after cooldown, allow one request through. If it succeeds, close the circuit. If it fails, reopen.

Track failure counts per tool. A circuit breaker on a tool that fails 5 times in 60 seconds saves you from burning tokens on retries that will not succeed.

### Hallucination Detection

The hardest failure mode. The agent does not know it is wrong, and it will express high confidence in incorrect answers.

**Structural checks (cheapest):**
- Schema validation — if the output doesn't match the expected structure, it's suspect. Catches a surprising number of hallucinations where the agent fabricates fields or invents formats.
- Reference verification — if the agent cites a file, read the file. If it quotes a function signature, grep for it. If it claims a URL exists, fetch it. Hallucinated references are common and cheap to catch.
- Constraint checking — verify outputs against known invariants. If the agent says "this function has no side effects" but the function writes to a database, the claim is hallucinated.

**Cross-validation (moderate cost):**
- Run the same task through two agents independently and compare. Disagreement doesn't prove either is wrong, but agreement increases confidence. This is expensive (2x token spend) so reserve it for high-stakes outputs.
- Have a validator agent check the primary agent's output against the source material. The validator doesn't redo the work — it spot-checks claims.

**Confidence calibration (least reliable):**
- Ask the agent to rate its confidence. Useful as one signal among many, but agents are poorly calibrated — they express high confidence even when wrong. Never use confidence alone as a quality gate. Use it to prioritize which outputs get deeper checks.

**When to invest in hallucination detection:** when the cost of a wrong answer exceeds the cost of checking. Customer-facing content, code that will be deployed, financial calculations, security assessments — these warrant cross-validation. Internal research notes or brainstorming — probably not.

### Loop Detection

Agents can get stuck: calling the same tool repeatedly, going back and forth between two options, or generating increasingly long responses without progress. Detect by monitoring:

- Repeated tool calls with identical or near-identical inputs (3+ times is a strong signal)
- Response length growing without new information being added
- Turn count exceeding expected range for the task complexity
- The agent apologizing or restating the problem — this is a reliable signal it is stuck

When a loop is detected, intervene: inject a prompt that breaks the pattern, escalate to a human, or terminate with a partial result. Do not let it burn tokens indefinitely.

## Resilience Patterns

### Timeout Budgets

Set a total wall-clock and token budget for a multi-agent task. Subdivide:

- Coordinator gets 20% for routing and synthesis
- Each specialist gets a proportional share of the remaining 80%
- If a specialist exceeds its budget, the coordinator must proceed without its result

Budget enforcement must be external to the agent. Agents are not good at tracking their own resource consumption. The orchestration layer should enforce hard limits. When a budget is exceeded, the coordinator should receive a structured signal — not just a timeout — so it can make an informed decision about how to proceed with reduced information.

### Dead-Letter Handling

When an agent fails terminally, what happens to its work?

- Log the partial result and the failure reason with full trace context
- Route the task to a fallback agent or back to the coordinator with the failure context attached
- Never silently drop a task — this creates invisible gaps in output that are extremely hard to debug

The dead-letter queue is your audit trail. Every failed task should be queryable: what was attempted, why it failed, what was recovered.

### Idempotent Recovery

If the system crashes mid-task, you need to resume from a checkpoint.

- Design agent tasks to be restartable: the coordinator can re-delegate to a specialist without causing duplicate side effects
- Persist checkpoints to external state (see `agent-state`) for long-running multi-agent workflows
- Use the artifact store as the source of truth — completed work is stored, incomplete work is re-attempted
- Side-effecting tools (write file, send email, deploy) need idempotency keys or pre-checks to avoid double execution

## Human-in-the-Loop as Observability

Approval checkpoints serve two purposes: trust gate AND observability window. This is your most powerful debugging tool.

**When to surface decisions to humans:**

- Before irreversible actions (deploy, send, delete, publish)
- When the agent's confidence is below a threshold
- For novel situations the agent has not encountered before
- When the cost of proceeding exceeds a threshold
- When two agents disagree on the correct approach

**What to show the human:**

- The decision the agent wants to make
- Why — the reasoning and evidence that led to this choice
- What alternatives were considered and why they were rejected
- What will happen if they approve or reject
- The current cost and time spent on this task so far

A human reviewing an agent's decision can catch errors that no automated check will find. But do not over-use this — too many approval gates and the system is no longer autonomous, it is a chatbot with extra steps. Reserve human checkpoints for high-stakes, irreversible, or low-confidence decisions.

## Cost Observability

Token spend is the cloud bill of agent systems. Track it or be surprised by it.

**Track by dimension:**

- Per agent — which agents are expensive?
- Per task type — which workflows cost the most?
- Per tool — tool outputs that consume lots of tokens are expensive inputs
- Per model tier — if using different models for different agents, track each tier separately

**Cost optimization signals:**

- A specialist using Opus for a task that Haiku could handle — right-size the model
- Tool outputs that are mostly discarded (the agent only reads 10% of what the tool returns) — add filtering to the tool
- Coordinator spending more tokens than specialists — the routing is more expensive than the work, simplify the coordinator
- Retry loops consuming budget without making progress — fix the root cause instead of retrying

**Budget alerts:**

- Set cost limits per task type based on historical averages
- Alert when a task exceeds 2x its typical cost
- Hard-stop when a task hits a maximum budget — this prevents runaway loops from draining your account
- Track cost trends over time — increasing costs for the same task type means something is degrading
- Log the model used for each agent invocation — model version changes can silently change cost profiles

**Cost attribution in multi-agent flows:**

Assign costs to the originating task, not just the agent that spent the tokens. A research agent's cost belongs to the user-facing task that triggered it. Without this attribution, you optimize individual agents but miss that certain task types are disproportionately expensive end-to-end.

## Anti-Patterns

### Observability Tax

Tracing every decision, logging every tool call in full, capturing complete context at every span. The observability system consumes 20-30% of the token budget, the traces are too verbose to read, and nobody looks at them because there's too much data.

**Right-size your observability:**
- Trace all agent invocations and tool calls (cheap — just names, timestamps, status)
- Log full context and reasoning only on errors or anomalies (expensive — do it selectively)
- Sample detailed traces in production (1 in 10 or 1 in 100) rather than tracing everything
- Set retention policies — detailed traces older than a week are rarely useful

The goal is enough observability to diagnose problems, not a complete recording of everything. If your observability costs more than 5% of your total token spend, you're over-observing.

## Related Skills

- For designing agents that are observable by default → see `agent-decomposition`
- For communication patterns that support tracing → see `agent-communication`
- For state management and checkpointing → see `agent-state`
- For building tools that produce observable, well-structured output → see `tool-design`
