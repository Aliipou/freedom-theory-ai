# Security Policy

## Scope of security guarantees

Freedom Kernel makes **formal** security guarantees only for the Trusted Computing Base (TCB):
`freedom-kernel/src/engine.rs`, `wire.rs`, and `crypto.rs`.

The Python fallback, adapters, extensions, and framework integrations are **not** in the TCB.
Do not rely on them as a security boundary.

See [TCB.md](TCB.md) for the full TCB analysis and [THREAT_MODEL.md](THREAT_MODEL.md) for the attacker model.

## Current security status

This project is in **engineering-grade** state, not **production-grade** state.

The invariants are formally verified (Lean 4 + Kani). The architecture is sound.
But kernel-grade trust requires external hostile review by independent cryptographers,
formal methods researchers, OS engineers, and security auditors — none of which has
happened yet.

**Do not use this as a security boundary in a production system until external audit is complete.**

## Responsible disclosure

If you find a vulnerability in the TCB (a way to bypass a sovereignty flag, cause a panic
in `engine.rs`, or forge an audit signature), please report it privately:

- **Email**: nikzadpars@gmail.com
- **Subject**: `[freedom-kernel] Security vulnerability`
- **PGP**: Not yet set up — request key in email

Please include:
- Which invariant is bypassed
- A minimal reproducer (ActionWire JSON or Rust test)
- Your assessment of exploitability

We will acknowledge within 48 hours and aim to patch within 7 days.

## What we will accept as a valid finding

| Finding type | Counts as vulnerability? |
|---|---|
| Bypass of a sovereignty flag in `engine.rs` | YES — critical |
| A4/A6 enforcement bypass | YES — critical |
| Panic in `engine.rs` with crafted input | YES — DoS |
| Forged ed25519 signature | YES — critical |
| Audit log entries dropped or modified | YES — high |
| Python fallback behavior differs from Rust | YES — medium |
| Panic in ffi.rs with crafted JSON | YES — medium |
| Information leakage through timing | YES — low (see TCB.md §K4) |
| Behavior outside the formal model | YES — report for documentation |
| Policy IR or adapter bugs | NO — outside TCB |
| Extension layer bugs | NO — outside TCB |

## Known limitations (not vulnerabilities)

These are documented gaps, not undisclosed vulnerabilities:

- No replay attack prevention (document in THREAT_MODEL.md §5.2)
- `freeze()` is not enforced at construction time (THREAT_MODEL.md §5.3)
- Python fallback is not formally verified (THREAT_MODEL.md §5.5)
- No constant-time claim lookup (TCB.md §K4)

## External audit program

We are seeking volunteer reviewers from:

- Cryptography (ed25519 implementation review)
- Programming language theory / formal methods (Lean 4 proof review)
- Systems security (Rust safety audit, fuzzing)
- OS engineering (process isolation assumptions)

If you are a researcher in any of these areas and willing to review, please open an issue
tagged `audit-request` with your area of expertise.

All findings will be publicly credited (unless you prefer anonymity).
