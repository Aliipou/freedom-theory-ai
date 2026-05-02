"""
Freedom Theory Kernel — minimal formal AGI governance gate.

LLM → ActionIR → FreedomVerifier → Execute

This package contains only what is necessary for the permission gate:
  entities    — typed data (Entity, Resource, RightsClaim)
  registry    — ownership + conflict detection (OwnershipRegistry)
  verifier    — deterministic gate (FreedomVerifier)

No manipulation detection, synthesis engine, or resolution queue.
Those live in freedom_theory.extensions.
"""
try:
    from freedom_kernel import (  # type: ignore[import]
        Action,
        AgentType,
        ConflictRecord,
        Entity,
        FreedomVerifier,
        OwnershipRegistry,
        Resource,
        ResourceType,
        RightsClaim,
        VerificationResult,
    )
    _BACKEND = "rust"
except ImportError:
    from freedom_theory.kernel._pure import (  # noqa: F401
        Action,
        AgentType,
        ConflictRecord,
        Entity,
        FreedomVerifier,
        OwnershipRegistry,
        Resource,
        ResourceType,
        RightsClaim,
        VerificationResult,
    )
    _BACKEND = "python"

__all__ = [
    "AgentType",
    "Entity",
    "Resource",
    "ResourceType",
    "RightsClaim",
    "ConflictRecord",
    "OwnershipRegistry",
    "Action",
    "FreedomVerifier",
    "VerificationResult",
]
