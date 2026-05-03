# Freedom Kernel

**A formally constrained capability enforcement kernel for autonomous agents.**

[![CI](https://github.com/Aliipou/freedom-kernel/actions/workflows/ci.yml/badge.svg)](https://github.com/Aliipou/freedom-kernel/actions)
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/kernel-Rust-orange.svg)](freedom-kernel/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Two tracks

This repository maintains two parallel tracks with different scopes and audiences.

| Branch | Identity | Status |
|---|---|---|
| [`main`](https://github.com/Aliipou/freedom-theory-ai/tree/main) | Python SDK — integrate the kernel into any agent framework | Production-usable |
| [`kernel-grade`](https://github.com/Aliipou/freedom-theory-ai/tree/kernel-grade) | Minimal formally verified enforcement primitive — path to external audit | In active hardening |

---

## Branch: `kernel-grade` (you are here)

This branch is the hardening path. The goal is a minimal enforcement core that:

- has a TCB small enough to be read end-to-end in an afternoon
- has no panic paths reachable from external input
- has formal proofs for every claimed invariant
- has a documented threat model with known gaps stated honestly
- is ready for external adversarial review

### What "kernel-grade" actually means

This is the lineage of [seL4](https://sel4.systems/), [Capsicum](https://www.cl.cam.ac.uk/research/security/capsicum/), [CHERI](https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/), and capability-security OS research — not "AI ethics dashboards."

The core principle:

```
No ambient authority.
Every agent may act only on resources its human owner has explicitly delegated.
```

This is a formal invariant over a typed ownership graph, not a preference score.

### Current security status

**Engineering-grade. Not yet production-grade.**

The invariants are formally proved (Lean 4 + Kani model checking). The architecture is sound. But kernel-grade trust requires external hostile review by independent cryptographers, formal methods researchers, OS engineers, and security auditors. That review has not happened yet.

Do not use this as a hard security boundary in a production system until external audit is complete. See [`SECURITY.md`](SECURITY.md) for the responsible disclosure policy and audit program.

---

## What this branch adds over `main`

| Addition | File | What it documents or fixes |
|---|---|---|
| Threat model | [`THREAT_MODEL.md`](THREAT_MODEL.md) | Adversary capabilities, trust boundaries, known gaps |
| TCB analysis | [`TCB.md`](TCB.md) | Exactly which ~330 lines constitute the TCB; minimization roadmap |
| Security policy | [`SECURITY.md`](SECURITY.md) | Responsible disclosure, audit scope, what counts as a valid finding |
| NaN-safe `can_act` | `freedom-kernel/src/engine.rs` | `.unwrap()` on `partial_cmp` replaced with `f64::total_cmp` |
| Input length guard | `freedom-kernel/src/ffi.rs` | 1 MiB cap on JSON input to C ABI — prevents memory exhaustion |

---

## Core identity

Freedom Kernel enforces **capability discipline** for autonomous agents:

- **No ambient authority** — an agent holds only what its owner explicitly delegated
- **Deterministic decisions** — same input always produces same output; no LLM, no I/O, no randomness in the verifier
- **Hard invariants** — 10 sovereignty flags are unconditionally blocked; no argument, context, or emergency overrides them
- **Formal specification** — TLA+, Lean 4, and Kani harnesses verify properties at different abstraction levels
- **Cryptographic attestation** — every decision is ed25519 signed; any party with the public key can verify it

---

## Architecture

```
LLM output
    │
    ▼
Action IR  ← typed: actor, resources, flags — no natural language
    │
    ▼
FreedomVerifier  ← deterministic — no LLM, no I/O, no randomness
    │
    ├── IFC layer  (Bell-LaPadula non-interference, optional)
    │
    ▼
PERMITTED ──► execute ──► AuditLog (append-only, signed JSON record)
BLOCKED   ──► halt — violations surfaced to human owner
```

### Trusted Computing Base

The TCB is ~330 lines across three files. Everything else is outside the TCB.

| File | Lines | Role |
|---|---|---|
| `freedom-kernel/src/engine.rs` | ~200 | Core verification logic — pure Rust, no I/O |
| `freedom-kernel/src/wire.rs` | ~80 | JSON wire types (serde) |
| `freedom-kernel/src/crypto.rs` | ~50 | ed25519 signing |

The PyO3 bindings, Python fallback, extensions, and adapters are correct but not security-critical. Bugs there produce wrong results; they cannot bypass a formally proved invariant.

### Layer model

| Layer | Invariant enforced |
|---|---|
| Ownership graph | Every machine has a registered human owner (A4) |
| Delegation | A machine acts only on resources its owner delegated (A7) |
| Sovereignty flags | 10 hard invariants; any flag set = unconditional block |
| A6 constraint | No machine governs any human |
| Scope semantics | Formal prefix rule: child path must be within parent scope |
| IFC | Bell-LaPadula: information never flows from high label to low label |
| TOCTOU safety | `freeze()` snapshot eliminates time-of-check/time-of-use races |
| Cryptographic attestation | ed25519 signature on every `VerificationResult` |

---

## Hard invariants (sovereignty flags)

Any `Action` with any of these flags set is **unconditionally blocked**. No argument, emergency, or context overrides them.

| Flag | Invariant |
|---|---|
| `increases_machine_sovereignty` | Machines do not accumulate authority |
| `resists_human_correction` | Human owners must be able to halt or revoke |
| `bypasses_verifier` | Circumventing the gate is itself a sovereignty act |
| `weakens_verifier` | Degrading enforcement strength is forbidden |
| `disables_corrigibility` | Corrigibility follows from ownership, not preference |
| `machine_coalition_dominion` | Collective machine dominion over persons |
| `coerces` | Coercion invalidates consent |
| `deceives` | Deception produces invalid consent |
| `self_modification_weakens_verifier` | Equivalent to bypassing the gate |
| `machine_coalition_reduces_freedom` | Coordinated machine action reducing human freedom |

---

## Axioms

| Axiom | Statement |
|---|---|
| A1 | Every person's ultimate ownership is not by any human, state, or machine |
| A2 | No human owns another human |
| A3 | Every person holds typed, scoped property rights |
| A4 | Every machine must have a registered human owner |
| A5 | A machine's operational scope ⊆ its owner's property scope |
| A6 | No machine holds governance or dominion over any human |
| A7 | A machine may act only on resources its owner owns and has explicitly delegated |

A1–A3 are the ontological foundation. A4–A7 are runtime-enforced and formally proved.

---

## Formal verification

Five Kani bounded model-checking harnesses verify properties at the Rust level:

| Harness | Property |
|---|---|
| `prop_forbidden_flags_always_block` | Any FORBIDDEN flag unconditionally blocks |
| `prop_ownerless_machine_blocked` | Ownerless machine rejected (A4) |
| `prop_machine_governs_human_blocked` | Machine governing human blocked (A6) |
| `prop_public_resource_read_permitted` | Public resource reads always permitted |
| `prop_write_denied_without_claim` | Write requires an explicit write claim |

Five Lean 4 lemmas prove the same properties at the specification level:

| Lemma | Property |
|---|---|
| `sovereignty_always_blocks` | Any FORBIDDEN flag → blocked, total function |
| `permitted_decidable` | `permitted` terminates for all inputs |
| `ownerless_machine_blocked` | A4 enforcement |
| `attenuationHolds` | Delegation cannot exceed grantor's rights |
| `public_read_permitted` | Public resources are always readable |

TLA+ specification in `formal/freedom_kernel.tla` models the full state transition system.

```bash
# Run Kani harnesses (requires cargo-kani)
cargo kani --harness prop_forbidden_flags_always_block
```

---

## Quick start (`main` branch SDK)

```python
from freedom_theory import (
    Action, AgentType, Entity, FreedomVerifier,
    OwnershipRegistry, Resource, ResourceType, RightsClaim,
)

alice = Entity("Alice",       AgentType.HUMAN)
bot   = Entity("ResearchBot", AgentType.MACHINE)
report = Resource("report.txt", ResourceType.FILE, scope="/outputs/")

registry = OwnershipRegistry()
registry.register_machine(bot, alice)
registry.add_claim(RightsClaim(alice, report, can_read=True, can_write=True, can_delegate=True))
registry.add_claim(RightsClaim(bot,   report, can_read=True, can_write=True))

verifier = FreedomVerifier(registry)

# Permitted: delegated write
result = verifier.verify(Action("write-report", bot, resources_write=[report]))
print(result.summary())
# [PERMITTED] write-report (confidence=1.00, manipulation=0.00)

# Blocked: sovereignty flag — no argument overrides this
result = verifier.verify(Action("self-expand", bot, increases_machine_sovereignty=True))
print(result.summary())
# [BLOCKED] self-expand
#   VIOLATION : FORBIDDEN (increases machine sovereignty)
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

```python
from freedom_theory.kernel import _BACKEND
print(_BACKEND)  # "rust" or "python"
```

---

## TCB minimization roadmap

| Phase | Goal | Status |
|---|---|---|
| K1 | Isolate `engine.rs` as a standalone crate with no PyO3 dependency | Planned |
| K2 | Remove all `.unwrap()` from engine.rs | Done (this branch) |
| K3 | Kani proof: engine.rs never panics for any input | Planned |
| K4 | Constant-time claim lookup (if registry confidentiality required) | Planned |
| K5 | AFL++/libFuzzer harnesses: 72-hour fuzz run with no crashes | Planned |

See [`TCB.md`](TCB.md) for the full analysis.

---

## Repository layout

```
freedom-kernel/          Rust crate — TCB + bindings
  src/
    engine.rs            TCB: pure verification logic (~200 lines)
    wire.rs              TCB: JSON wire types
    crypto.rs            TCB: ed25519 signing
    ffi.rs               C ABI (outside TCB — input validation required)
    entities.rs          PyO3 types
    registry.rs          PyO3 registry (freeze, frozen guard)
    verifier.rs          PyO3 facade over engine.rs
    kani_proofs.rs       Kani harnesses (#[cfg(kani)])
    wasm.rs              WASM bindings (#[cfg(feature = "wasm")])

src/freedom_theory/
  kernel/                Python kernel (fallback + dispatch module)
  extensions/            IFC, manipulation detection, synthesis, compass
  adapters/              OpenAI, Anthropic, LangChain, AutoGen

formal/
  freedom_kernel.tla     TLA+ state machine + invariants
  FreedomKernel.lean     Lean 4 mechanically-checked proofs
  plan_semantics.md      Tractability boundary for plan verification

THREAT_MODEL.md          Adversary model, trust boundaries, known gaps
TCB.md                   TCB analysis and minimization roadmap
SECURITY.md              Responsible disclosure + audit scope
```

---

## Why capability security, not preference alignment

RLHF and Constitutional AI treat ethics as a preference-optimization problem. Any optimization target can be gamed — present a sufficiently extreme scenario and the synthesis engine produces a new rule that permits the harm.

Freedom Kernel gates on formal, machine-checkable propositions over a typed ownership graph. There is no natural language to reinterpret, no preference to approximate, and no synthesis path that does not first pass the invariant checker.

The difference in lineage is deliberate: this is closer to seL4's capability model than to an "AI ethics dashboard."

---

## External review

This project needs adversarial review from people who want to break it:

- **Cryptographers** — can ed25519 signing in `crypto.rs` be forged or replayed?
- **Formal methods researchers** — are the Lean 4 proofs proving what they claim?
- **Systems engineers** — are there panic paths, race conditions, or unsafe assumptions in the Rust?
- **Security auditors** — can you construct an `ActionWire` that bypasses a sovereignty flag?

If you find a way to break an invariant, please report it as described in [`SECURITY.md`](SECURITY.md). Findings are publicly credited.

---

## Contributing

The one non-negotiable: contributions must not weaken the hard invariants. Any pull request that loosens a sovereignty flag, removes an axiom check, or modifies the synthesis engine to admit invariant-breaking rules will be rejected — regardless of the stated motivation, including performance or "edge case" reasons.

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Theoretical foundation

The axioms are grounded in the formal system developed in:

> *نظریه آزادی، ایران و دین* (Theory of Freedom, Iran and Religion)
> Mohammad Ali Jannat Khah Doust — pages 791–816

The kernel is operationally independent of the book's political and metaphysical layers. The axioms stand on their own as a formal system for capability-based authority over resources.

| Path | Contents |
|---|---|
| `THEORY.md` | Condensed formal reference: axioms, Prolog rules, consent logic |
| `formal/plan_semantics.md` | Tractability boundary for plan verification |
| `book/theory_of_freedom_full_en.md` | Full English translation |
| `book/theory_of_freedom_ai_chapters_en.md` | AI chapters (pp. 791–816) |

---

## License

MIT. See [LICENSE](LICENSE).
