---
name: agent-decomposition
description: Use when the user asks about splitting a system into agents, agent boundaries, how many agents to use, agent responsibility assignment, capability allocation, or agent topology design. Also triggers on "should this be one agent or multiple", "agent architecture", or "agent roles".
version: 1.0.0
---

# Agent Decomposition — Boundaries, Roles, and Topologies

This is the most consequential architectural decision you will make in an agentic system. Every other choice — communication patterns, state management, tool design — flows downstream from how you draw agent boundaries. Get it wrong in one direction and you have a god-agent drowning in a 200k-token context window, unable to focus. Get it wrong in the other direction and you have six agents burning tokens on coordination overhead, passing messages back and forth to accomplish what one agent could have done in a single turn. The default posture is restraint: start with one agent and split only when you have evidence.

## The Decomposition Decision

**The default is one agent with tools.** A single agent with a focused system prompt and a well-chosen tool set handles the vast majority of tasks. Multi-agent is not an upgrade — it is a trade-off. You are exchanging simplicity and shared context for isolation and specialization.

Only decompose when at least one of these concrete pressures exists:

| Pressure | Signal | Example |
|----------|--------|---------|
| **Context window saturation** | Agent performance degrades as conversation grows; it forgets earlier instructions or loses track of state | A coding agent working across a 500-file monorepo that needs domain docs, API specs, and test fixtures simultaneously |
| **Role specialization** | The system prompt tries to be two things at once and does both poorly | "You are an expert code reviewer AND a creative copywriter" — these require fundamentally different personalities |
| **Trust boundaries** | Different tasks need different permission levels | One task needs filesystem write access; another should only read from a web API |
| **Model cost differentiation** | Some subtasks are simple extraction; others need deep reasoning | Use Opus for architectural decisions, Haiku for parsing log files |
| **Independent scaling** | One capability is called 100x more than others | A data-extraction pipeline that fans out to dozens of parallel workers |

**Priority order when multiple pressures exist:** context window saturation is the strongest signal — if an agent is hitting context limits, split immediately. Role specialization is next — contradictory system prompts degrade everything. Trust boundaries come third. Cost differentiation and independent scaling are weaker signals that rarely justify splitting on their own.

**Decision checklist before splitting:**

1. Have you actually hit context limits, or are you anticipating them?
2. Would a better system prompt or tool design solve the problem without splitting?
3. Can you quantify the coordination cost of the split?
4. Will each resulting agent have enough context to do its job independently?
5. Is there a simpler solution — like clearing context mid-conversation — that avoids multi-agent entirely?

If you answered "no" to question 1 or "yes" to question 2, stop. You do not need multiple agents yet.

## Agent Responsibility Patterns

These are the recurring roles that emerge in well-designed multi-agent systems. Not every system needs all of them — most need two or three.

### Coordinator

Routes work, does not do it. Holds the plan and delegates to specialists. The coordinator's system prompt is about task decomposition and routing logic, not domain expertise. It should be thin by design — if your coordinator's system prompt is longer than any specialist's, something is wrong.

A coordinator decides: "This looks like a database migration task, sending to the data-specialist" or "The user wants a code review followed by documentation updates — I will sequence specialist-code-review then specialist-docs."

### Specialist

Deep domain expertise, narrow tool set. Does one thing well. A specialist agent might be "the database agent" with access to query tools, schema introspection, and migration utilities — and nothing else. Its system prompt is dense with domain knowledge because it does not waste context on capabilities it will never use.

### Transformer

Reshapes data between systems or formats. No domain logic, pure translation. When agent A produces output in format X and agent B needs format Y, a transformer sits between them. This is often a function rather than an agent — only promote it to an agent when the transformation requires LLM reasoning (e.g., summarizing a 50-page document into a structured brief).

### Validator

Checks output quality and enforces constraints. A second pair of eyes. Validators are especially valuable when the cost of errors is high — generating SQL that will run against production, producing customer-facing content, or making irreversible API calls. The validator does not produce; it critiques.

### Aggregator

Collects results from multiple agents and synthesizes a unified response. In a map-reduce topology, the aggregator is the reduce step. It resolves conflicts between specialist outputs, merges partial results, and presents a coherent answer to the user.

### Anti-pattern: The God Agent

One agent with 30 tools, a 4000-word system prompt, and instructions that try to cover every possible scenario. You will recognize it by its symptoms: inconsistent behavior depending on which part of the system prompt the model attends to, tools that are never called, and performance that degrades as you add more capabilities. If you have a god agent, the fix is not "add more instructions" — it is decomposition.

**But note**: a single agent with many capabilities is often the RIGHT starting point. The god agent is an anti-pattern only when you have evidence of the symptoms above. A capable agent with 15 tools and a well-structured prompt that performs well is not a god agent — it is a well-designed single agent. Do not split preemptively to avoid a label.

### Anti-pattern: Atomic Agent Syndrome

The opposite extreme — one agent per tool, each maximally specialized. A `file_reader_agent`, a `grep_agent`, a `file_writer_agent`. It sounds clean in theory. In practice, coordination overhead dominates: the coordinator spends more tokens routing between 12 micro-agents than a single agent would spend doing the work directly. Every delegation is a context hop, and every hop loses information. If an agent has only one tool and no domain knowledge in its system prompt, it should not be an agent — it should just be a tool.

## Capability Allocation

Which tools belong to which agent. The core principle: **an agent should only have tools it has the context to use well.**

**Group tools by domain.** All database tools go with the data agent. All API integration tools go with the integration agent. All filesystem tools go with the coding agent. When a tool does not clearly belong to one domain, that is a signal your domain boundaries need refinement.

**Remove unused tools.** If an agent has access to a tool it never uses in practice, remove it. Every tool description consumes context tokens and adds cognitive load to the model's tool-selection reasoning. Audit tool usage periodically.

**Tool count as a code smell.** If an agent has more than 10 tools, question whether it is really one agent or two agents crammed together. The sweet spot for most specialists is 3-7 tools.

**Read vs. write asymmetry.** Read-only tools (search, query, inspect) can be shared more freely across agents because they cannot cause damage. Write tools (create, update, delete, execute) should be allocated carefully and usually belong to exactly one agent. If two agents both need to write to the same resource, you either have a boundary problem or you need a mediator.

## Topology Patterns

### Hub-and-Spoke (Coordinator to Specialists)

The most common and most recommended starting topology.

```
           ┌──────────┐
           │Coordinator│
           └─────┬─────┘
        ┌────────┼────────┐
        ▼        ▼        ▼
   ┌────────┐ ┌──────┐ ┌──────┐
   │Code    │ │Data  │ │Docs  │
   │Agent   │ │Agent │ │Agent │
   └────────┘ └──────┘ └──────┘
```

One coordinator fans out to specialists. Simple to reason about, single point of coordination, easy to debug because all routing decisions are visible in one place. Start here.

### Pipeline (Sequential Handoff)

```
   ┌──────────┐    ┌──────────┐    ┌──────────┐
   │ Extract  │───▶│Transform │───▶│ Validate │
   └──────────┘    └──────────┘    └──────────┘
```

Each stage transforms or enriches the output of the previous stage. Good for ETL-like workflows, content pipelines (draft → review → polish), or any process with clear sequential phases. The key constraint: each agent must be able to do its job with only the output of the previous stage — no reaching back two steps.

### Map-Reduce (Fan-out / Fan-in)

```
                ┌──────────┐
                │Coordinator│
                └─────┬─────┘
           ┌──────┬───┼───┬──────┐
           ▼      ▼   ▼   ▼      ▼
         ┌───┐ ┌───┐┌───┐┌───┐ ┌───┐
         │ W1│ │ W2││ W3││ W4│ │ W5│
         └─┬─┘ └─┬─┘└─┬─┘└─┬─┘ └─┬─┘
           └──────┴───┬┴────┴──────┘
                ┌─────▼─────┐
                │ Aggregator│
                └───────────┘
```

Coordinator splits work into N parallel tasks, workers execute independently, aggregator combines results. Good for parallelizable work like searching multiple sources, processing batches, or evaluating multiple options. The workers must be truly independent — if worker 3 needs results from worker 1, this is not a map-reduce problem.

### Peer Network

Agents communicate directly without a central coordinator. Each agent decides when to invoke another. This is harder to debug and reason about, but avoids the coordinator bottleneck. Use only when agents are truly autonomous and the interaction patterns are unpredictable. In practice, most systems that think they need peer networks actually work better with hub-and-spoke.

### The Two-Level Sweet Spot

In practice, most systems work best with at most two levels of hierarchy: a coordinator and its specialists. Deeper hierarchies — a coordinator that delegates to sub-coordinators that delegate to specialists — add latency, lose context at each handoff, and make debugging painful. If you find yourself building a three-level hierarchy, reconsider your decomposition. You may be over-splitting, or you may need to restructure as two independent two-level systems rather than one deep tree.

## Boundary Heuristics

Concrete rules for drawing the line.

**Split when:**

- Context window would exceed roughly 60% capacity in normal operation, leaving room for the conversation itself to grow
- Tasks require different model tiers and the cost difference is material at your scale
- Trust requirements genuinely differ — one task needs filesystem access, another should be sandboxed to web search only
- Failure of one task should not poison another's context — a failed code generation attempt filling context with error traces should not degrade the research agent's performance
- Domain expertise is genuinely disjoint — a coding agent and a market-research agent share almost no system prompt content

**Do not split when:**

- The agents would just pass data through without adding value — if agent B needs everything agent A knows, they should be one agent
- Coordination overhead (routing logic, message formatting, context summarization) exceeds the context savings from splitting
- The "specialist" would need the coordinator's full context to function — this is a sign the boundary is in the wrong place
- You are splitting for organizational reasons ("the database team wants their own agent") rather than technical ones
- The task is simple enough that tool-use within a single agent handles it cleanly

## Evolutionary Decomposition

Do not design a multi-agent system on a whiteboard. Grow it from a working single-agent system.

**The decomposition journey:**

1. **Single agent with all tools.** This is your starting point. It works until context limits hit or capability conflicts emerge. Do not skip this step — you need the empirical evidence of where the single agent struggles.

2. **Extract the first specialist.** Look for the capability that is most context-heavy or most frequently called. Extract it into its own agent with its own focused system prompt and tool set. The original agent becomes a coordinator by default.

3. **Add a thin coordinator.** Once you have 3+ specialists and routing logic starts cluttering the original agent's prompt, extract the routing into a dedicated coordinator with no domain tools — only the ability to delegate.

4. **Stop.** Resist the urge to keep splitting. Every new agent adds coordination cost, increases latency, and creates another failure mode. Add agents only when you have measured evidence that the current system cannot handle the load, context, or capability requirements.

### Worked Example: Code Review System

**Start**: single agent with tools: `read_file`, `grep`, `glob`, `web_search`, `create_comment`. System prompt: "You are a code reviewer. Read the diff, research relevant best practices, check for security issues, and post review comments."

**Problem 1**: context fills up. The agent reads the diff (2K tokens), searches for best practices (8K tokens of search results), reads 5 related files (15K tokens), and has barely any room left for reasoning. Signal: context saturation.

**Split 1**: extract a research agent. The research agent gets `web_search` and `read_file`. The original agent keeps `grep`, `glob`, `create_comment` and becomes the reviewer. Coordinator delegates: "research best practices for X pattern" → gets a 500-token summary back instead of 8K of raw results.

**Problem 2**: security review needs different expertise and tools. The reviewer's system prompt is trying to be both a style reviewer and a security auditor. It catches style issues well but misses vulnerabilities. Signal: role specialization.

**Split 2**: extract a security specialist with its own system prompt focused on OWASP patterns, plus access to a `vulnerability_db` tool the style reviewer doesn't need.

**Result**: 3 agents (coordinator, research, security) + the original reviewer, now focused on style and correctness. Each agent's context budget is comfortable and its system prompt is focused. The coordinator is thin — it reads the diff, routes to the relevant specialists, and synthesizes their findings.

**What we did NOT split**: the coordinator still handles final comment creation. It could delegate this to a "comment writer" agent, but that would add a hop without adding value — the coordinator already has the synthesized findings and can write the comment directly.

The most common mistake is jumping to step 3 or 4 on day one. Premature decomposition is harder to recover from than a monolithic agent — at least the monolith works, even if slowly. An over-decomposed system might not work at all.

## Related Skills

- For how agents communicate once decomposed, see `agent-communication`
- For designing the tools agents use, see `tool-design`
- For managing state across agents, see `agent-state`
- For tracing and debugging multi-agent decompositions, see `agent-observability`
