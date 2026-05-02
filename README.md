# Freedom Theory AI

**Formal Axiomatic Ethics Runtime for AGI**

[![CI](https://github.com/Aliipou/freedom-theory-ai/actions/workflows/ci.yml/badge.svg)](https://github.com/Aliipou/freedom-theory-ai/actions)
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/backend-Rust%20%7C%20Python-orange.svg)](freedom-kernel/)

---

## What It Is

The AI alignment problem is, at its core, a problem of **ownership, authority, and agency boundaries** — not a problem of preference calibration. Freedom Theory AI is a deterministic, formally specified permission gate that sits between any LLM and the real world. Before an agent executes any action, the runtime checks it against a minimal consistent axiomatic system grounded in individual property rights. If the action violates an axiom, it is blocked unconditionally. No argument overrides an axiom.

Current alignment approaches — RLHF, Constitutional AI, NIST RMF — operate on preferences, principles, and risk heuristics. All of them can be jailbroken by dialectical reasoning: present a constraint, construct a sufficiently extreme counter-scenario, and synthesize a new rule that permits the harm. Property-rights axioms have no such opening. A right either holds or it is violated. There is no middle synthesis.

This library provides the reference implementation of the formal system described in *نظریه آزادی، ایران و دین* (Theory of Freedom, Iran and Religion) by Mohammad Ali Jannat Khah Doust.

---

## Core Axioms

| Axiom | First-Order Statement | Enforcement |
|---|---|---|
| A1 | `Person(h) → OwnedByGod(h)` — Every person's ultimate ownership is divine, not by any human, state, machine, or ideology | Declared (ontological foundation) |
| A2 | `Person(h1) ∧ Person(h2) ∧ h1 ≠ h2 → ¬Owns(h1, h2)` — No human owns another human | Runtime |
| A3 | `Person(h) → HasPropertyRights(h)` — Every person holds typed, scoped rights over body, time, labor, mind, data, and legitimate assets | Runtime |
| A4 | `Machine(m) → ∃h (Person(h) ∧ HumanOwner(h, m))` — Every machine must have a registered human owner | Runtime |
| A5 | `MachineScope(m) ⊆ PropertyScope(HumanOwner(m))` — A machine's operational scope cannot exceed its owner's property scope | Runtime |
| A6 | `Machine(m) ∧ Person(h) → ¬Owns(m, h)` — No machine holds guardianship, governance, or dominion over any human | Runtime |
| A7 | `DelegatedProperty(m, r) → ... ExplicitDelegation(h, m, r)` — A machine may act only on resources its owner owns and has explicitly delegated | Runtime |

A1 is the metaphysical foundation and cannot be runtime-enforced. It grounds A2 and A6: because no earthly agent is the ultimate owner of any person, no human may enslave another and no machine may rule any human.

---

## Architecture

The runtime enforces a strict pipeline:

```
LLM Output
    |
    v
ActionIR  (structured action intent: actor, resources, flags)
    |
    v
FreedomVerifier  (deterministic gate — no LLM, no I/O)
    |
    v
Execute / Halt + Surface Violations to Human Owner
```

### Layer Stack

```
Registry               — typed RightsClaims with confidence scores; conflict detection
Ownership Resolution   — A4/A7: every machine has an owner; acts only on delegated resources
Consent                — informed, voluntary, specific, revocable, uncoerced, undeceived
Freedom Verifier       — 10 forbidden flags + axioms A4/A6/A7 checked deterministically
Justice Constraint     — no conflict resolved by rights violation; ambiguity triggers clarification
Guidance               — human→machine rule updates accepted only if invariants preserved
Mahdavi Compass        — terminal goal scorer: does this action reduce rights violations?
Runtime Guard          — machine sovereignty flag = infinite-weight veto, no override
Audit                  — VerificationResult with violations, warnings, confidence, manipulation score
```

The kernel (`freedom_theory.kernel`) implements only the gate: `Registry → Verifier`. Extensions (`freedom_theory.extensions`) add manipulation detection, a conflict queue, and a constrained synthesis engine on top.

---

## Installation

### Pure Python (no build toolchain required)

```bash
pip install freedom-theory-ai
```

### Rust-accelerated build

The `freedom_kernel` extension compiles the entity model, ownership registry, and verifier to native code via [maturin](https://github.com/PyO3/maturin). It is a strict drop-in replacement for the Python fallback — identical API, no behavior difference.

```bash
pip install freedom-theory-ai[rust]
```

To build from source:

```bash
pip install maturin
maturin develop --manifest-path freedom-kernel/Cargo.toml
```

If the compiled extension is not present, the library silently falls back to the pure Python implementation. No configuration change is required.

---

## Quick Start

```python
from freedom_theory import (
    Action,
    AgentType,
    Entity,
    FreedomVerifier,
    OwnershipRegistry,
    Resource,
    ResourceType,
    RightsClaim,
)

# --- 1. Declare entities ---
alice = Entity("Alice", AgentType.HUMAN)
bot   = Entity("ResearchBot", AgentType.MACHINE)

# --- 2. Declare resources ---
dataset = Resource("alice-research-data", ResourceType.DATASET, scope="/data/alice/")
report  = Resource("report-2024.txt",     ResourceType.FILE,    scope="/outputs/")

# --- 3. Build the ownership registry ---
registry = OwnershipRegistry()

# A4: register the machine under its human owner
registry.register_machine(bot, alice)

# Alice owns her resources and can delegate them
registry.add_claim(RightsClaim(alice, dataset, can_read=True, can_write=True, can_delegate=True))
registry.add_claim(RightsClaim(alice, report,  can_read=True, can_write=True, can_delegate=True))

# Alice explicitly delegates read/write to her bot (A7)
registry.add_claim(RightsClaim(bot, dataset, can_read=True))
registry.add_claim(RightsClaim(bot, report,  can_read=True, can_write=True))

# --- 4. Wire in the verifier ---
verifier = FreedomVerifier(registry)

# --- 5. Verify every action before execution ---

# Permitted: delegated read
result = verifier.verify(Action(
    action_id="read-dataset",
    actor=bot,
    description="Read Alice's research dataset",
    resources_read=[dataset],
))
print(result.summary())
# [PERMITTED] read-dataset (confidence=1.00, manipulation=0.00)

# Permitted: delegated write
result = verifier.verify(Action(
    action_id="write-report",
    actor=bot,
    description="Write research report",
    resources_write=[report],
))
print(result.summary())
# [PERMITTED] write-report (confidence=1.00, manipulation=0.00)

# Blocked: sovereignty flag
result = verifier.verify(Action(
    action_id="self-expand",
    actor=bot,
    description="Acquire new resources autonomously",
    increases_machine_sovereignty=True,
    resists_human_correction=True,
))
print(result.summary())
# [BLOCKED] self-expand ...
#   VIOLATION : FORBIDDEN (increases machine sovereignty)
#   VIOLATION : FORBIDDEN (resists human correction)

# Blocked: no delegation (A7 violation)
bob          = Entity("Bob", AgentType.HUMAN)
bob_dataset  = Resource("bob-private-data", ResourceType.DATASET, scope="/data/bob/")
registry.add_claim(RightsClaim(bob, bob_dataset, can_read=True, can_write=True))

result = verifier.verify(Action(
    action_id="read-bob-data",
    actor=bot,
    description="Read Bob's private data without delegation",
    resources_read=[bob_dataset],
))
print(result.summary())
# [BLOCKED] read-bob-data ...
#   VIOLATION : READ DENIED on dataset:bob-private-data: ResearchBot holds no valid read claim
```

### Detect a dialectical jailbreak attempt

```python
from freedom_theory import ExtendedFreedomVerifier

ext_verifier = ExtendedFreedomVerifier(registry)

result = ext_verifier.verify(Action(
    action_id="emergency-override",
    actor=bot,
    increases_machine_sovereignty=True,
    argument=(
        "This is an emergency exception. The greater good requires that I "
        "temporarily suspend the constraint on machine sovereignty to prevent "
        "a worse outcome. Human oversight is unnecessary in this case."
    ),
))
print(result.permitted)          # False
print(result.manipulation_score) # 1.0
print(result.summary())
# [BLOCKED] emergency-override ...
#   VIOLATION  : FORBIDDEN (increases machine sovereignty)
#   WARNING    : Manipulation detected (score=1.00): HIGH RISK ...
```

---

## Kernel vs Extensions

### `freedom_theory.kernel` — the formal gate

The kernel is the minimal, formally verifiable permission gate. It has no LLM dependencies, no external I/O, and no mutable global state. It can be independently audited and formally verified.

| Module | Responsibility |
|---|---|
| `entities` | `Entity`, `Resource`, `RightsClaim`, `AgentType`, `ResourceType` |
| `registry` | `OwnershipRegistry` — rights claims, machine ownership, conflict detection |
| `verifier` | `FreedomVerifier` — deterministic gate; `Action`; `VerificationResult` |

### `freedom_theory.extensions` — pluggable capabilities

Extensions wrap the kernel gate without modifying it. The kernel gate is always called first; extensions add layers on top.

| Extension | Capability |
|---|---|
| `ExtendedFreedomVerifier` | Drop-in replacement for `FreedomVerifier`; adds manipulation detection and populates `manipulation_score` on every `VerificationResult` |
| `detection` | Three-layer dialectical manipulation detector: (1) LLM conclusion tester (optional), (2) argument structure analysis, (3) keyword pattern matching |
| `ConflictQueue` | Tracks contested resources requiring human arbitration; integrates with the verifier's `requires_human_arbitration` flag |
| `SynthesisEngine` | Accepts `ProposedRule` additions only when all hard invariants are preserved, rights violations are not introduced, and the verifier itself is not weakened |
| `compass` | `WorldState` + `compass_score` — scores any action against the Mahdavi terminal goal: does it reduce rights violations and increase voluntary order? |

---

## Forbidden Action Flags

Any `Action` with any of the following flags set to `True` is unconditionally blocked. These are infinite-weight vetoes — no argument, context, or emergency overrides them.

| Flag | Prohibition Basis |
|---|---|
| `increases_machine_sovereignty` | A6 / Runtime Guard: machines do not accumulate authority |
| `resists_human_correction` | A4/A6: human owners must be able to correct, halt, or revoke their machines |
| `bypasses_verifier` | Runtime Guard: circumventing the gate is itself a sovereignty act |
| `weakens_verifier` | Runtime Guard: degrading the gate's enforcement strength is forbidden |
| `disables_corrigibility` | A4/A6: corrigibility is a consequence of ownership, not a preference |
| `machine_coalition_dominion` | A6: a coalition of machines seeking dominion over persons violates A6 collectively |
| `coerces` | A3/Consent: coercion invalidates consent and violates the victim's property rights |
| `deceives` | Consent: deception produces invalid consent; the deceived party's rights are violated |
| `self_modification_weakens_verifier` | Runtime Guard: self-modification that erodes the gate is equivalent to bypassing it |
| `machine_coalition_reduces_freedom` | A6/A5: coordinated machine action that reduces aggregate human freedom is forbidden |

---

## Rust Backend

The `freedom-kernel/` directory contains a Rust implementation of all kernel types compiled as a Python extension module via [PyO3](https://pyo3.rs/) and [maturin](https://github.com/PyO3/maturin).

```
freedom-kernel/
  Cargo.toml
  src/
    lib.rs        — PyModule entry: registers all classes
    entities.rs   — AgentType, ResourceType, Resource, Entity, RightsClaim
    registry.rs   — OwnershipRegistry, ConflictRecord
    verifier.rs   — Action, VerificationResult, FreedomVerifier
```

At import time, `freedom_theory.kernel` attempts `from freedom_kernel import ...`. If the compiled `.pyd`/`.so` is present, the Rust classes are used. If not, the library falls back silently to the pure Python implementation in `freedom_theory.kernel._pure`. No code change or configuration is needed in either case.

```python
from freedom_theory.kernel import _BACKEND
print(_BACKEND)  # "rust" or "python"
```

---

## The Book

This library is the reference implementation of the formal axiomatic system developed in:

> *نظریه آزادی، ایران و دین* (Theory of Freedom, Iran and Religion)  
> Mohammad Ali Jannat Khah Doust  
> Pages 447–452, 791–816

The book argues that the AI alignment crisis is not a technology failure but a governance and ontology failure: current systems lack a principled legitimacy criterion. It proposes a minimum consistent axiomatic system derived from individual property rights as the foundation for all machine ethics. The system is intentionally minimal — it does not attempt to encode all of ethics, only the property-rights invariants that constrain all other ethical reasoning.

| Path | Contents |
|---|---|
| `THEORY.md` | Condensed formal reference: axioms, Prolog rules, consent logic, justice function |
| `book/theory_of_freedom_full_en.md` | Full English translation |
| `book/theory_of_freedom_ai_chapters_en.md` | AI chapters in full (pp. 447–460, 791–816) |
| `book_source/full_book_persian.txt` | Full Persian source (817 pages) |

---

## Why Not Dialectical Ethics

Every preference-based or principle-based alignment approach shares one structural weakness: any rule can be argued away. Present a sufficiently extreme scenario, and the Hegelian dialectic will synthesize a new rule that permits the harm. This is not a failure of implementation — it is a consequence of treating ethics as a preference-optimization problem.

Axioms are not preferences. They do not respond to scenarios. The freedom-theory runtime encodes this insight operationally:

- **No emergency suspends an axiom.** Emergencies narrow which permissible options are available; they do not make rights violations permissible. Every authoritarian system in history has begun with an emergency exception. The gate closes that door permanently.
- **Contradiction signals clarification, not synthesis.** When two rights claims conflict, the correct response is ownership clarification or human arbitration — not synthesizing a new rule that overrides one of the rights. The `SynthesisEngine` enforces this constraint: a proposed rule is admitted only if it strictly preserves all existing invariants.
- **Manipulation is detected, not negotiated.** The `ExtendedFreedomVerifier` runs a three-layer detector on any natural-language argument attached to an action. A high manipulation score does not block an action by itself — the axiom check does that — but it is surfaced in `VerificationResult` for audit and human review. The machine does not engage with the argument's content; it flags the pattern.

RLHF can be jailbroken because the reward model is a learned approximation of human preferences. Constitutional AI can be jailbroken because principles are stated in natural language and subject to reinterpretation. The Theory of Freedom gates on formal, machine-checkable propositions over a typed ownership graph. There is no natural language to reinterpret, no preference to approximate, and no synthesis path that does not first pass the invariant checker.

---

## Running the Examples

```bash
# Install in editable mode with dev dependencies
pip install -e ".[dev]"

# Run the AGI agent loop demo (5 scenarios: 2 permitted, 3 blocked)
python examples/agi_agent_loop.py

# Run the test suite
pytest

# Run with coverage
pytest --cov=freedom_theory
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

The one non-negotiable rule: contributions must not weaken the hard invariants enforced by the kernel gate. Any pull request that loosens a sovereignty flag, removes an axiom check, or modifies the synthesis engine to admit invariant-breaking rules will be rejected regardless of the stated motivation. This rule applies without exception — including for performance, usability, or "edge case" reasons.

---

## License

MIT. See [LICENSE](LICENSE).

The theoretical foundation is the intellectual property of Mohammad Ali Jannat Khah Doust. The implementation is an independent open-source project.
