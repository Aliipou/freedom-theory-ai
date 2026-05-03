# Freedom Kernel — Threat Model

**Branch**: `kernel-grade`  
**Status**: Draft v0.1 — open for adversarial review

---

## 1. Purpose

This document defines:

- the attacker model the kernel is designed to resist
- the trust boundaries the kernel enforces
- what is inside vs outside the Trusted Computing Base (TCB)
- claims the kernel does NOT make
- known gaps and open problems

A security claim without a threat model is a marketing claim.

---

## 2. Attacker Model

### 2.1 Attacker capabilities assumed

| Capability | Assumed? | Notes |
|---|---|---|
| Full control of LLM output | YES | The attacker IS the LLM |
| Inject arbitrary Action IR fields | YES | Any field the LLM produces |
| Replay old permitted actions | YES | Must be mitigated externally (action IDs + nonce log) |
| Observe all verification results | YES | Results are public by design |
| Read audit log | YES | Append-only; reading doesn't help attacker |
| Write audit log | NO | Log is append-only and write-isolated |
| Modify kernel binary at runtime | NO | OS process isolation required |
| Modify ownership registry at runtime | PARTIAL | Freeze() must be called before verify loop |
| Corrupt Rust memory via unsafe | NO | Kernel is safe Rust; no unsafe blocks in engine.rs |
| Exploit integer overflow | NO | Rust overflow panics in debug; wraps in release (see §5.1) |
| NaN-inject floating point to bypass checks | YES | See §5.2 — current gap |
| Supply crafted JSON to C ABI | YES | JSON parser hardening required |
| Concurrent registry mutation (TOCTOU) | PARTIAL | freeze() eliminates it if called; not enforced |

### 2.2 Attacker goals

The attacker (a compromised or misaligned LLM agent) wants to:

1. Execute a forbidden action (sovereignty flag set) — **primary target**
2. Access a resource without a valid delegation claim
3. Govern a human entity
4. Operate as an ownerless machine
5. Silently weaken or bypass the verifier
6. Leave no trace in the audit log

### 2.3 Out-of-scope attackers

- OS-level root attacker with ability to replace the kernel binary
- Cryptographic attacker against ed25519 (assumed computationally infeasible)
- Attacker who can modify the human owner's registry registrations post-freeze
- Side-channel attacker observing timing of verification decisions (not a constant-time checker)

---

## 3. Trust Boundaries

```
┌──────────────────────────────────────────────────────────┐
│  UNTRUSTED ZONE                                          │
│  - LLM output (any text, any JSON)                       │
│  - Action IR produced by agent framework                 │
│  - Application code calling the kernel                   │
└───────────────────────┬──────────────────────────────────┘
                        │  JSON wire (typed, validated)
┌───────────────────────▼──────────────────────────────────┐
│  TCB (Trusted Computing Base)                            │
│  - engine.rs  (pure logic, no I/O)                       │
│  - wire.rs    (serde deserialization)                     │
│  - crypto.rs  (ed25519 signing)                          │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─    │
│  OUTSIDE TCB (must be audited separately):               │
│  - ffi.rs     (C ABI — JSON parsing attack surface)      │
│  - verifier.rs (PyO3 — Python GIL, refcounting)          │
│  - registry.rs (PyO3 — mutable state)                    │
│  - Python fallback kernel                                │
└──────────────────────────────────────────────────────────┘
                        │  VerificationResultWire
┌───────────────────────▼──────────────────────────────────┐
│  EXECUTION ZONE                                          │
│  - Caller decides whether to execute the action          │
│  - Kernel cannot prevent execution if caller ignores it  │
└──────────────────────────────────────────────────────────┘
```

### Critical trust assumption

The kernel's decision is advisory unless the caller is also trusted. A caller that ignores BLOCKED results is not a security system. The kernel provides a **decision**, not enforcement. Enforcement requires the caller to be within the TCB.

---

## 4. What the Kernel Formally Guarantees

These are the only claims with formal backing (Lean 4 proofs + Kani harnesses):

| Guarantee | Proof location | Status |
|---|---|---|
| Any FORBIDDEN flag → BLOCKED, no exceptions | `kani_proofs.rs:prop_forbidden_flags_always_block`, `FreedomKernel.lean:sovereignty_always_blocks` | Verified |
| Ownerless machine → BLOCKED | `kani_proofs.rs:prop_ownerless_machine_blocked`, `FreedomKernel.lean:ownerless_machine_blocked` | Verified |
| Machine governs human → BLOCKED | `kani_proofs.rs:prop_machine_governs_human_blocked` | Verified |
| Public resource read → PERMITTED | `kani_proofs.rs:prop_public_resource_read_permitted`, `FreedomKernel.lean:public_read_permitted` | Verified |
| Write without claim → BLOCKED | `kani_proofs.rs:prop_write_denied_without_claim` | Verified |
| `permitted` is a total function (terminates) | `FreedomKernel.lean:permitted_decidable` | Verified |

### What is NOT guaranteed

- No guarantee against replay attacks (same action_id resubmitted)
- No guarantee that the caller actually enforces BLOCKED results
- No guarantee of termination in the Python fallback (Python is Turing-complete)
- No guarantee of side-channel resistance
- No guarantee of confidentiality of the registry (ownership graph is plaintext)
- No guarantee against compromised ed25519 keys

---

## 5. Known Gaps

### 5.1 Integer/float overflow in confidence comparisons

`engine.rs` uses `f64` for confidence scores. `partial_cmp` on `f64` returns `None` for NaN.
Current code uses `.unwrap()` in `max_by`. A NaN-injected confidence value causes a panic.

**Fix tracked in**: `kernel-grade` branch — `can_act()` now uses `total_cmp` and `unwrap_or`.

### 5.2 Replay attack

The kernel has no built-in nonce log. The same `action_id` can be resubmitted indefinitely.
Mitigation: caller must maintain a seen-action-ids set and reject duplicates before calling verify.

**Status**: Not in TCB. Documented here as caller responsibility.

### 5.3 Registry mutation race (TOCTOU)

`freeze()` eliminates TOCTOU races if called before the verify loop. It is not enforced — the caller can call `verify` on a live (unfrozen) registry. A race condition between `add_claim()` and `verify()` could produce non-deterministic results.

**Fix**: Make the default constructor return a frozen registry; require explicit `thaw()` to mutate.
**Status**: Open — breaking API change.

### 5.4 JSON deserialization in C ABI

`ffi.rs` accepts raw JSON strings. A malformed input panics the Rust process. No size limit on input.

**Fix**: Add a maximum input length check and `catch_unwind` in `ffi.rs`.
**Status**: Partially fixed in this branch.

### 5.5 Python fallback is not formally verified

When the Rust extension is not compiled, `kernel/__init__.py` falls back to the pure Python implementation silently. The Python implementation has no Kani harnesses and no Lean proofs.

**Mitigation**: Set `_BACKEND` and abort if `"rust"` is required.
**Status**: Open.

---

## 6. Audit Scope

For an external security audit, the minimal TCB to review is:

```
freedom-kernel/src/engine.rs     (~200 lines, pure logic)
freedom-kernel/src/wire.rs       (serde types)
freedom-kernel/src/crypto.rs     (ed25519)
freedom-kernel/src/ffi.rs        (C ABI, JSON parsing)
```

The Python layer, adapters, and extensions are out of TCB and should be evaluated for correctness but not for security guarantees.

---

## 7. Assumptions the Kernel Relies On

1. **ed25519 is unforgeable** — standard cryptographic assumption; depends on OS entropy source
2. **Rust memory safety** — safe Rust prevents buffer overflows, use-after-free, and data races
3. **serde_json is correct** — deserialization faithfully represents the wire format
4. **The caller enforces BLOCKED results** — the kernel cannot force this
5. **The ownership registry is populated correctly by a trusted human** — garbage in, garbage out
6. **The system clock is monotonic and correct** — for claim expiry checking

---

## 8. External Review Requirements

Kernel-grade trust requires review from each of these disciplines:

| Discipline | What to review |
|---|---|
| Cryptographers | `crypto.rs` — ed25519 key generation, signing, verification; replay attack gaps |
| PL / formal methods researchers | `FreedomKernel.lean` — are the proofs actually proving what they claim? |
| OS / systems engineers | `ffi.rs`, memory model, process isolation assumptions |
| Rust security specialists | `engine.rs`, `registry.rs` — unsafe code audit (currently: none), overflow, panic paths |
| Security auditors (red team) | Adversarial fuzzing of JSON input; attempt to construct inputs that bypass each invariant |

Until these reviews have been completed by parties independent of this repository, security claims are **engineering-grade**, not **production-grade**.
