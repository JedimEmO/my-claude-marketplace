---
name: agent-state
description: Use when the user asks about state management in agent systems, where agent state lives, prompt architecture, system prompt design, context window management, shared state between agents, agent memory, context compression, or prompt versioning.
version: 1.0.0
---

# Agent State — Context, Prompts, and Shared Memory

State management in agent systems is fundamentally different from traditional services. There is no database by default — state is spread across conversation history, system prompts, and whatever external stores you wire up. The conversation IS the agent's working memory, the system prompt IS its configuration, and the context window IS its RAM. Understanding these constraints shapes every design decision.

Most agent failures are state failures: an agent acting on stale context, a system prompt that contradicts itself, a handoff that lost critical information, or a context window that silently dropped the instructions that mattered most. Get state right and everything else gets easier.

## State Locations

Three places state can live, each with different characteristics. The art is picking the cheapest location that meets your durability and sharing requirements.

### Conversation Context (Ephemeral, Window-Bounded)

The agent's working memory — everything that has happened in this session. It grows as the conversation progresses, and it is the most natural place for state to accumulate.

- Bounded by the context window — eventually gets compressed or truncated
- Ephemeral — gone when the session ends, no persistence guarantee
- Free to write (it is just conversation), free to read (the model always sees it)
- Positional bias matters: information near the start or end of context gets more attention than information buried in the middle
- Use for: current task state, intermediate results, reasoning chains, scratchpad work
- Do not use for: anything that must survive a session boundary, anything shared with other agents

### System Prompt (Persistent Per-Session)

The agent's configuration — loaded at the start of every interaction. This is the most important piece of state in the system because it defines who the agent is and what it does.

- Persistent within a session, but static — it does not learn or change during conversation
- High-attention position — the model weights system prompt content heavily
- Competes with working memory for context budget
- Use for: identity, capabilities, constraints, behavioral rules, output format requirements, tool usage guidance
- Do not use for: dynamic state, user-specific data that changes per request, large reference material that should be fetched on demand

### External Stores (Persistent, Shared)

Files, databases, key-value stores, vector databases — anything the agent accesses via tool calls. This is the only state that persists across sessions and can be shared between agents.

- Persistent across sessions, shareable across agents
- Requires tool calls to read and write — adds latency, token spend, and failure modes
- Can grow without bound (not constrained by context window)
- Use for: accumulated knowledge, user preferences, project state, artifacts, audit logs
- Cost: every read and write is a tool call, which means tokens and latency

**Decision rule**: conversation context is free but ephemeral. System prompt is free but static. External stores are durable but expensive. Start with conversation context, promote to external stores only when you need persistence or sharing, and keep the system prompt tight and stable.

## Prompt Architecture

System prompts are contracts. Design them with the same rigor you would give an API specification. A poorly structured system prompt produces inconsistent behavior — not because the model is unreliable, but because the instructions are ambiguous.

### Structure

A well-structured system prompt follows a consistent order. The model processes it sequentially, so put the most important constraints early where they get the strongest attention.

1. **Identity** — who the agent is, one sentence. This anchors all subsequent behavior.
2. **Capabilities** — what it can do, what tools it has access to. Reference the tool list rather than duplicating tool descriptions.
3. **Constraints** — what it must NOT do. Hard boundaries. These must be unambiguous and testable.
4. **Process** — how to approach tasks. Optional for simple agents, essential for complex workflows. Step-by-step when ordering matters, principles when it does not.
5. **Output format** — what the output should look like. Be specific: JSON schema, markdown structure, required fields.
6. **Context injection point** — where dynamic context gets inserted per-session or per-task.

This ordering works because identity and constraints frame everything that follows. An agent that knows its boundaries first makes better decisions about process and output.

### Composition

System prompts are rarely monolithic. They compose from layers, and keeping layers separate matters for versioning, testing, and reuse.

- **Base prompt** — identity, core capabilities, universal constraints. Shared across all instances of this agent type. Changes infrequently.
- **Context injection** — dynamic data loaded per-session or per-task. User information, project state, relevant history. Changes every session.
- **Task-specific instructions** — what to do right now. Often comes from the delegating agent as part of the handoff, not from the system prompt itself.

Assemble them with clear delimiters. Use XML tags or markdown headers to separate sections so the model can parse them reliably:

```
<identity>You are a code review specialist...</identity>
<constraints>Never modify code directly...</constraints>
<context>The project uses Rust with a workspace layout...</context>
<task>Review the changes in the following diff...</task>
```

This structure makes it obvious what is stable configuration versus what is dynamic input. It also makes it easier to test — you can swap the context and task sections while keeping identity and constraints fixed.

### Versioning

Version prompts like APIs. When you change a system prompt, the downstream effects are just as real as changing a function signature.

- **Breaking changes** (different output format, removed capabilities, changed identity) = major version. Any agent that consumes this agent's output may break.
- **Behavioral changes** (different strategies, new constraints, reordered priorities) = minor version. Output format is stable but results may differ.
- **Clarifications and rewording** (same intent, clearer language) = patch. Should produce identical behavior.

This matters most when multiple agents depend on each other's output format. If Agent A produces structured JSON that Agent B parses, changing Agent A's output format without updating Agent B is exactly like changing an API without updating the client.

Store prompts in version control alongside the code that uses them. They are configuration, not content.

### Interface Versioning

Prompt versioning covers what an agent IS. Interface versioning covers how agents TALK to each other. Both matter.

When Agent A delegates to Agent B, the handoff schema is an interface. When you change what Agent B expects — adding a required field, changing the output format, renaming a key — you are making an interface change.

**Treat agent interfaces like API versions:**
- Adding optional fields to a handoff = backwards compatible (minor version)
- Adding required fields or changing output format = breaking change (major version)
- When the coordinator expects `{"findings": [...]}` and the specialist starts returning `{"results": [...]}`, everything downstream breaks silently

**Version the handoff schema alongside the agent's prompt.** When you version-bump an agent's output format, check every consumer of that output. This is the agent equivalent of "grep for callers before changing the function signature."

## Context Window as Working Memory

The context window is the agent's RAM. It is finite, and how you allocate it determines what the agent can accomplish in a single session.

### Budget Allocation

Think of the context window as a budget with competing demands:

- **System prompt**: 10-20% — keep it tight. Every word in the system prompt is a word the agent cannot use for reasoning.
- **Tool definitions**: 5-15% — more tools means less room for actual work. Only include tools the agent will actually use.
- **Working state**: 40-60% — the conversation, tool outputs, intermediate reasoning. This is where the agent does its job.
- **Reserve**: 15-25% — room for the next response and unexpected tool outputs. If you budget to 100%, the first large tool output will push critical context out of the window.

If your system prompt plus tool definitions exceed 35% of the window, you are constraining the agent's ability to reason about anything complex. Either trim the prompt, reduce the tool count, or accept that this agent can only handle simple tasks.

### Context Management Strategies

**Summarization checkpoints** — periodically have the agent summarize completed work and compress the conversation. The agent replaces detailed step-by-step history with a concise summary of what was done and what matters going forward. This is the agent equivalent of garbage collection.

**Structured context blocks** — use clear delimiters and structure (headers, XML tags, JSON) so the agent can efficiently scan context. Unstructured prose is harder to parse and more likely to be misinterpreted. Structure also helps when you need to reference specific context blocks later.

**Sliding window** — for long-running tasks, keep only the most recent N turns plus the system prompt. Older turns are summarized or dropped. Simple to implement, but lossy — important details from early in the conversation may be lost if the summarization is not careful.

**Selective tool output** — configure tools to return only what the agent needs, not everything available. A search tool returning 50 full documents when the agent needs 3 relevant paragraphs wastes most of the window on noise. Design tool outputs to be concise and relevant.

**Priority tagging** — mark certain context as high-priority (must retain) versus low-priority (can compress or drop). When the window fills, compress low-priority context first. This is more sophisticated but gives you explicit control over what survives.

## Shared State Patterns

When multiple agents need to share state, you need a pattern. Each has tradeoffs in complexity, consistency, and scalability.

### Artifact Store

Agents read and write named documents or artifacts. Like a shared filesystem with named keys.

- Simple mental model, easy to implement with file tools or a key-value store
- Works well for: document drafting, code generation, report building, any workflow where agents produce and refine artifacts
- Ownership model: ideally one agent writes to a given artifact, others read. If multiple agents write, use last-write-wins — conflict resolution between agents is not worth the complexity
- Challenge: no built-in notification. Agents poll or must be told when an artifact changes.

### Blackboard Pattern

A shared workspace where agents post findings. All agents can see everything on the blackboard. Each agent reads the full board, adds its contribution, and moves on.

- Good for: collaborative analysis, research tasks where findings build on each other
- Natural for convergent workflows — multiple specialists contribute to a shared understanding
- Challenge: the blackboard grows. Without a cleanup strategy, the cost of reading the full board grows unbounded and eventually dominates the context budget
- Mitigate with: periodic summarization of the blackboard by a coordinator, archiving completed topics, or partitioning the board into sections

### Event Log (Append-Only)

Agents append events to a shared log. Other agents read events they care about, filtered by type or topic. Like a commit log or message queue.

- Good for: audit trails, tracing, event-driven choreography, workflows where ordering matters
- Natural ordering, no conflict resolution needed (appending never conflicts)
- Challenge: reading relevant events from a long log requires filtering. Without indexing, agents spend tokens scanning irrelevant entries.
- Works best with: explicit event types, correlation IDs, and filtering by type or time range

### Structured Handoff State

Not persistent — passed directly between agents during delegation. Like function arguments and return values.

- Best for: coordinator-to-specialist delegation where the specialist does not need to share state with other agents
- Include: task description, relevant context (pruned to essentials), expected output format, constraints and budget
- Advantages: no shared mutable state, no consistency problems, clear ownership
- See `agent-communication` for detailed handoff patterns

**Pattern selection rule**: use structured handoffs by default. Promote to an artifact store when agents need to share persistent artifacts. Use a blackboard when agents need to see each other's work. Use an event log when you need ordering and auditability.

## Cross-Agent Consistency

When Agent A changes state that Agent B relies on, you have a consistency problem. In traditional systems this is solved with transactions and locks. In agent systems, the answer is simpler and more pragmatic.

**Design for eventual consistency and idempotent operations.**

- Agents should tolerate stale state. If Agent B reads data that Agent A is about to update, the worst case should be wasted work, not corruption. Design operations so that acting on slightly old data produces a suboptimal but not incorrect result.
- Prefer idempotent operations. If an agent retries because it did not see the result of its previous attempt, the outcome should be the same. "Set X to 5" is idempotent. "Increment X" is not.
- When strong consistency is required (rare in practice), use explicit coordination: Agent A completes and signals before Agent B starts. Sequential execution through a coordinator is the simplest form. Do not try to build distributed transactions between agents — the complexity is not worth it.
- Accept that agents will occasionally do redundant work. This is cheaper than building a coordination layer to prevent it.

**Concrete failure scenario:**

A coordinator delegates two parallel tasks: Agent A updates a project's config file, Agent B reads the config to generate documentation. B starts before A finishes, reads the old config, and generates documentation for the old settings. A completes, config is updated, but the documentation now describes the previous version.

**Why this is usually fine:** the documentation is stale but not corrupt. A human reviews and catches it, or the next run regenerates correctly. The cost of this inconsistency (one stale document) is far lower than the cost of coordinating A and B with locks (complexity, latency, deadlock risk).

**When it is NOT fine:** if Agent B's output triggers an irreversible action based on stale data — e.g., deploying with the old config because the documentation said it was current. In these cases, enforce ordering: A completes before B starts. Use the coordinator for sequencing, not locks.

## Memory and Persistence

What should survive a session versus what should be recomputed? The answer depends on acquisition cost, stability, and staleness risk.

**Persist when:**
- The information was expensive to acquire — multi-step research, user interviews, complex analysis
- The information is stable and reusable — user preferences, project conventions, architectural decisions
- Loss would degrade the user experience — accumulated context about a project, learned patterns

**Recompute when:**
- The information changes frequently — current file contents, git status, test results
- Recomputing is cheap — reading a config file, running a quick search, checking a status endpoint
- Staleness is dangerous — persisted "the tests pass" becomes a lie after code changes, cached "the API is at v2" breaks when the API upgrades

**Memory is a cache, not a source of truth.** Always verify persisted state against current reality before acting on it. The cost of a verification read is almost always less than the cost of acting on stale data. An agent that confidently acts on month-old cached state will produce confident, wrong results.

### Memory Hierarchy

Structure persistent memory in layers, from most stable to most volatile:

1. **Project knowledge** — architecture, conventions, team preferences. Changes rarely. Safe to persist long-term.
2. **Session summaries** — what was accomplished in previous sessions. Useful for continuity but verify before acting.
3. **Cached analysis** — results of expensive computations. Persist with a timestamp and invalidation strategy.
4. **Ephemeral notes** — scratchpad state for the current task. Do not persist — reconstruct from context.

## Anti-Patterns

### The God Prompt

A system prompt that tries to cover every possible scenario. 3000 words of instructions, edge cases, and conditional logic. The model cannot prioritize when everything is priority one. Keep system prompts focused. If you need conditional behavior, use context injection to load the relevant instructions for this specific task.

### Stateless Agents in Stateful Workflows

Agents that forget everything between calls, in workflows where continuity matters. Every call starts from scratch, re-reads the same files, re-discovers the same context. If a workflow has multiple steps that build on each other, persist the intermediate state explicitly — do not rely on the next agent to rediscover it.

### Unbounded Context Accumulation

Agents that never summarize, never compress, and just keep appending to context until the window fills and critical information gets silently dropped. The most dangerous form of this is when the dropped information includes constraints — the agent starts violating rules it was given early in the conversation because those rules are no longer in the active window.

### Shared Mutable State Without Ownership

Multiple agents reading and writing the same state with no coordination. This works until it does not — and when it fails, the debugging is painful because the state corruption happened turns ago with no trace. Assign clear ownership: one agent writes, others read.

### Prompt Drift

System prompts evolve informally — a tweak here, a new constraint there, a reworded section — with no versioning, no changelog, and no compatibility checks. Over weeks, the coordinator's expected output format drifts out of sync with what the specialist actually produces. The handoff schemas that worked last month silently break.

This is the agent equivalent of changing a library's API without bumping the version. The fix is the same: version prompts, version interfaces, and test compatibility when either changes. If you change a specialist's output format, check every agent that consumes it — before deploying, not after.

## Related Skills

- For how agents communicate state during delegation, see `agent-communication`
- For deciding which agents own which state, see `agent-decomposition`
- For observing state flow and debugging, see `agent-observability`
- For designing the tools agents use to access state, see `tool-design`
