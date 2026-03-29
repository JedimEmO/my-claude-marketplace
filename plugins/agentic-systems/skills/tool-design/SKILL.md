---
name: tool-design
description: Use when the user asks about designing tools for agents, tool granularity, tool schemas, input/output contracts, error contracts for tools, tool composability, tool descriptions, idempotency, or when building the tool layer of an agentic system.
version: 1.0.0
---

# Tool Design — Interfaces, Contracts, and Composability

Tools are the hands of an agent. The quality of your agentic system is bounded by the quality of its tool interfaces. A brilliant agent with poorly designed tools will produce poor results — it will call the wrong tool, pass the wrong parameters, misinterpret the output, and burn tokens recovering from avoidable confusion. Tool design is API design. The consumer just happens to be an LLM instead of a developer. This changes the priorities (descriptions matter more, consistency matters more, error clarity matters more) but the core discipline is identical: design for the caller, not for the implementer.

## Tool Granularity

The right granularity: a tool should do one meaningful thing that the agent cannot accomplish through reasoning alone.

**Too granular** — the agent spends tokens orchestrating micro-steps. If the agent always calls tool A then tool B then tool C in that exact sequence, those should be one tool. You are forcing the agent to be a workflow engine, and agents are bad workflow engines. Every tool call is a decision point where the agent can make a mistake. Minimize unnecessary decision points.

**Too coarse** — the agent loses control. If a tool does 5 things and the agent only needed 1, the other 4 are wasted work or, worse, unwanted side effects. A tool that "creates a project, initializes git, installs dependencies, and opens the editor" is four tools pretending to be one. The agent that just wanted to create a directory now has an editor window open.

**The litmus test**: can the agent meaningfully choose NOT to call this tool in some scenarios? If the answer is always yes — sometimes the agent needs this, sometimes it doesn't — the granularity is right. If the agent must always call it as part of every workflow, it should be automatic or implicit, not a tool. If the agent never calls it independently (always paired with another tool), merge them.

**Compound tools are fine** when the compound operation is the natural unit of work. `search_and_rank` is better than separate `search` + `rank` if ranking without searching never makes sense. The boundary is: does the combination represent a coherent operation, or is it just bundling for convenience?

**Prefer fewer, well-designed tools over many narrow ones.** An agent with 50 tools has a harder time choosing the right one than an agent with 12. If you find yourself adding tools that overlap in purpose, consolidate. Tool sprawl is the tool equivalent of microservice sprawl — it shifts complexity from the implementation to the coordination layer, which is the worst place for it in an agentic system.

**How to evaluate granularity in practice:** list every tool your agent has access to and ask: "If I removed this tool, what task becomes impossible?" If the answer is "nothing becomes impossible, another tool mostly covers it," merge or remove it. Then ask: "If I split this tool in two, would the agent use each half independently?" If yes, split. If not, keep it whole.

**Watch for emergent sequences.** Once your agent is running, look at its tool call traces. If you see the same 3-tool sequence appearing in 80% of completions, that sequence is a candidate for a compound tool. The agent is telling you where your granularity is wrong.

## Schema Design

Tool inputs and outputs are the agent's API contract. Design them like you would design a public API — because that is exactly what they are.

### Inputs

- **Every parameter should have a clear, unambiguous name.** `query` is better than `input`. `max_results` is better than `limit` (which limit?). `file_path` is better than `path` (path to what?). The agent reads the parameter name and the description to decide what to pass. Ambiguous names cause ambiguous behavior.

- **Use enums for constrained choices.** Don't make the agent guess valid values. If a parameter accepts `"json"`, `"csv"`, or `"text"`, say so in the schema. Free-text parameters that secretly only accept specific values are a trap.

- **Required vs optional:** required parameters should be the minimum needed to do the operation at all. Optional parameters with sensible defaults let the agent operate simply in the common case and precisely in the advanced case. If you have more than 3-4 required parameters, the tool is probably too complex.

- **Avoid boolean flags that change behavior dramatically.** If `dry_run=true` and `dry_run=false` produce fundamentally different behavior (one reads, one writes), these should be separate tools. The agent reasons about tool safety from the tool description. A tool that is "safe" in one mode and "destructive" in another mode forces the agent to track state it should not have to.

- **Accept the most natural input format.** If the agent will have a file path, accept a file path — don't require a file ID that forces a lookup first. If the agent will have natural language, accept natural language — don't require structured syntax.

- **Validate early, fail fast.** Check all inputs before doing any work. An agent that gets a validation error after a tool has already partially executed is in an ambiguous state. Did the side effect happen or not? Validate everything up front, then execute. If validation fails, return a clear error listing all invalid parameters at once — not just the first one found.

- **Document valid ranges.** If `max_results` has a ceiling of 1000, say so. If `query` must be under 500 characters, say so. Undocumented limits cause silent truncation or cryptic failures that the agent cannot diagnose.

### Outputs

- **Structured output when downstream processing is needed.** If another tool or agent will consume this result, return JSON with a predictable shape. The agent should not have to parse prose to extract a value.

- **Natural language output when the result is for reasoning.** If the agent needs to think about the result (summarize it, make a judgment call, explain it to the user), natural language is fine. Not everything needs to be JSON.

- **Include metadata.** Status, count, pagination info, timestamps. Don't make the agent infer "there are more results" from the absence of results. Don't make the agent guess whether an empty list means "no matches" or "something went wrong."

- **Consistent output shape.** Success and error cases should have the same top-level structure. If success returns `{"status": "ok", "data": [...]}`, then failure should return `{"status": "error", "error": {...}}` — not a raw string or a different shape entirely. The agent should not have to detect the output format before it can process the output.

- **Trim aggressively.** Return what the agent needs, not what the implementation happens to have. An agent working in a 200k context window does not need 50KB of raw log output. Summarize, filter, truncate — and offer a way to get the full data if needed (write to file, paginate).

- **Prefer stable output ordering.** If a tool returns a list, sort it deterministically. The agent may compare outputs from successive calls to detect changes. Non-deterministic ordering makes comparison unreliable and wastes reasoning effort.

## Error Contracts

How a tool communicates failure determines whether the agent can recover or just flails. Error design is not an afterthought — it is half the interface.

### Recoverable Errors

The agent can retry or try a different approach. The error must include: what went wrong, why, and what the agent could try instead.

```json
{
  "status": "error",
  "error": "rate_limited",
  "message": "API rate limit exceeded (60 requests/minute)",
  "recoverable": true,
  "retry_after_seconds": 30,
  "suggestion": "Reduce batch size or wait before retrying"
}
```

The agent reads this and knows: wait 30 seconds, then retry. Or reduce batch size. It has a plan.

### Terminal Errors

Nothing the agent can do. Be explicit about this so the agent does not waste tokens retrying.

```json
{
  "status": "error",
  "error": "not_found",
  "message": "Repository 'foo/bar' does not exist",
  "recoverable": false
}
```

The agent reads this and knows: stop trying, inform the user.

### What Not to Return

- **Raw stack traces.** Useless to an agent, wastes context. The agent cannot fix your NullPointerException.
- **Ambiguous errors.** "Something went wrong" gives the agent nothing to work with. It will guess, and it will guess wrong.
- **Silent success on failure.** Returning `{"status": "ok"}` when the operation actually failed is the worst possible outcome. The agent proceeds confidently on a false foundation.

### Error Categories the Agent Should Distinguish

| Category | Agent Response | Example |
|----------|---------------|---------|
| Input validation | Fix the inputs and retry | "Parameter 'date' must be ISO 8601 format" |
| Transient failure | Wait and retry | "Connection timeout after 30s" |
| Permission denied | Escalate or abort | "API key lacks write access to this resource" |
| Not found | Search differently or inform user | "No file matching pattern '*.rs' in /empty-dir" |
| Resource exhausted | Back off or reduce scope | "Result set exceeds 10MB limit, add filters" |

Design your error responses so the agent can distinguish these categories programmatically — not by parsing English sentences.

**The `recoverable` field.** This single boolean saves more wasted tokens than any other error design choice. When the agent sees `"recoverable": false`, it stops trying. Without it, the agent may retry a permissions error five times, burning tokens and time on an operation that will never succeed. Cheap to implement, massive impact.

**Error messages should be written for the agent, not for a log file.** The agent does not need to know which line of code threw the exception. It needs to know what it can do about it. "Query parameter 'since' must be an ISO 8601 timestamp, received '2 days ago'" is actionable. "ValueError at line 342 in parser.py" is not.

## Composability

Tools that compose well create emergent capability. Tools that don't compose force the agent to be the glue code, and agents are unreliable glue code. But composability is not always a virtue — for safety-critical flows where tools must be called in a specific sequence with specific guards, constrain the sequence explicitly rather than relying on the agent to discover the correct composition. The goal is composability where flexibility helps, and rigidity where safety demands it.

**Output-to-input compatibility.** The output of tool A should be directly usable as input to tool B without the agent reformatting. If the agent is always extracting a field from one tool's output to pass to another, you have a design problem. Either adjust the schemas to be compatible, or create a compound tool that handles the pipeline internally.

**Consistent conventions across all tools.** All tools that return lists use the same pagination pattern. All tools that accept identifiers use the same ID format. All tools that accept paths use the same path format (absolute vs relative, with or without trailing slash). Consistency reduces the cognitive load on the agent. Every inconsistency is a potential error.

**Pipeline-friendly design.** Tools that naturally chain (fetch then transform then store) should have compatible interfaces. The output of `fetch` should be a valid input to `transform` without the agent understanding serialization formats, encoding, or data layout.

**Avoid hidden coupling.** If tool B only works after tool A has been called (because A sets up some state that B depends on), make this explicit. Better: make B accept the state as a parameter so the dependency is visible in the schema, not hidden in runtime behavior.

**Test composability by chaining.** Take your tool set and attempt common multi-step tasks using only tool outputs as subsequent tool inputs. If you find yourself mentally reformatting data between steps — extracting an ID from a nested object, converting a timestamp format, joining fields into a string — your schemas have friction. Smooth that friction at the tool boundary, not in the agent's reasoning.

**Shared vocabulary.** Define a glossary of terms that all tools use consistently. If one tool calls it `user_id` and another calls it `userId` and a third calls it `account_id` referring to the same concept, the agent will eventually mix them up. One name, one concept, everywhere.

## Idempotency and Side Effects

Agents retry. Agents sometimes call tools they have already called. Agents sometimes call tools speculatively to see what happens. Your tools must handle all of this gracefully.

**Read tools** should always be idempotent and safe to call any number of times. No side effects. No rate limiting that punishes repeated reads. If reading has side effects (audit logging, view counts), those should be invisible to the agent.

**Write tools** should be idempotent where possible. Creating a resource that already exists should return the existing resource, not fail with a conflict error. Updating a resource to a state it is already in should succeed, not complain. The agent should not have to check-then-act — that pattern is both wasteful and racy.

**Label side effects explicitly.** The tool description should state whether calling it changes state. Agents reason about safety and reversibility. A tool described as "Get project details" that secretly triggers a webhook on every call is a violation of trust.

**Side effect categories:**

| Category | Safety | Retry? | Example |
|----------|--------|--------|---------|
| Pure read | Always safe | Always | Search files, get status |
| Idempotent write | Safe to retry | Yes | Update config, set value |
| Non-idempotent write | Each call changes state | Carefully | Send email, post message |
| Destructive | Cannot be undone | Never blind | Delete resource, force push |

Make the category obvious from the tool name and description. Agents that accidentally call a destructive tool because it was poorly labeled are a design failure, not an agent failure.

**Naming conventions that signal intent.** Prefix or suffix tool names to make the side-effect category obvious at a glance. `get_*`, `list_*`, `search_*` for pure reads. `create_*`, `update_*`, `set_*` for idempotent writes. `send_*`, `post_*`, `trigger_*` for non-idempotent writes. `delete_*`, `destroy_*`, `drop_*` for destructive operations. The agent should be able to infer the safety profile from the name alone, before reading the description.

**State visibility.** If a write tool changes state that other tools will reflect, document this. "After calling `deploy_service`, subsequent calls to `get_service_status` will show the new deployment." The agent needs to build a mental model of how tools affect each other. Don't make it guess.

## Description as Interface

The tool description is the only thing between the agent and correct tool usage. It is the API documentation, the README, the type signature, and the docstring — combined into one piece of text that must be complete enough for an LLM to use the tool correctly on the first try.

**A good description answers four questions:**

1. **What does this tool do?** One sentence. No jargon the agent would not know.
2. **When should I use it?** Trigger conditions — what situation calls for this tool?
3. **What should I expect?** Output shape, common outcomes, rough size of results.
4. **When should I NOT use it?** Common confusion with other tools, misuse cases.

**Example — good:**
> Search for files matching a glob pattern in the codebase. Use when you need to find files by name or extension. Returns a list of matching absolute file paths sorted by modification time. Do not use this to search file contents — use Grep for content search.

**Example — bad:**
> File search utility.

The bad description leaves the agent to guess: search by name? By content? By metadata? What does it return? When would I use this vs the other search tool? Every unanswered question is a coin flip the agent will get wrong some percentage of the time.

**Description length:** longer is better than ambiguous, but concise is better than verbose. Aim for 2-4 sentences. If you need more, the tool is probably too complex.

**Include examples in descriptions** for tools with non-obvious input formats. "Accepts glob patterns like `**/*.ts` or `src/**/*.rs`" is more useful than "Accepts a pattern string."

**Describe boundaries between similar tools.** If you have `search_files` and `grep_files`, the description of each should mention the other and explain the boundary. "Use search_files for name/path matching; use grep_files for content matching." Agents encounter all tools simultaneously and need to choose between them. Make the choice obvious.

**Test your descriptions.** Give the tool list (names and descriptions only, no code) to an LLM and ask it which tool to use for a set of tasks. If it picks wrong, your descriptions are unclear. This is the cheapest test you can run and it catches the most common tool design failures.

## Anti-Patterns

**Kitchen-Sink Tool** — one tool with an `action` or `mode` parameter that completely changes behavior. `manage_database(action="create")` and `manage_database(action="drop")` should be separate tools. The agent cannot reason about safety when the same tool creates and destroys.

**Brittle Tool** — fails on any unexpected input. Agent inputs are noisy. Agents add extra whitespace, include quotes around values that don't need them, use slightly different date formats. Good tools handle minor formatting variations. Validate strictly on semantics, loosely on syntax.

**Opaque Tool** — returns "done" or "success" with no useful detail. The agent needs to know WHAT happened, not just that something happened. Return the created resource, the matched count, the operation summary. "Created user with ID 'abc-123'" is actionable. "Success" is not.

**Chatty Tool** — returns megabytes of raw data. The agent's context window is finite and expensive. Tools should return relevant, filtered results. If the full data is needed, write it to a file and return the path. Do not dump a 10,000-line log into the agent's context.

**God Tool** — does everything. "execute_workflow" that takes an arbitrary workflow definition is not a tool, it is a sub-system. The agent cannot reason about what it does because what it does depends entirely on the input. Break it down into tools the agent can understand individually.

**Secret-State Tool** — behavior depends on hidden mutable state that the agent cannot observe. If calling tool A changes what tool B returns, and the agent has no way to know this, you have created a trap. Make state explicit in inputs and outputs.

**Inconsistent Tool** — tools in the same set use different conventions. One returns `{"items": [...]}`, another returns `{"results": [...]}`, a third returns a bare array. One accepts `user_id`, another accepts `userId`. Each inconsistency is a small tax on the agent's attention. Across dozens of tool calls, these taxes compound into failures.

**Undocumented Limit Tool** — has constraints the description does not mention. Silently truncates results at 100 items, times out on queries over 10 seconds, rejects input over 4KB. The agent has no way to know these limits exist and no way to work around them. Document every constraint in the description.

## Design Process

When designing a new tool set, work through these steps in order:

1. **List the tasks the agent must accomplish.** Not the tools — the tasks. "Find relevant source files," "apply a code change," "verify tests pass."
2. **For each task, ask: does this require external action?** If the agent can accomplish it through reasoning alone, it is not a tool. If it requires interacting with the outside world (filesystem, API, database), it is a tool.
3. **Group related actions by resource or domain.** All file operations together, all database operations together. This reveals natural tool boundaries.
4. **Define the input/output contract for each tool.** What is the minimum input? What does the agent need from the output to proceed?
5. **Write the descriptions before the implementations.** If you cannot explain when to use a tool in 2-4 sentences, the tool is not well-defined yet.
6. **Test with an agent.** Give the tool definitions to an LLM, describe a task, and see which tools it selects and how it uses them. Iterate on the design based on where it goes wrong.

## Related Skills

- For deciding which tools belong to which agent: see `agent-decomposition`
- For how tool calls flow between agents: see `agent-communication`
- For monitoring tool usage and handling failures at runtime: see `agent-observability`
- For managing state that tools read and write: see `agent-state`
