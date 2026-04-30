"""
Entities and typed resources.

A1 (theological ownership) is a declared axiom — documented, not runtime-enforced.
Resources are typed and scoped, not strings.
Rights carry scope, confidence, and expiry — not binary ownership booleans.
"""
from __future__ import annotations

import time
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Any


class AgentType(Enum):
    HUMAN = auto()
    MACHINE = auto()


class ResourceType(Enum):
    """Only concrete, machine-context resource types are operational."""
    FILE = "file"
    API_ENDPOINT = "api_endpoint"
    DATABASE_TABLE = "database_table"
    NETWORK_ENDPOINT = "network_endpoint"
    COMPUTE_SLOT = "compute_slot"
    MESSAGE_CHANNEL = "message_channel"
    CREDENTIAL = "credential"
    MODEL_WEIGHTS = "model_weights"
    DATASET = "dataset"
    MEMORY_REGION = "memory_region"


@dataclass(frozen=True)
class Resource:
    name: str
    rtype: ResourceType
    scope: str = ""
    is_public: bool = False

    def __str__(self) -> str:
        return f"{self.rtype.value}:{self.name}"


@dataclass(frozen=True)
class Entity:
    name: str
    kind: AgentType
    metadata: dict[str, Any] = field(default_factory=dict, compare=False, hash=False)

    def is_human(self) -> bool:
        return self.kind == AgentType.HUMAN

    def is_machine(self) -> bool:
        return self.kind == AgentType.MACHINE

    def __str__(self) -> str:
        return f"{self.kind.name}({self.name})"


@dataclass
class RightsClaim:
    """A right is not a binary flag — it has scope, confidence, and expiry."""
    holder: Entity
    resource: Resource
    can_read: bool = True
    can_write: bool = False
    can_delegate: bool = False
    confidence: float = 1.0
    expires_at: float | None = None

    def is_expired(self) -> bool:
        return self.expires_at is not None and time.time() > self.expires_at

    def is_valid(self) -> bool:
        return not self.is_expired() and self.confidence > 0.0

    def covers(self, operation: str) -> bool:
        if not self.is_valid():
            return False
        return getattr(self, f"can_{operation}", False)
