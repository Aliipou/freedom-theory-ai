"""
Capability object model tests.

Tests the four core properties of capability security:
  1. Unforgeable  — externally constructed caps fail verification
  2. Delegatable  — owner can issue sub-capabilities
  3. Attenuatable — delegation can only reduce rights, never amplify
  4. Revocable    — revocation cascades to all derived capabilities
"""
import pytest

from freedom_theory.kernel.capability import Capability, CapabilityStore, Rights


def _resource(name="r"):
    from freedom_theory.kernel import Resource, ResourceType
    return Resource(name=name, rtype=ResourceType.FILE)


# ── Rights dataclass ──────────────────────────────────────────────────────────

def test_rights_attenuation_reduces():
    r = Rights(can_read=True, can_write=True, can_delegate=True)
    reduced = r.attenuate(can_write=False)
    assert reduced.can_read is True
    assert reduced.can_write is False
    assert reduced.can_delegate is True


def test_rights_attenuation_cannot_amplify():
    r = Rights(can_read=False, can_write=False, can_delegate=False)
    # Trying to set True on a False right — attenuation can only reduce
    amplified = r.attenuate(can_read=True, can_write=True)
    assert amplified.can_read is False   # False AND True = False
    assert amplified.can_write is False


def test_rights_subsumes():
    full = Rights(can_read=True, can_write=True, can_delegate=True)
    readonly = Rights(can_read=True)
    assert full.subsumes(readonly) is True
    assert readonly.subsumes(full) is False


def test_rights_permits():
    r = Rights(can_read=True, can_write=False)
    assert r.permits("read") is True
    assert r.permits("write") is False
    assert r.permits("nonexistent") is False


# ── CapabilityStore.issue ─────────────────────────────────────────────────────

def test_store_issues_capability_with_correct_rights():
    store = CapabilityStore()
    res = _resource()
    cap = store.issue(res, can_read=True, can_write=True)
    assert cap.permits("read") is True
    assert cap.permits("write") is True
    assert cap.permits("delegate") is False


def test_store_verify_permits_valid_capability():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True)
    assert store.verify_capability(cap, "read") is True


def test_store_verify_blocks_wrong_operation():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True)
    assert store.verify_capability(cap, "write") is False


# ── Unforgeability ────────────────────────────────────────────────────────────

def test_capability_from_different_store_is_rejected():
    store_a = CapabilityStore()
    store_b = CapabilityStore()
    cap_a = store_a.issue(_resource(), can_read=True)
    # cap_a was not issued by store_b — must be rejected
    assert store_b.verify_capability(cap_a, "read") is False


def test_hand_crafted_capability_is_rejected():
    store = CapabilityStore()
    res = _resource()
    # External code cannot construct a valid Capability because
    # it does not know the store's secret
    forged = Capability(res, Rights(can_read=True), _store_secret="wrong-secret")
    assert store.verify_capability(forged, "read") is False


def test_non_capability_object_is_rejected():
    store = CapabilityStore()
    assert store.verify_capability("not-a-cap", "read") is False  # type: ignore[arg-type]


# ── Delegation ────────────────────────────────────────────────────────────────

def test_delegate_creates_sub_capability():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True, can_write=True, can_delegate=True)
    child = cap.delegate(can_read=True, can_write=False)
    assert store.verify_capability(child, "read") is True
    assert store.verify_capability(child, "write") is False


def test_delegate_requires_can_delegate_right():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True, can_delegate=False)
    with pytest.raises(PermissionError, match="delegation"):
        cap.delegate(can_read=True)


def test_delegation_cannot_amplify_rights():
    store = CapabilityStore()
    # Owner grants read-only, no write
    cap = store.issue(_resource(), can_read=True, can_write=False, can_delegate=True)
    # Attempt to delegate with write=True — attenuate ensures write stays False
    child = cap.delegate(can_read=True, can_write=True)
    assert store.verify_capability(child, "write") is False


def test_multi_hop_delegation():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True, can_write=True, can_delegate=True)
    child = cap.delegate(can_read=True, can_write=True, can_delegate=True)
    grandchild = child.delegate(can_read=True, can_write=False)
    assert store.verify_capability(grandchild, "read") is True
    assert store.verify_capability(grandchild, "write") is False


# ── Revocation ────────────────────────────────────────────────────────────────

def test_revoke_invalidates_capability():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True)
    cap.revoke()
    assert store.verify_capability(cap, "read") is False


def test_revoke_cascades_to_children():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True, can_delegate=True)
    child = cap.delegate(can_read=True)
    grandchild = child.delegate(can_read=True)
    # Revoke the root — all descendants must be revoked
    cap.revoke()
    assert store.verify_capability(child, "read") is False
    assert store.verify_capability(grandchild, "read") is False


def test_cannot_delegate_from_revoked_capability():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True, can_delegate=True)
    cap.revoke()
    with pytest.raises(PermissionError, match="revoked"):
        cap.delegate(can_read=True)


def test_rights_property_raises_on_revoked():
    store = CapabilityStore()
    cap = store.issue(_resource(), can_read=True)
    cap.revoke()
    with pytest.raises(PermissionError, match="revoked"):
        _ = cap.rights
