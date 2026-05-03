# Formal Verification — What Is and Is Not Proved

**Branch**: `kernel-grade`

This document precisely distinguishes what is mechanically verified from what is claimed but unverified. Reading this before the Lean or Kani files is strongly recommended.

---

## What exists

### Kani bounded model-checking harnesses (Rust)

Five harnesses in `freedom-kernel/src/kani_proofs.rs`:

| Harness | What it checks |
|---|---|
| `prop_forbidden_flags_always_block` | For all `ActionWire` inputs where any sovereignty flag is set, `verify()` returns `permitted = false` |
| `prop_ownerless_machine_blocked` | For all `ActionWire` with actor kind = MACHINE and empty `machine_owners`, `verify()` returns `permitted = false` |
| `prop_machine_governs_human_blocked` | For all actions where a machine governs a human, `verify()` blocks |
| `prop_public_resource_read_permitted` | For all actions reading a public resource with no other violations, `verify()` permits |
| `prop_write_denied_without_claim` | For all actions writing a resource with no write claim, `verify()` denies |

**What Kani actually proves**: For all inputs within the bounded unfolding depth, the property holds on the *Rust source code*. This is a check on the actual implementation, not a model.

**What Kani does not prove**: Properties outside the bounded depth, or properties about the PyO3 wrapper, Python fallback, or FFI layer.

---

### Lean 4 specification (`formal/FreedomKernel.lean`)

Five lemmas:

| Lemma | What it proves |
|---|---|
| `sovereignty_always_blocks` | In the formal model: if any sovereignty flag is set, `permitted` returns `false` |
| `permitted_decidable` | `permitted` is a total computable function — it always terminates |
| `ownerless_machine_blocked` | In the formal model: ownerless machine → not permitted |
| `attenuationHolds` | Stated as a Prop (not yet fully proved mechanically) |
| `public_read_permitted` | In the formal model: public resource read → permitted |

**What the Lean proofs actually prove**: The *Lean specification* (a model of the kernel) satisfies these properties. The Lean types are written to mirror the Rust/Python types, but this mirroring is by hand — it is not mechanically checked.

**Critical gap — no refinement proof**: There is no proof that the Rust implementation faithfully *refines* the Lean specification. This is the central unsolved problem in formal verification of systems software.

---

## The refinement gap — why it matters

seL4's key formal contribution (Klein et al., SOSP 2009; Sewell et al., PLDI 2013) was a *refinement proof*: a machine-checked proof that the C implementation is a faithful refinement of the abstract Isabelle/HOL specification. Every observable behavior of the C code is a behavior permitted by the specification.

The gap this project has is the same gap that every formal methods project faces before completing the refinement proof:

```
Lean spec  satisfies  properties      ✓ (proved in FreedomKernel.lean)
Rust impl  satisfies  properties      ✓ (bounded — Kani harnesses)
Lean spec  faithfully models Rust     ✗ (not proved — hand-written correspondence)
```

If the Lean model has a type that does not faithfully capture the Rust type, the Lean proof proves a property of a different system. This is the "model gap" problem.

**Concrete example of where the gap could bite**:

The Lean `Entity` type is:
```lean
structure Entity where
  name : String
  kind : AgentKind
```

The Rust `EntityWire` type is:
```rust
pub struct EntityWire { pub name: String, pub kind: String }
```

`kind` in Lean is an enum; in Rust it is a `String` compared with `== "MACHINE"`. If an attacker passes `kind = "machine"` (lowercase), the Lean proof still holds (enum comparison is structural) but the Rust check `actor.kind == "MACHINE"` fails silently — the machine is treated as human, bypassing A4.

This is a real class of bug that refinement proofs catch.

---

## What "formally verified" means for this project right now

**Accurate claim**: The abstract specification has mechanically checked proofs of the stated properties. The Rust implementation has bounded model-checking harnesses that verify the same properties for all bounded inputs.

**Inaccurate claim**: The implementation is formally verified. (This would require a refinement proof.)

The README uses the phrase "formally proved" in the context of the Lean lemmas. This is accurate — the Lean lemmas are proved. It does not mean the Rust implementation is proved correct.

---

## Path to closing the refinement gap

### Option A — Lean4 + Lean-Rust bridge (research-grade, multi-year)

Write the Rust engine in a Lean-verifiable subset and use Lean's code generation, or use a tool like `aeneas` (which extracts Lean proofs from Rust programs via `charon`). This is the seL4 approach: write the implementation in a verifiable way, then prove the refinement.

- Tooling: `aeneas` (Aeneas: Rust programs to Lean 4 proofs, Ho et al. 2022)
- Feasibility: `engine.rs` is ~200 lines of safe Rust — a tractable target for `aeneas`
- Status: planned research direction

### Option B — Property-based testing as a bridge (practical, near-term)

Use `proptest` or `quickcheck` to run the Kani properties as randomized tests against both the Lean-extracted executable and the Rust binary. If both agree on all sampled inputs, the correspondence is empirically validated (not proved, but falsifiable).

```rust
// proptest in engine_tests.rs
proptest! {
    #[test]
    fn lean_rust_sovereignty_agreement(flags in arb_sovereignty_flags()) {
        let rust_result = engine::verify(&registry, &action_with(flags));
        // Compare against reference implementation extracted from Lean
        assert_eq!(rust_result.permitted, lean_reference::verify(flags));
    }
}
```

- Status: planned (K5 phase)

### Option C — Annotate the correspondence explicitly (immediate)

For each Lean type, add a comment in the Lean file naming the exact Rust type and field it corresponds to, and the cases where the correspondence is approximate. This makes the gap explicit and auditable.

- Status: should be done now

---

## Summary table

| Claim | Evidence | Strength |
|---|---|---|
| Sovereignty flags block in the formal model | Lean proof | Strong — mechanically checked |
| Sovereignty flags block in the Rust engine | Kani harnesses | Strong — bounded model checking |
| Lean model faithfully represents Rust engine | Hand inspection | Weak — not mechanically checked |
| Python fallback matches Rust behavior | Python test suite | Weak — behavioral tests only |
| No panic in engine.rs for any input | Partial — `.unwrap()` removed | Medium — not proved exhaustively |
| Capability attenuation is structural | Code review + 18 tests | Medium — tests, not proofs |
