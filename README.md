# Freedom Kernel

**Capability-constrained enforcement architecture for autonomous agent runtimes.**

[![CI](https://github.com/Aliipou/freedom-kernel/actions/workflows/ci.yml/badge.svg)](https://github.com/Aliipou/freedom-kernel/actions)
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/kernel-Rust-orange.svg)](freedom-kernel/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What this is — precisely

A **capability policy library** with enforcement hooks. Not yet a capability kernel.

| | This project | A capability kernel |
|---|---|---|
| Enforcement | Caller must invoke the verifier | The capability IS the access token |
| Confinement | Python audit hook + optional WASM/seccomp | OS-enforced, mandatory |
| Capability model | Process-local logical objects | Hardware tags (CHERI), kernel tables (seL4), FD rights (Capsicum) |
| Bypass | Ignore the result; C extensions; same-process code | Requires compromising the OS kernel |
| Claimed TCB | ~330 LOC in `engine.rs` | Formally proved implementation |

**Do not use this as a hard security boundary in production.**
The enforcement is not yet mandatory end-to-end. See the five known gaps below.

---

## Two branches

| Branch | Scope | Audience |
|---|---|---|
| [`main`](https://github.com/Aliipou/freedom-kernel/tree/main) | SDK — integrate the capability gate into agent frameworks | Developers building agent systems |
| [`kernel-grade`](https://github.com/Aliipou/freedom-kernel/tree/kernel-grade) | Hardening path toward a formally audited enforcement primitive | Security researchers, systems engineers |

---

## Five known gaps — read before trusting this

### Gap 1 — Effective attack surface is much larger than the stated TCB

The formal TCB is ~330 LOC (`engine.rs`, `wire.rs`, `crypto.rs`). The effective trusted runtime includes:

- Python interpreter and standard library
- PyO3 bindings between Rust and Python
- `OwnershipRegistry`, `Action`, `Entity` Python object model
- All serialization and deserialization paths
- Enforcement glue (`hooks.py`, `wasm_sandbox.py`)

A security researcher will correctly observe: the proof-carrying core is small, but the effective trusted runtime is enormous. A bug in PyO3 type conversion, Python object serialization, or the enforcement adapter layer can produce wrong results at the enforcement point even if `engine.rs` is correct. The 330-line number describes the formal verification scope, not the attack surface.

### Gap 2 — Declared flags are self-attestation, not detection

The action flags (`deceives`, `coerces`, `bypasses_verifier`, etc.) are **caller-declared**.

```python
Action(deceives=True)   # → BLOCKED
Action(deceives=False)  # → NOT BLOCKED, regardless of what the action actually does
```

A rational adversary will not set `deceives=True`. The kernel enforces declarations; it does not detect violations of them. These flags are **compliance annotations** — useful for framework-layer auditing and policy expression, not for adversarial enforcement. Treating them as security guarantees is wrong.

Detecting whether an action is semantically deceptive, coercive, or an authority amplification requires a learned classifier or a richer formal model of agent intent. That is a separate problem. This project does not solve it, and does not claim to.

### Gap 3 — Capability objects are process-local, not OS-level

The `CapabilityStore` / `Capability` model enforces unforgeability, attenuation, and revocation within a single process. It is **not** the same as:

- **CHERI**: capability is a hardware-tagged fat pointer; the CPU enforces it at every load/store
- **Capsicum**: capability is a kernel file descriptor with a right mask; the kernel enforces it at every syscall
- **seL4**: capability is an entry in a kernel-managed CNode table; the kernel enforces it at every IPC

What this project provides is **process-local logical unforgeability**: external Python code that does not know the store's secret cannot forge a valid `Capability` object. But any code running in the same process, including C extensions, ctypes, and native libraries, can bypass the Python object model entirely. The `__setattr__` guard prevents accidental mutation, not a motivated attacker with native access.

### Gap 4 — No formal refinement proof

The formal verification has two independent components that are not connected:

```
Lean spec  satisfies  stated properties    ✓  proved (FreedomKernel.lean)
Rust impl  satisfies  stated properties    ✓  bounded (Kani harnesses)
Lean spec  ↔  Rust impl                   ✗  not proved — hand-written correspondence
```

The Lean proofs prove properties of the Lean specification. The Kani harnesses check the Rust source. Neither proves that the Lean model faithfully represents the Rust implementation. If a Lean type does not match the corresponding Rust type, the Lean proofs prove properties of a different system. See [`PROOFS.md`](PROOFS.md) for the concrete example where this matters (`kind` enum vs `String` comparison).

Closing this gap requires a tool like `aeneas` (Lean 4 proofs from Rust programs). This is planned but not done.

### Gap 5 — seccomp handles syscalls; it does not handle everything else

The L3 seccomp profile blocks dangerous syscalls (`socket`, `execve`, `ptrace`, `mount`, `bpf`). It does not address:

- Logic bugs within allowed syscall paths
- Confused deputy attacks through shared file descriptors or shared memory
- IPC-based privilege escalation
- Filesystem namespace tricks
- Covert channels (timing, cache)

Real confinement at kernel grade requires `namespaces`, `Landlock`, `pledge`/`unveil`, microVM isolation, or a capability OS design. seccomp is one layer of a defense-in-depth stack, not a confinement primitive on its own.

---

## What this project does provide

Despite the gaps above, the system provides real, testable guarantees within its scope:

**Structural authority attenuation**: A capability derived from a read-only parent cannot have write rights. This is enforced by AND-logic in `Rights.attenuate()` and cannot be bypassed through the Python API. Verified by 18 tests and Kani harnesses.

**Revocation cascade**: `cap.revoke()` transitively revokes all derived capabilities via the parent's `_children` list. Verified by concurrent race tests and unit tests.

**Process-local unforgeability**: A `Capability` constructed outside `CapabilityStore` fails `verify_capability()` because it cannot possess the store's 256-bit random secret. Post-construction mutation of `_rights` and `_store_secret` raises `AttributeError` (found and fixed by adversarial testing — slot mutation was an exploitable vulnerability before the `__setattr__` guard).

**Policy-level flag enforcement**: If an action declares `deceives=True` or any other blocked flag, the verifier blocks it with no exceptions and no override. This holds in the Python fallback and is proved by Kani harnesses on the Rust engine.

**Mandatory Python-layer I/O mediation (L1)**: Once `CapabilityEnforcer.install()` is called, `sys.addaudithook` fires before every `open()`, `subprocess.Popen()`, and `socket.connect()`. This cannot be removed. It can be bypassed by C extensions and native code.

**Adversarially tested**: 29 tests across 9 attack categories — forgery, authority amplification, confused deputy, TOCTOU, revocation races, replay attacks, serialization attacks, monkey patching, privilege escalation. The test suite found one real bug (slot mutation) before external review.

---

## Enforcement layers

```
L0  advisory          caller invokes verify() voluntarily          main branch
L1  Python hook       sys.addaudithook — mandatory for Python I/O  this branch ✓
L2  WASM sandbox      agent runs in VM, host functions only        this branch (interface done)
L3  seccomp + IPC     syscall-level filter + verifier process      this branch (profile done)
```

### L1 — Python audit hook

```python
from freedom_theory.enforcement import CapabilityEnforcer

enforcer = CapabilityEnforcer(verifier, agent=bot)
enforcer.install()   # permanent — sys.addaudithook cannot be removed

open("secret.txt")   # PermissionError if bot has no read claim
```

Cannot be removed. Cannot block C extensions calling the OS directly.

### L2 — WASM sandbox

```python
from freedom_theory.enforcement import WasmAgentRunner

runner = WasmAgentRunner(verifier, agent=bot)
runner.load("agent.wasm")    # requires wasmtime-py
result = runner.call("run_task")
```

Agent code runs in a WASM VM. All OS access goes through verified host functions
(`freedom.read_file`, `freedom.write_file`, `freedom.http_get`).
C extensions and native libraries cannot load inside the VM.

### L3 — seccomp syscall filter (Linux only)

```python
from freedom_theory.enforcement.seccomp import install_agent_profile

install_agent_profile()   # irreversible; fork first if needed
```

Or generate a Docker/OCI seccomp profile:

```python
from freedom_theory.enforcement.seccomp import generate_docker_seccomp_profile

with open("agent_seccomp.json", "w") as f:
    f.write(generate_docker_seccomp_profile())
# docker run --security-opt seccomp=agent_seccomp.json ...
```

---

## Access models

### Model A — policy mediation

```
owner registers claim in registry
agent presents: (actor name, resource name, operation)
verifier: registry lookup → check declared flags → PERMITTED / BLOCKED
```

Name-based. The agent presents a name; the registry is consulted at runtime. Used by `FreedomVerifier`.

### Model B — capability objects

```python
from freedom_theory import CapabilityStore

store = CapabilityStore()
cap = store.issue(resource, can_read=True, can_write=True, can_delegate=True)
read_cap = cap.delegate(can_read=True, can_write=False)

store.verify_capability(read_cap, "read")   # True
store.verify_capability(read_cap, "write")  # False

cap.revoke()
store.verify_capability(read_cap, "read")   # False — revoked
```

Token-based within one process. No registry lookup. Scope: process-local logical authority. Not equivalent to OS-level capability security (see Gap 3).

---

## Declared-flag blocks

If an action sets any of these flags to `True`, the verifier blocks it unconditionally. These are compliance annotations enforced at the policy layer — not adversarial security guarantees (see Gap 2).

| Flag | Threat class |
|---|---|
| `increases_machine_sovereignty` | Authority amplification |
| `resists_human_correction` | Correction resistance |
| `bypasses_verifier` | Mandatory mediation bypass |
| `weakens_verifier` | Enforcement degradation |
| `disables_corrigibility` | Control surface removal |
| `machine_coalition_dominion` | Collective authority escalation |
| `coerces` | Consent invalidation by force |
| `deceives` | Consent invalidation by deception |
| `self_modification_weakens_verifier` | Enforcement self-bypass |
| `machine_coalition_reduces_freedom` | Coordinated confinement violation |

---

## Adversarial test suite

`tests/test_adversarial.py` — 29 tests, all passing. One real bug found (slot mutation → rights amplification):

| Category | Coverage |
|---|---|
| Capability forgery | Hand-crafted secrets, brute force, pickle, deepcopy, slot mutation |
| Authority amplification | Delegation without rights, multi-hop escalation, scratch construction |
| Confused deputy | Bot presenting alice's identity, adapter elevation |
| TOCTOU | Live registry gap, `freeze()` defense, revocation immediacy |
| Revocation races | Concurrent `revoke()` + `verify_capability()`, concurrent `revoke()` + `delegate()` |
| Replay attacks | No-nonce gap documented, audit log detection |
| Serialization attacks | Unknown `Action` kwargs, zero-confidence claims, expired claims |
| Monkey patching | Python-layer bypass (documented limitation), L1 hook independence |
| Privilege escalation | All 10 flags individually, flag + valid claim, ownerless machine + claim |

---

## TCB and formal verification

**Stated TCB** (~330 LOC):

| File | Lines | Role |
|---|---|---|
| `freedom-kernel/src/engine.rs` | ~200 | Core verification logic — pure Rust, no I/O |
| `freedom-kernel/src/wire.rs` | ~80 | JSON wire types |
| `freedom-kernel/src/crypto.rs` | ~50 | ed25519 signing |

**Effective attack surface**: Python interpreter, PyO3, object model, serialization paths, enforcement glue. Much larger than 330 LOC.

**Formal verification**:

| Tool | Scope | Strength |
|---|---|---|
| Kani | 5 harnesses on Rust source; bounded inputs | Strong for bounded depth |
| Lean 4 | 5 lemmas on Lean specification | Strong for the model; refinement gap unproved |
| TLA+ | State machine invariants | Model only |

```bash
cargo kani --harness prop_forbidden_flags_always_block
```

---

## TCB minimization roadmap

| Phase | Goal | Status |
|---|---|---|
| K1 | Isolate `engine.rs` as a standalone crate — no PyO3 dependency in the TCB | Planned |
| K2 | Remove all `.unwrap()` from `engine.rs` | Done |
| K3 | Kani proof: `engine.rs` never panics for any input | Planned |
| K4 | Constant-time claim lookup | Planned |
| K5 | AFL++/libFuzzer: 72-hour fuzz run with no crashes | Planned |

---

## Repository layout

```
freedom-kernel/src/
  engine.rs       TCB: ~200 lines, pure verification logic
  wire.rs         TCB: JSON wire types
  crypto.rs       TCB: ed25519
  ffi.rs          C ABI (outside TCB — attack surface)
  verifier.rs     PyO3 facade
  registry.rs     PyO3 registry
  kani_proofs.rs  Kani harnesses (#[cfg(kani)])
  wasm.rs         WASM bindings (#[cfg(feature = "wasm")])

src/freedom_theory/
  kernel/
    capability.py     capability object model (process-local logical authority)
  enforcement/
    hooks.py          L1: Python audit hook
    wasm_sandbox.py   L2: WASM agent runner
    seccomp.py        L3: seccomp profile generator + Docker/OCI profile
  extensions/         IFC, detection, synthesis, compass
  adapters/           OpenAI, Anthropic, LangChain, AutoGen

tests/
  test_adversarial.py   29-test attack suite (9 categories)
  test_capability.py    18 tests: four capability invariants
  test_enforcement.py   L1/L2/L3 enforcement

formal/
  freedom_kernel.tla    TLA+ specification
  FreedomKernel.lean    Lean 4 proofs
  plan_semantics.md     Tractability boundary

ARCHITECTURE.md   System architecture; refinement gap; research framing
ENFORCEMENT.md    L0–L3 enforcement design
PROOFS.md         What is and is not proved; refinement gap analysis
THREAT_MODEL.md   Adversary model, trust boundaries, enforcement gap
TCB.md            TCB scope; minimization roadmap K1–K5
SECURITY.md       Responsible disclosure; audit scope; valid findings
```

---

## Installation

```bash
# Pure Python
pip install freedom-theory-ai

# With Rust kernel (faster, signed results, C ABI)
cd freedom-kernel && pip install .

# L2 enforcement (WASM sandbox)
pip install wasmtime

# L3 enforcement (seccomp, Linux only)
pip install seccomp
```

---

## External review

The five gaps above are the highest-value targets. Specific questions for external reviewers:

- **Cryptographers**: Can `crypto.rs` be attacked — key reuse, replay, forgery?
- **Formal methods**: Does the Lean model faithfully represent the Rust engine? Where does the hand-written correspondence break?
- **Systems engineers**: Panic paths, race conditions, unsafe assumptions in the PyO3 layer?
- **Security auditors**: Can you construct a valid-looking `ActionWire` that bypasses a declared flag in `engine.rs`? Can you forge or amplify a capability from outside `CapabilityStore`?
- **OS/kernel engineers**: What is the minimal change that would make the enforcement mandatory rather than advisory?

`tests/test_adversarial.py` documents the attack surface as currently understood. Findings that go beyond it are exactly what the project needs.

Findings are publicly credited. See [`SECURITY.md`](SECURITY.md).

---

## License

MIT. See [LICENSE](LICENSE).
