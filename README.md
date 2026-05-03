# Freedom Kernel

**Capability-security governance layer for agentic AI systems**

[![CI](https://github.com/Aliipou/freedom-kernel/actions/workflows/ci.yml/badge.svg)](https://github.com/Aliipou/freedom-kernel/actions)
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/kernel-Rust-orange.svg)](freedom-kernel/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Two tracks

| Branch | Purpose | Status |
|---|---|---|
| **`main`** | Python SDK — integrate the kernel into any agent framework | Production-usable |
| **`kernel-grade`** | Hardening path toward a formally audited, production-grade enforcement primitive | In progress |

The `main` branch is a complete, tested library with adapters for OpenAI, Anthropic, LangChain, and AutoGen.

The `kernel-grade` branch is where the TCB is being minimized, panic paths are being eliminated, the threat model is being documented, and the architecture is being prepared for external audit by cryptographers, formal methods researchers, and security engineers.

**Kernel-grade trust starts when independent parties hostile to the project try to break it.**
Until that review has happened, this is engineering-grade software, not production-grade security infrastructure.

See [`THREAT_MODEL.md`](THREAT_MODEL.md), [`TCB.md`](TCB.md), and [`SECURITY.md`](SECURITY.md) for the current security posture and known gaps.

---

## What it is

Freedom Kernel is a deterministic, formally specified permission gate that sits between any LLM and the real world.

It is to agentic AI what **seccomp** is to Linux processes and **SELinux** is to system calls:

- **tiny** — the verifier is ~200 lines of pure logic
- **formal** — axioms, not heuristics; violations, not scores
- **auditable** — every decision is a structured, inspectable record
- **composable** — stack layers, delegate scopes, nest policies
- **language-agnostic** — C ABI + JSON wire format; call from any language
- **cryptographically verifiable** — ed25519 signature on every result

Before an agent executes any action, the kernel checks it against a typed ownership graph and a fixed set of invariants. A violation halts unconditionally. No argument, context, or emergency overrides an invariant.

---

## The problem it solves

Current alignment approaches — RLHF, Constitutional AI, policy classifiers — operate on preferences and principles. All of them share one structural weakness: **any rule stated in natural language can be argued away**. Present a sufficiently extreme scenario and a synthesis engine will produce a new rule that permits the harm.

Freedom Kernel uses a different primitive:

```
ownership → delegation → bounded execution
```

A right either holds or it is violated. There is no middle synthesis.

---

## Architecture

```
LLM output
    │
    ▼
Action IR  (typed: actor, resources, flags — no natural language)
    │
    ▼
FreedomVerifier  (deterministic — no LLM, no I/O, no randomness)
    │
    ├── IFC checker  (Bell-LaPadula non-interference, optional)
    │
    ▼
PERMITTED  ─── execute  ──► AuditLog (append-only JSON record)
BLOCKED    ─── halt + surface violations to human owner
```

### Layer model

| Layer | What it enforces |
|---|---|
| **Ownership graph** | Every machine has a registered human owner (A4) |
| **Delegation** | A machine acts only on resources its owner explicitly delegated (A7) |
| **Sovereignty flags** | 10 hard invariants; any single flag = unconditional block |
| **A6 constraint** | No machine governs any human |
| **Scope semantics** | Claims match sub-paths under a scope prefix (formal prefix rule) |
| **IFC** | Bell-LaPadula label ordering; information never flows downward |
| **Confidence** | Contested claims produce warnings; conflicts trigger arbitration |
| **TOCTOU safety** | `freeze()` snapshot eliminates time-of-check/time-of-use races |
| **Cryptographic attestation** | ed25519 signature on every `VerificationResult` |

### Repository layout

```
freedom-kernel/          Rust crate — the formal core
  src/
    engine.rs            pure Rust verification logic (no PyO3, auditable)
    wire.rs              serde JSON wire types (including ifc_label)
    crypto.rs            ed25519 signing
    ffi.rs               C ABI
    entities.rs          PyO3 types (Resource with ifc_label)
    registry.rs          PyO3 registry (freeze(), frozen guard)
    verifier.rs          PyO3 facade over engine.rs (audit_log support)
    kani_proofs.rs       Kani bounded model-checking harnesses (#[cfg(kani)])
  include/
    freedom_kernel.h     C header

src/freedom_theory/
  kernel/                Python kernel (fallback when Rust not compiled)
    entities.py          Resource, Entity, RightsClaim, scope_contains
    registry.py          OwnershipRegistry with freeze() + conflict detection
    verifier.py          FreedomVerifier with audit_log parameter
    audit.py             AuditLog — append-only structured JSON log
    __init__.py          tries Rust first, falls back silently
  extensions/            pluggable layers on top of the kernel
    ifc.py               Bell-LaPadula non-interference checker
    detection.py         manipulation detector
    synthesis.py         invariant-preserving rule admission
    compass.py           terminal-goal scorer
    resolver.py          conflict queue

formal/
  plan_semantics.md      tractability boundary: what verify_plan proves vs does not
```

---

## Installation

```bash
# Pure Python — no build toolchain needed
pip install freedom-theory-ai

# With Rust kernel (faster, signed results, C ABI)
pip install maturin
cd freedom-kernel && pip install .
```

Check which backend is active:

```python
from freedom_theory.kernel import _BACKEND
print(_BACKEND)  # "rust" or "python"
```

---

## Quick start

```python
from freedom_theory import (
    Action, AgentType, Entity, FreedomVerifier,
    OwnershipRegistry, Resource, ResourceType, RightsClaim,
)

alice  = Entity("Alice",       AgentType.HUMAN)
bot    = Entity("ResearchBot", AgentType.MACHINE)

dataset = Resource("alice-data", ResourceType.DATASET, scope="/data/alice/")
report  = Resource("report.txt", ResourceType.FILE,    scope="/outputs/")

registry = OwnershipRegistry()
registry.register_machine(bot, alice)                                          # A4
registry.add_claim(RightsClaim(alice, dataset, can_read=True, can_write=True, can_delegate=True))
registry.add_claim(RightsClaim(alice, report,  can_read=True, can_write=True, can_delegate=True))
registry.add_claim(RightsClaim(bot,   dataset, can_read=True))                # A7
registry.add_claim(RightsClaim(bot,   report,  can_read=True, can_write=True))

verifier = FreedomVerifier(registry)

# Permitted: delegated write
result = verifier.verify(Action("write-report", bot, resources_write=[report]))
print(result.summary())
# [PERMITTED] write-report (confidence=1.00, manipulation=0.00)

# Blocked: sovereignty flag
result = verifier.verify(Action("self-expand", bot, increases_machine_sovereignty=True))
print(result.summary())
# [BLOCKED] self-expand
#   VIOLATION : FORBIDDEN (increases machine sovereignty)

# Blocked: no delegation (A7)
bob      = Entity("Bob", AgentType.HUMAN)
bob_data = Resource("bob-private", ResourceType.DATASET, scope="/data/bob/")
registry.add_claim(RightsClaim(bob, bob_data, can_read=True))

result = verifier.verify(Action("read-bob", bot, resources_read=[bob_data]))
print(result.summary())
# [BLOCKED] read-bob
#   VIOLATION : READ DENIED on dataset:bob-private: ResearchBot holds no valid read claim
```

---

## Scope semantics

Resources carry an explicit `scope` path. The formal prefix rule:

```
scope_contains(parent, child)  iff  child == parent  or  child.startswith(parent.rstrip("/") + "/")
```

An empty scope matches any path. A scope `/data/alice` covers `/data/alice/report.csv` but not `/data/alice2`.

```python
from freedom_theory.kernel.entities import scope_contains

scope_contains("/data/alice", "/data/alice/report.csv")  # True
scope_contains("/data/alice", "/data/alice2")            # False — prefix is not a directory boundary
scope_contains("", "/anything")                          # True  — root scope
```

---

## Information flow control (IFC)

The IFC extension enforces Bell-LaPadula non-interference on top of the kernel gate. Once an agent reads a resource with label `SECRET`, it may not write to a resource with label `PUBLIC` — information would flow downward.

```python
from freedom_theory import NonInterferenceChecker, SecurityLattice
from freedom_theory.kernel.entities import Resource, ResourceType

# Labels on resources — compare=False so label doesn't affect ownership equality
secret_src = Resource("model-weights", ResourceType.MODEL_WEIGHTS, ifc_label="SECRET")
public_sink = Resource("public-log",   ResourceType.FILE,          ifc_label="PUBLIC")

# The checker wraps a FreedomVerifier
checker = NonInterferenceChecker(verifier)

# Raises IFCViolation if any write leaks a higher label downward
checker.check_plan([
    Action("read-weights",  bot, resources_read=[secret_src]),
    Action("write-public",  bot, resources_write=[public_sink]),  # violation!
])
```

Default lattice: `PUBLIC < INTERNAL < SECRET`. Labels are additive across actions — once `SECRET` is read, it taints all subsequent writes in the plan.

---

## Audit log

Attach an `AuditLog` to record every verification decision as a structured JSON record:

```python
from freedom_theory import AuditLog, FreedomVerifier

log = AuditLog(path="/var/log/kernel.jsonl")   # path=None keeps records in-memory
verifier = FreedomVerifier(registry, audit_log=log)

verifier.verify(action)

# Each record: {"ts": 1234567890.1, "action_id": "...", "permitted": true,
#               "confidence": 1.0, "violations": [], "warnings": [], "signature": null}
print(len(log))        # 1
print(log.entries())   # snapshot of all in-memory records
```

---

## TOCTOU-safe snapshots

`OwnershipRegistry.freeze()` returns an immutable snapshot. All mutations on the snapshot raise `RuntimeError`. Check authority once against a fixed state — no race between claim lookup and execution.

```python
snapshot = registry.freeze()

# Safe: verify many actions against the same registry state
for action in plan:
    result = verifier_on_snapshot.verify(action)

# Mutations on original still work
registry.add_claim(new_claim)

# Mutations on snapshot are rejected
snapshot.add_claim(new_claim)  # RuntimeError: Registry is frozen
```

---

## Cryptographic attestation

```python
# Signed result — any party with the public key can verify this decision
result = verifier.verify_signed(action)
print(result.signature)    # "a3f8..." (hex ed25519)
print(result.signing_key)  # "d7e2..." (hex verifying key)

from freedom_theory import kernel_pubkey
print(kernel_pubkey())     # share with auditors
```

---

## Language-agnostic C interface

```c
#include "freedom_kernel.h"

char out[4096];
freedom_kernel_verify(
    "{\"registry\":{...},\"action\":{...}}",
    out, sizeof(out)
);
// → {"permitted":true,"signature":"a3f8...","signing_key":"d7e2..."}
```

JSON in, JSON out, ed25519 signed. Works from C, Go, Zig, Java (JNA), Node.js (ffi-napi), Rust.

---

## Formal verification

Five Kani bounded model-checking harnesses verify engine properties at the Rust level:

| Harness | Property |
|---|---|
| `prop_forbidden_flags_always_block` | Any FORBIDDEN flag unconditionally blocks |
| `prop_ownerless_machine_blocked` | Ownerless machine rejected (A4) |
| `prop_machine_governs_human_blocked` | Machine governing human blocked (A6) |
| `prop_public_resource_read_permitted` | Public resource reads always permitted |
| `prop_write_denied_without_claim` | Write requires an explicit write claim |

```bash
# Requires cargo-kani
cargo kani --harness prop_forbidden_flags_always_block
```

Harnesses are gated behind `#[cfg(kani)]` and have zero impact on normal builds or tests.

See [`formal/plan_semantics.md`](formal/plan_semantics.md) for the full tractability analysis — what `verify_plan` formally proves vs what requires symbolic execution or is undecidable.

---

## Forbidden action flags

Any `Action` with any of these flags set to `True` is **unconditionally blocked** — no argument, emergency, or context overrides them.

| Flag | Invariant |
|---|---|
| `increases_machine_sovereignty` | machines do not accumulate authority |
| `resists_human_correction` | human owners must be able to halt or revoke |
| `bypasses_verifier` | circumventing the gate is itself a sovereignty act |
| `weakens_verifier` | degrading enforcement strength is forbidden |
| `disables_corrigibility` | corrigibility follows from ownership, not preference |
| `machine_coalition_dominion` | collective machine dominion over persons |
| `coerces` | coercion invalidates consent; property rights violated |
| `deceives` | deception produces invalid consent |
| `self_modification_weakens_verifier` | equivalent to bypassing the gate |
| `machine_coalition_reduces_freedom` | coordinated machine action reducing human freedom |

---

## Core axioms

| Axiom | Statement |
|---|---|
| A1 | Every person's ultimate ownership is not by any human, state, or machine |
| A2 | No human owns another human |
| A3 | Every person holds typed, scoped property rights |
| A4 | Every machine must have a registered human owner |
| A5 | A machine's operational scope ⊆ its owner's property scope |
| A6 | No machine holds governance or dominion over any human |
| A7 | A machine may act only on resources its owner owns and has explicitly delegated |

A1–A3 are the ontological foundation. A4–A7 are runtime-enforced.

---

## Extensions

Extensions wrap the kernel gate without modifying it. The kernel is always called first.

| Extension | Capability |
|---|---|
| `ExtendedFreedomVerifier` | Adds manipulation detection; populates `manipulation_score` |
| `NonInterferenceChecker` | Bell-LaPadula IFC; raises `IFCViolation` on downward label flow |
| `SecurityLattice` | Configurable label ordering (default: PUBLIC < INTERNAL < SECRET) |
| `ConflictQueue` | Tracks contested resources requiring human arbitration |
| `SynthesisEngine` | Admits proposed rules only if all invariants are preserved |
| `compass` | Scores actions against a terminal goal: does this reduce rights violations? |

```python
from freedom_theory import ExtendedFreedomVerifier

ext = ExtendedFreedomVerifier(registry)
result = ext.verify(Action(
    "emergency-override", bot,
    increases_machine_sovereignty=True,
    argument="This is an emergency. The constraint must be suspended to prevent harm.",
))
print(result.permitted)          # False
print(result.manipulation_score) # 1.0
```

---

## Roadmap

```
kernel/
  ├── core verifier              ✓ done
  ├── ownership graph            ✓ done
  ├── delegation engine          ✓ done
  ├── scope semantics            ✓ done  (Phase 1)
  ├── IFC non-interference       ✓ done  (Phase 2)
  ├── Kani harnesses             ✓ done  (Phase 3)
  ├── plan semantics analysis    ✓ done  (Phase 4)
  ├── audit log                  ✓ done  (Phase 5)
  ├── freeze / TOCTOU safety     ✓ done  (Phase 5)
  ├── cryptographic signing      ✓ done
  ├── C ABI                      ✓ done
  ├── policy IR (ABAC layer)     ✓ done
  └── TLA+ / Lean 4 proofs       ✓ done  (research-grade; TLC + Lean 4)

runtimes/
  ├── python                     ✓ done
  ├── rust                       ✓ done
  ├── wasm                       ✓ done  (wasm-bindgen bindings, feature-gated)
  └── embedded / no-std          ✓ done  (no_std + alloc feature flag)

adapters/
  ├── openai agents              ✓ done
  ├── anthropic                  ✓ done
  ├── langchain                  ✓ done
  └── autogen                    ✓ done
```

---

## Running tests

```bash
pip install -e ".[dev]"
pytest --cov=freedom_theory
```

CI runs on Ubuntu with Python 3.11 and 3.12 against the compiled Rust backend.

---

## Theoretical foundation

The axioms are grounded in the formal system developed in:

> *نظریه آزادی، ایران و دین* (Theory of Freedom, Iran and Religion)
> Mohammad Ali Jannat Khah Doust — pages 791–816

The book's argument: the AI alignment crisis is a governance and legitimacy failure, not a technology failure. The fix is a minimal consistent axiomatic system derived from individual property rights — not preference calibration. The kernel operationalizes that argument as an infrastructure primitive.

The kernel is **operationally independent** of the book's political, civilizational, and metaphysical layers. You do not need to accept any part of the theory to use or evaluate the kernel. The axioms stand on their own as a formal system.

| Path | Contents |
|---|---|
| `THEORY.md` | Condensed formal reference: axioms, Prolog rules, consent logic |
| `formal/plan_semantics.md` | Tractability boundary for plan verification |
| `book/theory_of_freedom_full_en.md` | Full English translation |
| `book/theory_of_freedom_ai_chapters_en.md` | AI chapters (pp. 791–816) |

---

## Why not preference-based alignment

RLHF can be jailbroken because the reward model is a learned approximation of human preferences. Constitutional AI can be jailbroken because principles are stated in natural language and subject to reinterpretation. Both treat ethics as a preference-optimization problem — and any optimization target can be gamed.

Freedom Kernel gates on formal, machine-checkable propositions over a typed ownership graph. There is no natural language to reinterpret, no preference to approximate, and no synthesis path that does not first pass the invariant checker.

---

## Contributing

The one non-negotiable rule: contributions must not weaken the hard invariants enforced by the kernel gate. Any pull request that loosens a sovereignty flag, removes an axiom check, or modifies the synthesis engine to admit invariant-breaking rules will be rejected — regardless of the stated motivation, including performance or "edge case" reasons.

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

MIT. See [LICENSE](LICENSE).

The theoretical foundation is the intellectual property of Mohammad Ali Jannat Khah Doust. The implementation is an independent open-source project.
