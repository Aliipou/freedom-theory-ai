# Freedom Kernel — Architecture

## One principle

```
EVERYTHING IS CAPABILITIES
```

Not "AI is good." Not "AI has values." Not "AI follows rules."

**AI cannot exceed granted authority.**

That is the only invariant that matters at AGI scale.

---

## The real problem

When a system has planning, tool use, self-improvement, memory, multi-agent
coordination, and recursive goal pursuit — the question is no longer:

> "What values does it have?"

The question becomes:

> "Who controls execution authority?"

Values can be argued away. Authority boundaries cannot.

This is why alignment research that operates at the level of preferences,
principles, or reward models will fail at AGI scale: any rule stated in
natural language can be synthesized into a new rule that permits the harm.
A capability boundary cannot be reasoned around — it can only be violated,
and violations are detectable.

---

## The right analogy

AGI alignment will look more like **operating system security** than moral
philosophy.

| OS security primitive | Freedom Kernel equivalent |
|---|---|
| Process / namespace | `ExecutionContext` — bounded authority scope |
| Capability token | `RightsClaim` — unforgeable, scoped, revocable |
| No ambient authority | Agent has only what's in its context |
| `setuid` / privilege escalation | Blocked by sovereignty flags |
| Attenuation | `registry.delegate()` — you cannot grant what you don't have |
| Revocation | `context.revoke()` / claim expiry |
| Audit log | Signed `VerificationResult` chain |
| seccomp / syscall filter | `FreedomVerifier` — gate on every action |

The kernel enforces the same invariant that capability-secure operating
systems enforce: **no authority is ever invented inside the system** — it
can only flow downward from the root (human owner), and only attenuated.

---

## Current state

```
Action IR
    │
    ▼
FreedomVerifier  ←  OwnershipRegistry
    │
    ▼
PERMITTED / BLOCKED + VerificationResult (signed)
```

**What this handles:**
- Single-shot action verification
- Ownership and delegation checks (A4, A7)
- Machine sovereignty constraints (A6)
- 10 hard forbidden flags
- Conflict detection
- Cryptographic attestation of decisions

**What is missing for AGI-grade:**

---

## The gap: from gate to substrate

### 1. Attenuation enforcement ✓ (implemented)

`registry.delegate(claim, delegated_by)` now enforces:
- You cannot grant read if you lack read
- You cannot grant write if you lack write
- You cannot sub-delegate if you lack `can_delegate=True`
- You cannot grant confidence higher than you hold

Without this, the ownership graph is annotations. With it, the graph is a
real capability system: authority can only flow from owners downward, never
be invented inside the system.

### 2. Bounded execution contexts ✓ (implemented)

`ExecutionContext` gives each agent task a fixed authority ceiling:
- No ambient authority: agent has only what's in the context
- Child contexts are strictly ⊆ parent's authority (attenuation)
- Hard depth limit: prevents recursive delegation explosion
- Revocability: any context (and all descendants) can be cancelled instantly
- Expiry: time-bounded authority

### 3. Plan / trajectory verification — next

A single `verify(action)` is not enough for a planning agent. The verifier
needs to check multi-step plans:

```python
results = verifier.verify_plan([action1, action2, action3])
# Check that the entire trajectory stays within authority scope
# and that no intermediate state creates a sovereignty violation
```

This requires modeling *state changes* through a plan, not just
single actions.

### 4. Goal tree constraints — planned

Recursive goal pursuit is how AGI systems work. A goal G spawns subgoals
G1, G2, G3. Each subgoal must stay within the authority scope of G.

The kernel needs a `GoalNode` type and a `verify_goal_tree` function that
checks the entire goal decomposition against the ownership graph.

### 5. Formal semantics — critical, not yet done

Without formal semantics, there is no way to prove that the kernel enforces
what it claims to enforce. This is not optional for AGI-grade infrastructure.

**Target:** TLA+ or Lean proof of the core invariants:
- Authority monotonicity: authority never increases as it flows down
- Confinement: a confined agent cannot leak authority outside its context
- Non-bypass: the verifier cannot be bypassed from within the system

Until these proofs exist, the kernel is **research-grade**, not
**security-grade**. That gap must be closed before production deployment.

---

## Architecture roadmap

### Stage 1 — Stable minimal core (current)

```
engine.rs           ✓  pure Rust verification, no I/O
wire.rs             ✓  JSON wire format
crypto.rs           ✓  ed25519 attestation
ffi.rs              ✓  C ABI (language-agnostic)
registry.delegate   ✓  attenuation enforcement
ExecutionContext     ✓  bounded execution scope
```

### Stage 2 — Plan and goal verification

```
verifier.verify_plan(actions)      → list[VerificationResult]
GoalNode + verify_goal_tree        → checks recursive subgoal authority
StateProjection                    → models registry state after each action
```

### Stage 3 — Multi-agent authority propagation

```
AgentSpawnRequest    → spawning a sub-agent is itself a verified action
DelegationChain      → full chain from root human to executing agent
AuthorityBudget      → max compute/calls/time per context
```

### Stage 4 — Formal verification

```
tla+/freedom_kernel.tla     → TLA+ spec of core invariants
lean/FreedomKernel.lean     → Lean4 proof of confinement
model-checking/             → exhaustive state-space exploration
```

### Stage 5 — Runtime integrations

```
adapters/openai_agents/     → intercept tool calls via function-call hook
adapters/anthropic/         → Claude tool use → Action IR → verify → execute
adapters/langchain/         → LangChain tool wrapper
adapters/browser_agents/    → browser action interceptor
wasm/                       → verifier compiled to WASM (runs anywhere)
```

---

## Philosophical foundation vs. runtime coupling

**وفاداری فلسفی ≠ coupling معماری** — philosophical loyalty ≠ architectural coupling.

The axioms come from *نظریه آزادی* (Theory of Freedom) by Mohammad Ali Jannat Khah
Doust. That origin is real and should not be erased. But the runtime enforces
machine-checkable propositions over a typed ownership graph — not a theology.

The distinction must be explicit everywhere:

| Layer | Axiom | Status |
|---|---|---|
| Metaphysical | A1: every person's ultimate ownership is not by any human, state, or machine | Declared — ontological foundation, not runtime-enforced |
| Governance | A2–A3: no human owns another; every person has typed property rights | Declared — grounds the ownership model |
| **Operational** | **A4–A7: machine ownership, scope, dominion, delegation** | **Runtime-enforced — machine-checkable** |

A1 is not theology inside a syscall filter. It is the *why* behind A4–A7.
A4–A7 work as a formal system independently of anyone accepting A1.

**What this means in practice:**

- The runtime never checks metaphysical claims
- The runtime never scores "spirit" or "divine alignment"
- The runtime checks: does this agent have a human owner? Does it have a delegated claim? Does this action set a sovereignty flag?
- Those checks are purely structural — they require no worldview commitment from the caller

**What must NOT enter the runtime:**
- Satan detection
- Mysticism scoring
- Metaphysical alignment metrics
- Mahdavi compass as a hard gate (it is an optional extension, not a kernel primitive)
- Any check that requires theological interpretation

**What must stay in the foundation:**
- The ontological framing in documentation (A1 explains *why* A6 exists)
- Attribution to the source theory
- The moral weight that motivates the engineering constraints

Strip the ontology and the project becomes a generic capability kernel with no
identity. Couple the ontology into the runtime and it becomes a worldview-locked
monolith that no one outside the theory's community will deploy.

The right boundary: philosophy explains the axioms; the runtime enforces them.

---

## What the kernel is NOT

- Not a moral philosophy engine
- Not a preference optimizer
- Not a value alignment system
- Not a constitutional AI system
- Not a classifier or detector

Those approaches fail at AGI scale because they are stated in natural
language and can be synthesized away.

The kernel enforces **one thing**: authority boundaries.
Everything else follows from that.

---

## The five fatal mistakes to avoid

1. **Rewrite mania** — do not rewrite to Rust/C/WASM before the architecture
   is stable and formally specified. A fast implementation of the wrong design
   is worse than a slow implementation of the right one.

2. **Feature explosion** — the kernel must remain minimal. Every feature added
   to the kernel is a surface area that must be formally verified. Extensions
   belong in `extensions/`, not in `kernel/`.

3. **Philosophy inside the runtime** — the runtime enforces capability
   boundaries. It does not reason about ethics, justice, or metaphysics.
   Those belong in documentation, not in code paths.

4. **Grandiosity in claims** — do not claim the kernel solves AGI alignment.
   It enforces authority boundaries for agentic systems. That is a precise,
   valuable, verifiable claim. Keep it.

5. **No formal semantics** — this is fatal. Without a formal spec and
   mechanically-checked proofs, the kernel cannot be trusted for
   production AGI deployment. This is the most critical missing piece.

---

## The target

```
"Capability-security operating layer for autonomous agents"
```

Not "AI morality engine." Not "ethics runtime." Not "alignment system."

A capability-security kernel: tiny, formal, auditable, composable,
language-agnostic, cryptographically verifiable, and provably correct.

The same class of infrastructure as seccomp, SELinux, and capability-secure
operating systems — but for agentic AI execution.
