---
name: agent-communication
description: Use when the user asks about how agents communicate, orchestration vs choreography, delegation patterns, agent-to-agent messaging, trust boundaries, capability gates, human-in-the-loop checkpoints, or back-pressure in multi-agent systems.
version: 1.0.0
---

# Agent Communication — Delegation, Trust, and Flow Control

Once you have decomposed work into multiple agents, communication is where the real complexity lives. Get it wrong and you end up with chatty agents burning tokens on round-trips that accomplish nothing, context degradation through long delegation chains, or security holes where agents access capabilities they were never meant to have.

The goal is always the same: get the right information to the right agent with the minimum overhead, and make sure no agent can do more than its role requires.

## Orchestration vs Choreography

These are the two fundamental coordination strategies. Most real systems use a hybrid, but understanding the pure forms matters.

### Orchestration

A central coordinator owns the workflow. It decides what happens next, delegates to specialists, collects results, and makes routing decisions.

**Strengths:**
- Easy to reason about — follow the coordinator's trace and you see the whole flow
- Clear control flow with explicit sequencing and branching
- Simple error handling — the coordinator decides what to do when a specialist fails
- Natural place to enforce budget limits and deadlines

**Weaknesses:**
- Coordinator is a bottleneck and single point of failure
- Coordinator's context grows with system complexity — it needs to understand enough to route
- Adding a new specialist means changing the coordinator

**Use when:** the workflow has clear sequential or branching logic, you need guaranteed ordering, or the system is small-to-medium (under 5 specialists).

### Choreography

Agents react to events autonomously. No central controller. Each agent knows its triggers and what to produce. Agents publish results; interested agents pick them up.

**Strengths:**
- No single bottleneck — agents operate independently
- Agents are independently deployable and replaceable
- Naturally resilient — one agent failing doesn't block others (unless they depend on its output)

**Weaknesses:**
- Hard to debug — no single place shows the full flow
- Emergent behavior can surprise you — interactions between agents create effects nobody designed
- Difficult to guarantee ordering or ensure all steps completed
- Error handling is distributed and harder to get right

**Use when:** agents are truly independent, you need high resilience, or workflows are simple event-reaction pairs with minimal coordination.

### The Hybrid Approach (Usually the Right Choice)

Use a coordinator for the happy path — the main workflow that needs to happen in order. But let specialists escalate, emit events for exceptional cases, or communicate directly when it makes sense. The coordinator owns the skeleton; agents add flesh where needed.

This gives you debuggability (follow the coordinator) with flexibility (agents can handle edge cases locally).

**Decision rule**: default to orchestration. The coordinator owns the main happy path, controls sequencing, and is your primary observability checkpoint. Add choreography at the edges — for exception handling, escalation, and cases where a specialist needs to notify others without waiting for the coordinator to route. Keep the skeleton orchestrated; add choreography as a targeted enhancement, not a starting point.

## Delegation Patterns

How one agent hands work to another. Pick the simplest pattern that works.

### Direct Invocation

Agent A calls Agent B as a subtask, like a function call. A blocks until B returns a result.

- Synchronous, tightly coupled, simple to understand
- Best for: coordinator calling a specialist when it needs the result to continue
- Risk: deep call chains lose context at each hop and are hard to debug
- Keep chains to 2 hops maximum — if you need more, your decomposition is wrong

### Message Passing

Agent A publishes a message or artifact. Agent B subscribes to a topic or queue and picks it up. A does not wait for B.

- Asynchronous, loosely coupled, naturally parallel
- Best for: event-driven flows, fan-out to multiple consumers, when A does not need B's result immediately
- Risk: harder to trace end-to-end, messages can be lost or processed out of order, eventual consistency issues
- Requires explicit correlation IDs to trace a request through the system

### Shared Workspace

Agents read from and write to a common artifact store — a file system, database, or shared context object. Collaboration happens through the artifacts, not direct communication.

- Best for: iterative refinement workflows (drafting agent writes, review agent reads and annotates), parallel work on different aspects of the same artifact
- Risk: write conflicts when multiple agents modify the same artifact, stale reads if an agent caches, no built-in ordering
- Mitigate with: clear ownership (one writer per artifact section), versioning, or optimistic locking

### Structured Handoff Objects

When delegating, pass a structured handoff — not raw conversation history. A handoff object includes:

- **Task**: what the specialist should do, stated as a clear objective
- **Context**: relevant facts the specialist needs, pruned to essentials
- **Constraints**: time budget, token budget, tool restrictions, output format
- **Expected output**: what the result should look like

**Example handoff from coordinator to a research specialist:**

```json
{
  "task": "Find security best practices for JWT token refresh",
  "context": {
    "project_language": "Rust",
    "current_approach": "Rotating refresh tokens with 24h expiry",
    "concern": "User reported tokens not refreshing in mobile app"
  },
  "constraints": {
    "max_sources": 3,
    "token_budget": 4000,
    "focus": "mobile-specific JWT issues, not general JWT tutorials"
  },
  "expected_output": {
    "format": "structured findings",
    "fields": ["source_url", "finding", "relevance_to_our_case", "recommended_action"]
  }
}
```

Notice what is NOT in the handoff: the full conversation history, unrelated project details, the coordinator's reasoning chain. The specialist gets exactly what it needs to do its job.

This is the single most important pattern for preventing context degradation. Never forward full conversation history between agents. Every handoff is an opportunity to compress, focus, and clarify.

## Conversation Threading and Context Flow

How context moves between agents determines system quality. Get this wrong and agents act on stale or distorted information.

### Full Context Forwarding

Pass everything from one agent to the next. The entire conversation, all artifacts, full history.

- Simple to implement — just pass it along
- Context grows linearly with chain length
- Quickly hits token limits
- Almost never the right choice beyond a single hop

### Summary Passing

The coordinator summarizes relevant context before delegating. Only the summary is passed to the specialist.

- Bounded context size regardless of conversation length
- Lossy — the coordinator's judgment determines what matters
- Good enough for many workflows, especially when specialists are narrowly focused
- The coordinator must be good at summarization, which is an underrated requirement

### Structured Handoff (Recommended Default)

Define an explicit schema for what gets passed between agents. Treat it like an API contract.

- Most reliable approach — forces you to think about what information actually matters
- Self-documenting — the schema tells you what each agent needs
- Testable — you can validate handoff objects independently
- Decouples agents — as long as the contract holds, implementations can change

### The Context Compression Problem

Every hop between agents loses information. This is unavoidable — the question is how much and whether you lose the right things.

A 3-agent chain where each hop retains 70% of context delivers only 34% of the original information to the final agent. A 4-agent chain: 24%. This compounds fast.

Mitigations:
- Keep chains short (2 hops max for most workflows)
- Use structured handoffs to preserve critical information explicitly
- Have the final agent access the original source when possible, not just what was passed through the chain
- If a chain must be longer, use the coordinator as a context authority that specialists can query

**Concrete example of context degradation:**

User asks: "Review the auth module for security issues, focusing on the token refresh flow and the session cleanup cron job."

- **Coordinator** (full context): passes to research agent: "Research security patterns for JWT refresh and session cleanup"
- **Research agent** (70% retained): searches for "JWT refresh security" — the session cleanup part was in the context but not in the search query
- **Research agent returns**: findings about JWT refresh only
- **Coordinator passes to reviewer**: "Review auth module with these security findings" — session cleanup concern is now gone from the working context
- **Reviewer**: produces a review covering only JWT refresh. Session cleanup is never reviewed.

The user's full request was 2 concerns. After 2 hops, only 1 survived. This is the telephone game. Fix it by including both concerns explicitly in every structured handoff, not relying on context to carry them.

## Trust Boundaries and Capability Gates

Security in agent systems is enforced at the communication layer. An agent's tool list IS its permission set — this is the simplest and most effective security model.

### Least Privilege

Every agent should have exactly the tools it needs for its role and nothing more.

- Read-only agents (researchers, analyzers) should not have Write or Bash
- Agents that modify code should not have deployment tools
- Agents that interact with external services should be isolated from internal systems
- When in doubt, start with fewer tools and add as needed — it is much easier to grant access than to revoke it after a mistake

### Separating Read and Write Agents

For high-stakes workflows, split agents by whether they read or write:

- **Analysis agents**: Read, Glob, Grep — can explore freely but cannot change anything
- **Modification agents**: Edit, Write — can change things but only when given explicit instructions from a coordinator
- **Execution agents**: Bash — isolated, monitored, with constrained command sets

This separation means a confused or misbehaving analysis agent cannot accidentally modify files, and a modification agent cannot accidentally run destructive commands.

### Human-in-the-Loop Checkpoints

Human approval is not just a safety mechanism — it is a communication checkpoint. The agent must explain what it intends to do clearly enough for a human to make a decision.

**Place checkpoints at trust boundary crossings:**
- Before external side effects (API calls, file writes, deployments)
- Before irreversible actions (deletions, sends, publishes)
- Before high-cost operations (expensive API calls, large-scale changes)
- When the agent's confidence is below a threshold

**The approval request must include:**
- What will happen (the specific action)
- Why the agent decided this (reasoning chain)
- What the alternatives were (so the human can pick a different path)
- What happens if the human says no (graceful fallback)

### Escalation Patterns

Define escalation triggers explicitly in each agent's system prompt. Do not rely on the agent to figure out when to escalate — that is an unreliable heuristic.

- **Specialist to Coordinator**: "I cannot handle this input, routing back with explanation"
- **Any Agent to Human**: "This exceeds my confidence or authority, here is what I recommend"
- **Coordinator to Fallback**: "Primary specialist failed, trying alternative approach"

Each escalation must include: what was attempted, why it failed, and what the escalating agent recommends as a next step.

## Back-Pressure and Flow Control

Agents can generate work faster than downstream agents can process it. Without flow control, you get cascading token spend and runaway costs.

### Depth Limits

Cap how many levels deep a delegation chain can go. Three is usually plenty. If you find yourself needing more, your decomposition is almost certainly wrong — you are creating agents for what should be steps within a single agent.

Pass the current depth as part of every handoff. Each agent increments it and refuses to delegate further when the limit is reached, returning its best partial result instead.

### Token and Cost Budgets

Set a total token budget for a multi-agent task. The coordinator subdivides it across specialists based on expected complexity.

- When a specialist's budget is exhausted, it must return its best partial result — not fail silently
- The coordinator tracks total spend and can abort early if costs are trending above the budget
- Log actual vs budgeted spend for every task to calibrate future budgets

### Concurrency Limits

Cap how many specialists a coordinator can run in parallel. More parallelism means more simultaneous token spend, and results often need to be reconciled — which costs tokens too.

- Start with sequential execution and parallelize only when you have evidence it helps
- Two or three parallel specialists is usually the sweet spot
- Beyond that, the coordinator's reconciliation cost starts to dominate

### Timeout Budgets

Set a wall-clock deadline for the overall task. Subdivide it across agents.

- Each agent gets a time slice proportional to its expected work
- If a specialist exceeds its slice, the coordinator can cancel it and use a fallback or return a partial result
- Always prefer a partial result over a timeout with nothing — the coordinator can decide whether partial is good enough

## Compensating Actions (Saga Pattern)

When a multi-agent workflow partially fails, you need a plan for undoing completed work. This is the saga pattern from distributed systems, adapted for agents.

**The problem**: Agent A deploys a service. Agent B updates the config. Agent C sends a notification. B fails. Now you have a deployed service with no config update and no notification. The system is in an inconsistent state.

**The solution**: for each agent action that has side effects, define a compensating action — the undo. If the workflow fails partway through, execute compensations in reverse order.

| Agent Action | Compensating Action |
|---|---|
| Deploy service | Roll back deployment |
| Update config | Restore previous config |
| Send notification | Send correction/retraction |
| Create resource | Delete resource |
| Grant access | Revoke access |

**Design rules:**

- The coordinator must track which agents completed successfully, so it knows which compensations to run
- Compensating actions must be idempotent — compensating an already-compensated action should be safe
- Not all actions have compensations. "Send email" cannot be unsent. For irreversible actions, use human-in-the-loop approval BEFORE execution, not compensation after failure
- Compensations can fail too. Log compensation failures prominently — they leave the system in an inconsistent state that requires manual intervention
- Keep workflows short. The more steps in a saga, the more likely a mid-workflow failure and the more complex the compensation chain. If a workflow has more than 4-5 compensatable steps, reconsider the design

## Anti-Patterns

### Chatty Agents

Too many round-trips between agents when one agent could have done the work. If agents are constantly asking each other for clarification, they either need better handoff objects (the sender is not providing enough context) or they should be merged into a single agent.

Symptom: agent A delegates to agent B, which asks A a question, which A answers, which B uses to ask another question. This is a conversation, not a delegation.

### The Telephone Game

Context degrades through long chains. Agent C acts on a distorted version of what Agent A intended because B's summary was lossy. The longer the chain, the worse the distortion.

Fix: keep chains to 2 hops max. If the final agent needs original context, let it access the source directly rather than relying on intermediaries.

### Over-Delegation

A coordinator that does nothing itself — just farms everything out to specialists and stitches results together. If the coordinator adds no judgment, no routing logic, and no synthesis beyond concatenation, it should not exist.

The coordinator should own: routing decisions, context management, quality assessment of specialist outputs, and final synthesis. These are real jobs.

### Premature Choreography

Building an event-driven agent mesh when a simple coordinator would do. Choreography is powerful but hard to debug, hard to reason about, and hard to test. You earn choreography with proven complexity, not because it sounds architecturally elegant.

Start with orchestration. Move to choreography only when the coordinator becomes a genuine bottleneck or when agents truly need to operate independently.

## Related Skills

- For deciding which agents to create and how to scope them, see `agent-decomposition`
- For designing the tool interfaces agents use, see `tool-design`
- For tracing and debugging multi-agent flows, see `agent-observability`
- For managing state across agent boundaries, see `agent-state`
