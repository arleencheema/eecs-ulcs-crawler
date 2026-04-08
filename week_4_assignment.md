# Week 4 Homework Guide

## Building Your Context Infrastructure

### Total Time: 8-10 hours (tiered — see breakdown below)

---

## OVERVIEW

This week you transform from someone who types context into every prompt to someone who builds context as infrastructure. You’ll write a CLAUDE.md file that makes AI follow your conventions automatically, optionally configure MCP servers and custom agents, and prove the infrastructure works by building a real feature.

This is the final week of the Foundation phase. After this, you have everything you need to analyze real codebases in Week 5.

### Time Allocation by Tier

| Tier | Total Hours | CLAUDE.md | MCP Servers | Custom Agents | Feature Build | Comparison Doc |
| --- | --- | --- | --- | --- | --- | --- |
| **Minimum** | 3-4 hrs | 1.5 hrs | — | — | 1 hr | 0.5 hr |
| **Standard** | 5-7 hrs | 1.5 hrs | 2 hrs | — | 1.5 hrs | 1 hr |
| **Advanced** | 8-10 hrs | 1.5 hrs | 2 hrs | 2 hrs | 1.5 hrs | 1 hr |

**Push yourself.** The minimum is not the easy path — a genuinely good CLAUDE.md is hard to write. But if you can go further, the MCP and agent experience will pay dividends in Weeks 5-8.

---

## BEFORE YOU START

### Prerequisites

- [X]  Week 3 project accessible and working (you’ll extend it)
- [X]  Claude Code installed and configured (from Week 3)
- [X]  Your Week 3 Decision Log entries available for reference
- [X]  Supporting materials document open for templates and references

### Mindset

This week is about infrastructure, not features. The feature you build at the end is the TEST — it proves the infrastructure works. Resist the urge to jump to the feature. Build the foundation first.

Think of it like setting up a workshop before building furniture. Yes, organizing your tools takes time. But every project after this goes faster.

### Gather These Before Starting

1. Your Week 3 project’s README (or mental model of what you built)
2. A list of conventions you followed (naming, error handling, structure)
3. Architecture decisions you made (database choice, framework, patterns)
4. Known limitations or quirks in your project
5. Business context: what does your project do and for whom?

---

## PART 1: CLAUDE.md — Your Project’s Brain (1.5 hours)

### What It Is

CLAUDE.md is a markdown file placed in your project root that Claude Code reads automatically at the start of every session. It gives AI persistent knowledge about your project without you typing anything.

This is not documentation for humans (though humans benefit too). This is context infrastructure — written specifically to make AI produce output that matches your project’s standards.

### Step-by-Step Instructions

**Step 1: Create the file (5 minutes)**

Create a file called `CLAUDE.md` in the root of your Week 3 project. Use the template below as your starting structure.

**Step 2: Fill in each section (45 minutes)**

For each section, ask yourself: “If a senior engineer joined my team tomorrow and needed to contribute code in their first hour, what would they need to know?”

Use the template below. Fill in EVERY section. If a section doesn’t apply, write “N/A — [reason]” so it’s clear you considered it.

**Step 3: Test it (20 minutes)**

This is the critical step most people skip.

1. Close your current Claude Code session entirely
2. Open a NEW session in your project directory
3. Give it a real task: “Add input validation to the create endpoint” or similar
4. Evaluate: Does the output follow your conventions? Use your patterns? Respect your architecture?

**Step 4: Iterate (20 minutes)**

Based on the test, improve your CLAUDE.md:
- If AI used wrong naming → make conventions more explicit
- If AI ignored architecture → add more architecture detail
- If AI didn’t handle errors your way → add error handling examples
- If AI modified something it shouldn’t → add constraints section

Test again. Repeat until satisfied.

### CLAUDE.md Template

```markdown
# [Project Name]

## Business Context
<!-- What does this project do? Who uses it? Why does it exist? What's the business impact? -->

## Architecture Overview
<!-- Tech stack. Key patterns. Service/module boundaries. Data flow. -->

### Tech Stack
-Language:
-Framework:
-Database:
-Key Libraries:

### Project Structure
<!-- Brief description of folder/module organization -->

### Data Model
<!-- Key entities and their relationships. Not every field — just the important ones. -->

## Conventions & Standards

### Naming
-Files:
-Functions/Methods:
-Variables:
-Database tables/columns:

### Error Handling
<!-- How does this project handle errors? Exceptions? Result types? Error codes? -->

### Testing
<!-- Testing philosophy. What's tested, what's not. Test naming convention. -->

### Code Style
<!-- Formatting, commenting, import ordering — anything AI should follow -->

### Logging
<!-- Logging format. What to log, what not to log. Structured or unstructured? -->

## Build & Run

### Development
```

[commands to build and run locally]

```

### Testing
```

[commands to run tests]

```

### Common Tasks
```

[frequently used commands]

```

## Known Issues & Constraints
<!-- What's broken? What's limited? What should NOT be changed? -->

## Important Decisions & Rationale
<!-- Key architectural decisions and WHY they were made -->
```

### Self-Check

- [ ]  Every section has content (or explicit N/A with reason)
- [ ]  Business context section explains WHO uses this and WHY it matters
- [ ]  Conventions are specific enough that AI could follow them without asking
- [ ]  At least one “Known Issue” or constraint is documented
- [ ]  You’ve tested it with a fresh session and iterated at least once
- [ ]  Architecture section describes patterns, not just technologies

---

## PART 2: Custom Agent Configuration (2 hours)  — Standard Tier

### What It Is

Custom agents are specialized Claude Code behaviors with pre-configured instructions and context. Instead of one general-purpose AI, you create role-specific agents that bring different expertise to different tasks.

### Step-by-Step Instructions

**Step 1: Choose your first agent (20 minutes)**

Pick ONE agent that would provide the most value for your project. Consider:

| Agent Role | When to Use | Key Instructions to Include |
| --- | --- | --- |
| **Reviewer** | Before committing code | Your conventions, common mistakes, SOLID principles |
| **Architect** | When designing new features | System boundaries, patterns, trade-off framework |
| **Tester** | When writing tests | Test strategy, edge cases, coverage requirements |
| **Debugger** | When investigating issues | Logging strategy, common failure patterns |

**Step 2: Design the agent (30 minutes)**

Write the agent’s system prompt. Include:
- Role definition: what this agent does and doesn’t do
- Context: what conventions/standards it enforces
- Behavior: how it should respond (format, tone, focus areas)
- Constraints: what it should never do

Template:

```markdown
# [Agent Name] Agent

## Role
You are a [role] for [project name]. Your job is to [primary responsibility].

## Standards You Enforce
-[Convention 1]
-[Convention 2]
-[Convention 3]

## How You Respond
-[Format guidance]
-[Tone guidance]
-[What to always include]

## What You Never Do
-[Constraint 1]
-[Constraint 2]
```

**Step 3: Configure the agent (30 minutes)**

Custom slash commands in Claude Code are configured in your project settings. Create a `/[agent-name]` command that loads your agent’s prompt.

**Step 4: Test and iterate (40 minutes)**

Give your agent a real task. Evaluate:
- Does it stay in character?
- Does it enforce your standards?
- Is the output format useful?
- Where does it fall short?

Iterate on the prompt until the agent is reliably useful.

### Self-Check

- [ ]  At least 1 custom agent designed, configured, and tested
- [ ]  Agent has clear role boundaries (what it does and doesn’t do)
- [ ]  Agent enforces project-specific standards, not generic best practices
- [ ]  Documented rationale: why this agent? When would you use it?
- [ ]  Tested with at least 2 real tasks with satisfactory results

---

## PART 3: MCP Server Integration (2 hours) — Advanced Tier

### What It Is

MCP (Model Context Protocol) servers give Claude Code live access to external tools. Instead of just reading about your project, AI can interact with your actual systems — query databases, read files across directories, access documentation.

### Step-by-Step Instructions

**Step 1: Choose your servers (15 minutes)**

Pick 2 MCP servers that make sense for your project. Consider:

| Server | Best For | Recommended If… |
| --- | --- | --- |
| **Filesystem** | Reading/writing project files | Almost always useful |
| **SQLite/PostgreSQL** | Querying your database | Your project has a database |
| **Fetch** | Accessing web APIs or documentation | You reference external docs |
| **Memory** | Persistent knowledge across sessions | You want AI to remember decisions |
| **GitHub** | Repository operations | Your project is on GitHub |

For each server you choose, write down:
1. What context will this give AI?
2. What will AI be able to do that it couldn’t before?
3. What’s the security consideration?

**Step 2: Configure MCP servers (45 minutes)**

Configuration lives in your Claude Code settings. The exact location depends on your setup:

- **Project-level:** `.claude/settings.json` in your project root
- **Global:** `~/.config/claude/settings.json`

Use project-level for project-specific servers.

Example configuration structure:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/your/project"]
    },
    "sqlite": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "/path/to/your/database.db"]
    }
  }
}
```

**Step 3: Test each server (30 minutes)**

For each configured server:
1. Start a new Claude Code session
2. Ask AI to use the server: “What files are in the project root?” or “What tables are in the database?”
3. Verify AI can access the expected resources
4. Try a real task that uses the server’s capabilities

**Step 4: Document (30 minutes)**

For each MCP server, document:
- What it is and what it provides
- How it changes AI’s capabilities for your project
- Security considerations — what access does it have? What doesn’t it have?
- One concrete example of improved output because of the server

### Self-Check

- [ ]  At least 2 MCP servers configured and working
- [ ]  Each server tested with a real task
- [ ]  Documentation explains the CONTEXT each server provides (not just technical setup)
- [ ]  Security considerations explicitly addressed
- [ ]  Before/after example shows how MCP changed AI output

---

## PART 4: Feature Build — The Proof (1-1.5 hours)

### What It Is

All the infrastructure in the world is worthless if it doesn’t improve output. This part proves it works.

### Step-by-Step Instructions

**Step 1: Choose a substantial feature (10 minutes)**

Pick something that would exercise your project’s conventions. Good choices:
- A new API endpoint with validation, error handling, and tests
- A search/filter feature with database queries
- An integration with an external service
- A reporting or analytics feature

The feature should be complex enough that conventions matter. “Add a health check endpoint” is too simple. “Add user authentication with role-based access” is about right.

**Step 2: Build WITHOUT your infrastructure (30 minutes)**

If you haven’t already done this in Week 3 without CLAUDE.md:
1. Start a fresh Claude Code session
2. Make sure CLAUDE.md is NOT loaded (rename it temporarily)
3. Prompt AI to build the feature
4. Save the output (you’ll compare later)

If you have Week 3 output without infrastructure, use that as your “before.”

**Step 3: Build WITH your infrastructure (30 minutes)**

1. Restore your CLAUDE.md
2. Start a fresh Claude Code session (with MCP servers active if configured)
3. Give the SAME prompt for the SAME feature
4. Save the output

**Step 4: Compare and document (20 minutes)**

Create a comparison document. For each difference:
- What was different in the output?
- What piece of context infrastructure caused the difference?
- Is the “with infrastructure” version actually better? Why?

### Self-Check

- [ ]  Feature is substantial enough to exercise conventions
- [ ]  Before/after comparison is documented with specific differences
- [ ]  Each difference is traced to a specific context infrastructure element
- [ ]  Honest assessment: where did infrastructure help and where didn’t it?

---

## DELIVERABLES

All tiers submit these:

1. **CLAUDE.md** — tested and iterated (with notes on what you changed after testing)
2. **Feature code** — the new feature built with context infrastructure
3. **Before/After Comparison** — specific differences traced to infrastructure elements
4. **Decision Log entry** — see prompts below

Standard tier adds:
5. **MCP Server Documentation** — what you configured, what context it provides, security considerations

Advanced tier adds:
6. **Custom Agent Configuration** — prompt, rationale, test results
7. **Agent Rationale Document** — when to use it, what it enforces, how it fits the EPCC framework

---

## EVALUATION RUBRIC

| Criteria | Weight | Junior (Learning) | Mid (Competent) | Senior (Mastery) |
| --- | --- | --- | --- | --- |
| **CLAUDE.md Quality** | 30% | Exists but sparse. Missing sections. Generic content. | Complete. Project-specific. Covers architecture and conventions. | Rich with business context, constraints, known issues. Tested and iterated. |
| **Test & Iterate Evidence** | 25% | No evidence of testing. | Tested once, made some changes. | Multiple iterations with documented improvements. Clear methodology. |
| **Context Articulation** | 20% | Can describe what they configured. | Can explain what context each element provides. | Can trace specific output improvements to specific infrastructure decisions. |
| **Feature Quality** | 15% | Feature works. | Feature follows conventions established in CLAUDE.md. | Feature demonstrates measurable improvement from infrastructure. |
| **Decision Log** | 10% | Entry exists. | Reflects on what worked and didn’t. | Synthesizes learning about context quality with business implications. |

**Note:** This rubric evaluates THINKING QUALITY, not configuration perfection. A well-reasoned minimum-tier submission that demonstrates clear improvement outscores a poorly-understood advanced-tier submission.

---

## COMMON PITFALLS

1. **Writing CLAUDE.md without testing it.** The most common mistake. A beautiful CLAUDE.md that AI doesn’t follow is useless. The test loop is the entire point.
2. **Making CLAUDE.md too generic.** “We follow clean code principles” tells AI nothing actionable. “Error handling uses Result pattern — never throw exceptions from service layer” is specific and enforceable.
3. **Skipping the business context section.** Engineers love documenting tech stack and skip business context. But knowing “this handles 50K orders/day” changes how AI thinks about error handling, caching, and reliability. Business context is not optional.
4. **Configuring MCP servers without documenting why.** If you can’t explain what context a server provides, you’re configuring for the sake of configuring. Every server should have a clear rationale.
5. **Making the before/after comparison vague.** “Output was better” is not a comparison. “With CLAUDE.md, AI used PascalCase for public methods (matching our convention in line 47 of CLAUDE.md) instead of camelCase” is a comparison.
6. **Writing too much.** CLAUDE.md should be comprehensive but concise. AI reads the whole thing every session. If it’s 500 lines, the signal-to-noise ratio drops. Aim for 50-150 lines of high-value content.
7. **Forgetting constraints.** The “Known Issues” and “DO NOT modify” sections are often the most valuable. They prevent AI from making expensive mistakes.
8. **Not iterating.** Your first CLAUDE.md will have gaps. That’s expected. The senior behavior is noticing the gaps through testing and fixing them. One-and-done is junior behavior.

---

## DECISION LOG ENTRY

This week’s Decision Log should address:

### Required Prompts

1. **What context did I include in CLAUDE.md and why?** For each major section, explain the reasoning. What would AI get wrong without this information?
2. **What did I learn from testing?** What gaps did you find when you tested with a fresh session? What surprised you about what AI did or didn’t pick up?
3. **What did I iterate on?** What changed between your first draft and final version? Why?

### If You Completed Standard Tier

1. **What context do MCP servers provide that CLAUDE.md can’t?** What’s the difference between telling AI about your database and giving AI access to your database?

### If You Completed Advanced Tier

1. **Why did you choose this agent role?** What problem does it solve? How does it connect to the EPCC framework?

### Reflection Prompt

“Before this week, I thought good AI output came from [X]. Now I understand it comes from [Y].”

---

## LOOKING AHEAD

After this week, you have the complete Foundation toolkit:
- **Week 1:** Thinking habits — asking “why” before “how”
- **Week 2:** Prompting skills — prompts as technical specifications
- **Week 3:** Tool proficiency — building in any language with AI Code
- **Week 4:** Context infrastructure — making AI effective automatically

**Week 5 is the pivot.** You’ll move from greenfield projects (building new things) to legacy codebases (understanding existing things). It will feel different. Harder in some ways. That’s the point.

Your CLAUDE.md skill transfers directly — you’ll write one for the legacy codebase you analyze. Your testing methodology transfers — you’ll use the same “understand → test → iterate” loop on real systems. Everything you built this month has a purpose.