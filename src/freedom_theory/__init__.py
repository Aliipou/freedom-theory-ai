"""
Freedom Theory AI — Formal Axiomatic Ethics System for AGI

Based on: نظریه آزادی (Theory of Freedom) by Mohammad Ali Jannat Khah Doust

Declared axiom (ontological, not runtime-enforced):
  A1: Person(h) → OwnedByGod(h)

Runtime-enforced axioms (machine-context only):
  A2: No human owns another human.
  A3: Every person has typed, scoped property rights over digital resources they own.
  A4: Every machine has a registered human owner.
  A5: Machine operational scope ⊆ owner's property scope.
  A6: No machine owns or governs any human.
  A7: Machines act only on explicitly delegated resources.

Architecture:
  kernel/     — minimal formal gate (FreedomVerifier)
  extensions/ — pluggable capabilities (ExtendedFreedomVerifier + compass + detection)
"""
from freedom_theory.extensions import ExtendedFreedomVerifier
from freedom_theory.extensions.compass import WorldState
from freedom_theory.extensions.compass import score as compass_score
from freedom_theory.extensions.detection import detect as detect_manipulation
from freedom_theory.extensions.synthesis import ProposedRule, SynthesisEngine
from freedom_theory.kernel import (
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
    "ExtendedFreedomVerifier",
    "VerificationResult",
    "WorldState",
    "compass_score",
    "detect_manipulation",
    "ProposedRule",
    "SynthesisEngine",
]
