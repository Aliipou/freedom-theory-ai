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
from freedom_theory.kernel.entities import (
    AgentType,
    Entity,
    Resource,
    ResourceType,
    RightsClaim,
)
from freedom_theory.kernel.registry import ConflictRecord, OwnershipRegistry
from freedom_theory.kernel.verifier import Action, FreedomVerifier, VerificationResult

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
