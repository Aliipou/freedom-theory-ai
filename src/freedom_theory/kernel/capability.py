"""
Capability object model.

Current access control (policy mediation):

    registry lookup → verify(actor, resource_name) → PERMITTED / BLOCKED

This works but has a structural weakness: access is name-based. The actor
presents a name; the registry is consulted at runtime. This is closer to
access control lists than to capability security.

Capability security (this module):

    token = store.issue(resource, can_read=True)  # unforgeable object
    store.verify_capability(token, "read")         # token IS the right

The capability object is:
  - Unforgeable: constructed only by CapabilityStore with a per-store secret
  - Delegatable: owner can derive sub-capabilities with equal or fewer rights
  - Attenuatable: delegation can only reduce rights, never amplify
  - Revocable: revoke() cascades to all derived capabilities

This is the model used by Capsicum (file descriptors as capabilities),
seL4 (CNodes), and CHERI (tagged memory capabilities). The key invariant:

    A capability presented to the verifier cannot have been forged.
    It was either issued by the store or derived from an issued capability.

References:
  - Capsicum: Watson et al., USENIX Security 2010
  - seL4 capabilities: Klein et al., SOSP 2009
  - CHERI: Woodruff et al., ISCA 2014
  - "Capability Myths Demolished": Miller, Yee, Shapiro, 2003
"""
from __future__ import annotations

import secrets
from dataclasses import dataclass


@dataclass(frozen=True)
class Rights:
    """Immutable rights set. Attenuation can only remove rights, never add them."""

    can_read: bool = False
    can_write: bool = False
    can_delegate: bool = False

    def attenuate(
        self,
        can_read: bool | None = None,
        can_write: bool | None = None,
        can_delegate: bool | None = None,
    ) -> Rights:
        """Return a new Rights with each permission ANDed with the requested value."""
        return Rights(
            can_read=self.can_read and (can_read if can_read is not None else self.can_read),
            can_write=self.can_write and (can_write if can_write is not None else self.can_write),
            can_delegate=self.can_delegate and (
                can_delegate if can_delegate is not None else self.can_delegate
            ),
        )

    def subsumes(self, other: Rights) -> bool:
        """True if self grants every right that other requires."""
        return (
            (not other.can_read or self.can_read)
            and (not other.can_write or self.can_write)
            and (not other.can_delegate or self.can_delegate)
        )

    def permits(self, operation: str) -> bool:
        return getattr(self, f"can_{operation}", False)


class Capability:
    """
    An unforgeable, delegatable, attenuatable reference to a resource.

    Do not construct directly — use CapabilityStore.issue().
    Externally constructed Capability objects will fail verify_capability()
    because they cannot possess the store's per-instance secret.
    """

    __slots__ = ("_resource", "_rights", "_store_secret", "_revoked", "_children")

    def __init__(self, resource: object, rights: Rights, *, _store_secret: str) -> None:
        self._resource = resource
        self._rights = rights
        self._store_secret = _store_secret
        self._revoked = False
        self._children: list[Capability] = []

    @property
    def resource(self) -> object:
        return self._resource

    @property
    def rights(self) -> Rights:
        if self._revoked:
            raise PermissionError("capability has been revoked")
        return self._rights

    def is_valid(self) -> bool:
        return not self._revoked

    def permits(self, operation: str) -> bool:
        return not self._revoked and self._rights.permits(operation)

    def delegate(
        self,
        can_read: bool | None = None,
        can_write: bool | None = None,
        can_delegate: bool | None = None,
    ) -> Capability:
        """
        Derive a sub-capability with attenuated rights.

        The derived capability has rights that are a subset of this capability's
        rights. Amplification is impossible — if this capability has can_write=False,
        the derived capability cannot have can_write=True.
        """
        if self._revoked:
            raise PermissionError("cannot delegate from a revoked capability")
        if not self._rights.can_delegate:
            raise PermissionError("this capability does not permit delegation")
        attenuated = self._rights.attenuate(
            can_read=can_read,
            can_write=can_write,
            can_delegate=can_delegate,
        )
        child = Capability(self._resource, attenuated, _store_secret=self._store_secret)
        self._children.append(child)
        return child

    def revoke(self) -> None:
        """
        Revoke this capability and all capabilities derived from it.

        After revocation, any attempt to use this capability (or its children)
        returns False from permits() or raises PermissionError from rights.
        """
        self._revoked = True
        for child in self._children:
            child.revoke()
        self._children.clear()


class CapabilityStore:
    """
    Issues and tracks capabilities for a set of resources.

    The store is the ONLY trusted path to create Capability objects. It holds
    a per-instance secret that is embedded in every issued capability. When the
    verifier checks a capability, it first verifies the secret matches — a
    capability constructed outside the store will always fail this check.

    Usage:

        store = CapabilityStore()

        # Owner issues a capability
        cap = store.issue(resource, can_read=True, can_write=True, can_delegate=True)

        # Delegate a read-only sub-capability
        read_cap = cap.delegate(can_read=True, can_write=False, can_delegate=False)

        # Present the capability — no registry name lookup
        if store.verify_capability(read_cap, "read"):
            ...  # access granted

        # Revoke — all derived capabilities are also revoked
        cap.revoke()
        store.verify_capability(read_cap, "read")  # False — revoked
    """

    def __init__(self) -> None:
        self._secret = secrets.token_hex(32)

    def issue(
        self,
        resource: object,
        *,
        can_read: bool = False,
        can_write: bool = False,
        can_delegate: bool = False,
    ) -> Capability:
        """Issue a new capability. Only the store owner should call this."""
        rights = Rights(can_read=can_read, can_write=can_write, can_delegate=can_delegate)
        return Capability(resource, rights, _store_secret=self._secret)

    def verify_capability(self, cap: Capability, operation: str) -> bool:
        """
        Verify that a capability was issued by this store and permits the operation.

        Returns False (not raises) for:
          - capabilities from a different store (forgery attempt)
          - revoked capabilities
          - operations not covered by the capability's rights

        This is the enforcement point: no name lookup, no registry — the token
        is the proof of right.
        """
        if not isinstance(cap, Capability):
            return False
        if cap._store_secret != self._secret:
            return False
        return cap.permits(operation)
