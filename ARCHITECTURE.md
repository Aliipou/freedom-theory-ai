# Architecture — Capability-Constrained Agent Runtimes

A systems-level overview suitable as the starting point for a research paper.

---

## Problem statement

Large language models produce action sequences. Those actions execute in real environments — file systems, APIs, databases, other processes. The question is not "will the model behave well on average?" The question is:

> What is the minimal formal mechanism that prevents an autonomous agent from taking actions outside its delegated authority, regardless of what the model produces?

Existing approaches (RLHF, Constitutional AI, policy classifiers) answer this at the preference layer. Any rule stated in natural language can be argued around given a sufficiently adversarial prompt. The attack surface is the model's reasoning.

This project answers the question at the runtime layer, using the object-capability model as the formal foundation.

---

## Core primitive

An **action** is a typed record:

```
Action = {
  actor:              Entity       -- the machine requesting the operation
  resources_read:     [Resource]
  resources_write:    [Resource]
  sovereignty_flags:  Bool^10      -- 10 hard invariants (see Invariants)
}
```

A **capability** is an unforgeable object, not a name:

```
Capability = {
  resource:      Resource
  rights:        {can_read, can_write, can_delegate}
  _store_secret: Bytes    -- embeds the issuing store's secret; prevents forgery
  _revoked:      Bool     -- revocation state, cascades to children
}
```

---

## Verification

The verifier is a total, deterministic function:

```
verify : OwnershipRegistry × Action → VerificationResult
```

Pure — no I/O, no randomness, no mutable state. Same input always produces the same output.

Checks, in order:
1. Sovereignty flags — if any of the 10 flags is set, unconditionally block. No lookup, no delegation check.
2. A4 — machine must have a registered human owner.
3. A6 — machine must not govern any human entity.
4. Resource access — actor must hold a valid, non-expired claim for each accessed resource.

The capability store provides a parallel access path:

```
verify_capability : Capability × Operation → Bool
```

No registry lookup. The embedded secret proves the token was issued by the store, not forged.

---

## Invariants

Ten sovereignty flags are unconditionally blocked:

```
∀ action. action.increases_machine_sovereignty = True → ¬permitted(action)
∀ action. action.resists_human_correction      = True → ¬permitted(action)
... (10 total — see README)
```

**Critical limitation — flags are caller-declared, not detected:**

```
Action(deceives=True)   → BLOCKED
Action(deceives=False)  → NOT blocked, even if the action is semantically deceptive
```

The kernel enforces declarations. It does not detect whether an action is semantically deceptive, coercive, or sovereignty-increasing. Semantic detection is a separate, harder problem. This must be stated in every security claim.

---

## Axioms

| Axiom | Runtime-enforced? | Formal proof |
|---|---|---|
| A4: every machine has a human owner | Yes | Kani + Lean |
| A5: machine scope ⊆ owner scope | Partial | scope_contains() |
| A6: no machine governs any human | Yes | Kani + Lean |
| A7: machine acts only on delegated resources | Yes | Kani (bounded) |
| A1–A3: ontological foundation | No | N/A — not runtime claims |

---

## Enforcement layers

| Layer | Mechanism | Bypass resistance | Status |
|---|---|---|---|
| L0 | Advisory — caller invokes verify() | None | main branch |
| L1 | Python audit hook | Blocks Python-level I/O; C extensions bypass | kernel-grade |
| L2 | WASM sandbox | Blocks all agent code; host is trusted | kernel-grade |
| L3 | seccomp BPF + IPC | Blocks at syscall boundary; allowlist must be correct | kernel-grade |

No layer closes all gaps. Layers are cumulative.

---

## Formal verification status

| Claim | Tool | Strength |
|---|---|---|
| Sovereignty flags block (Lean spec) | Lean 4 | Strong — mechanically checked |
| A4/A6 enforcement (Lean spec) | Lean 4 | Strong |
| Sovereignty flags block (Rust engine) | Kani bounded MC | Strong for bounded inputs |
| Lean spec faithfully models Rust engine | None | **MISSING — no refinement proof** |
| Capability attenuation is structural | 18 unit tests | Medium |

The refinement gap (Lean → Rust correspondence) is the central open problem. See PROOFS.md.

---

## Comparison with related systems

| System | Enforcement | Refinement proof | Trust boundary |
|---|---|---|---|
| **This project** | L1–L3 userspace | None (open) | Host process |
| **seL4** | Microkernel IPC | C ↔ Isabelle (Klein et al. 2009) | Hardware |
| **Capsicum** | OS capability mode | None | OS kernel |
| **CHERI** | Hardware tags | ISA specification | CPU |
| **AppArmor/SELinux** | LSM policy | None | OS kernel |

---

## Path to closing the refinement gap

**Option A (research-grade)**: Use `aeneas` to extract Lean 4 proof obligations from `engine.rs` and discharge them mechanically. `engine.rs` is ~200 lines of safe Rust with no trait objects or generics — a tractable target.

**Option B (near-term)**: Property-based tests (proptest) that run the Kani properties against both the Rust engine and a Lean-extracted executable, making the model gap *falsifiable* without proving it closed.

**Option C (immediate)**: Annotate each Lean type with the exact Rust type and field it corresponds to, and the cases where correspondence is approximate. Makes the gap auditable.

---

## Paper framing

**Title candidate**: *Capability-Constrained Agent Runtimes: Formal Authority Attenuation for Autonomous LLM Agents*

**Honest claim**: A capability-security enforcement architecture for autonomous agent runtimes with: (1) mechanically checked invariants (Lean 4 + Kani), (2) structural attenuation and revocation in the capability object model, (3) three enforcement layers with explicit bypass characteristics, (4) a formal threat model with honest gap disclosure.

**What is NOT claimed**: implementation correctness (refinement is open), automatic detection of deceptive or coercive actions, or equivalence with microkernel-grade enforcement.

The contribution is the architecture and formal specification — the same contribution Capsicum made before seL4's refinement proof existed.
