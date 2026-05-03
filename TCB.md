# Trusted Computing Base (TCB) Analysis

**Branch**: `kernel-grade`

---

## What is the TCB?

The TCB is the set of software that must be correct for the security guarantees to hold.
If any part of the TCB has a bug, all guarantees are void.

The goal of kernel-grade software is to make the TCB as small as possible — small enough
that it can be exhaustively reviewed by independent parties.

---

## Current TCB Size

| File | Lines (approx) | Role |
|---|---|---|
| `freedom-kernel/src/engine.rs` | ~200 | Core verification logic |
| `freedom-kernel/src/wire.rs` | ~80 | JSON deserialization types |
| `freedom-kernel/src/crypto.rs` | ~50 | ed25519 signing |
| **Total TCB** | **~330 lines** | |

330 lines is auditable. The Linux kernel is 28 million lines. secp256k1 is ~3,500 lines.
For context, this is the right order of magnitude for a minimal security primitive.

---

## What is Outside the TCB

Outside the TCB means: bugs there can cause incorrect behavior or poor ergonomics,
but cannot compromise the formal invariants enforced by `engine.rs`.

| Component | Outside TCB? | Reason |
|---|---|---|
| `ffi.rs` | Partially | JSON parsing is an attack surface; output is fed to engine.rs |
| `verifier.rs` (PyO3) | YES | Thin wrapper; all logic delegated to engine.rs |
| `registry.rs` (PyO3) | YES | State management; correctness but not TCB |
| `kani_proofs.rs` | YES | Verification harnesses, not runtime code |
| `wasm.rs` | YES | Wrapper only |
| Python fallback (`kernel/*.py`) | YES | Not formally verified; use for dev only |
| Extensions (`extensions/*.py`) | YES | Layer on top of kernel; bugs there can't break kernel invariants |
| Adapters (`adapters/*.py`) | YES | Application-layer code |

---

## TCB Minimization Roadmap

### Phase K1 — Isolate engine.rs as a standalone crate (planned)

Currently `engine.rs` is one file inside the `freedom-kernel` crate that also contains
PyO3 bindings. The PyO3 dependency bloats the compilation unit even though `engine.rs`
never calls PyO3.

Proposed structure:

```
freedom-kernel-core/   (new — the TCB)
  src/
    engine.rs
    wire.rs
    crypto.rs
  Cargo.toml           (no PyO3, no wasm-bindgen, no serde features beyond core)

freedom-kernel/        (existing — PyO3 bindings over the core)
  Cargo.toml
  src/
    ffi.rs
    verifier.rs
    registry.rs
    entities.rs
    ...
  [depends on freedom-kernel-core]
```

Auditors review only `freedom-kernel-core/`. The binding layer is trusted to be correct
but is not in the security-critical path.

### Phase K2 — Remove all `.unwrap()` from engine.rs (in progress)

Any `.unwrap()` in `engine.rs` is a potential panic path. A panic in the engine is a
denial-of-service vulnerability — the verifier stops functioning.

Current status: `can_act()` uses `.unwrap()` on `partial_cmp` and `max_by`.
Fix: replace with `f64::total_cmp` (NaN-safe) and explicit fallback.

### Phase K3 — Formal claim: engine.rs never panics (planned)

Use Kani to prove the absence of panics for all reachable inputs. This requires:
- Removing all `.unwrap()` calls
- Proving that all loops terminate (trivial — no recursive calls)
- Proving that arithmetic operations don't overflow (f64 can't integer-overflow)

### Phase K4 — Constant-time comparisons for security-sensitive paths (planned)

The claim-lookup loop in `can_act()` leaks timing information about how many claims exist
and whether a match was found early. For adversaries with timing oracle access, this could
reveal registry structure.

For now, this is acceptable (the registry is assumed non-secret). If registry confidentiality
is required, constant-time lookup must be added.

### Phase K5 — Fuzzing infrastructure (planned)

AFL++ / libFuzzer harnesses against:
- JSON deserialization in `ffi.rs`
- The full `verify()` function with arbitrary `ActionWire` inputs
- Registry wire format

Goal: 72-hour fuzz run with no crashes before any production claim.

---

## Dependency Audit

Dependencies inside the TCB:

| Crate | Version | Purpose | Security notes |
|---|---|---|---|
| `serde` | 1.x | Serialization framework | No known vulnerabilities |
| `serde_json` | 1.x | JSON parsing | Attack surface in ffi.rs; input validation required |
| `ed25519-dalek` | 2.x | Ed25519 signatures | Audited by Cure53 (2022) |

Dependencies outside the TCB (binding layer only):

| Crate | Version | Purpose |
|---|---|---|
| `pyo3` | 0.22 | Python bindings |
| `wasm-bindgen` | 0.2 | WASM bindings (feature-gated) |

---

## Audit Instructions for External Reviewers

If you are an external reviewer:

1. **Start with `freedom-kernel/src/engine.rs`** — this is the only file that matters for
   security guarantees. Read every line. The verification logic is ~120 lines of pure Rust.

2. **Check `wire.rs`** — confirm that the serde types faithfully represent the claim/action
   model. Look for fields that could be omitted, defaulted to unsafe values, or type-confused.

3. **Check `crypto.rs`** — confirm that the signing key is fresh per-process, that verification
   uses the correct public key, and that there is no key reuse across unrelated operations.

4. **Review `ffi.rs`** — the C ABI accepts raw JSON. Check for:
   - Missing input length validation
   - Panic paths (missing `catch_unwind`)
   - Buffer truncation in the output

5. **Try to construct an ActionWire that bypasses a sovereignty flag** — if you find one,
   please open an issue or email the maintainer.

6. **Review the Lean 4 proofs** (`formal/FreedomKernel.lean`) — are the types a faithful
   model of the Rust types? Are the proofs proving what they claim to prove?
