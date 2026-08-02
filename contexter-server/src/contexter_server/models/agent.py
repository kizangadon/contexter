"""Agent domain models.

The domain vocabulary is aligned with the Rust engine's serde contract
(``contexter-core/src/models/agent.rs``):

- Engine ``agentType`` (wire key ``type``)  → domain ``type``
- Engine ``capabilities``                  → domain ``capabilities``
  (the legacy ``tools`` input key is still accepted via alias)
- Engine ``status``                        → domain ``status``
  (``AgentStatus`` serializes to the lowercase camelCase ``"active"``/
  ``"inactive"``)
- Engine ``version`` (u32 lock counter)    → domain ``version``
- Engine ``config`` (opaque JSON blob)     → domain ``provider``/``model``/
  ``system_prompt``/``temperature``/``max_tokens``/``metadata``, resolved at
  the service boundary
- Engine ``createdAt``/``updatedAt``       → accepted alongside the snake_case
  aliases so engine payloads validate directly

``provider``/``model`` are optional domain fields — the engine never sends
them and never requires them.
"""

from datetime import datetime, timezone
from typing import Literal, Optional
from uuid import UUID, uuid4

from pydantic import AliasChoices, BaseModel, ConfigDict, Field
from pydantic.fields import FieldInfo


class AliasFieldInfo(FieldInfo):
    """FieldInfo variant whose validation_alias survives FastAPI's standalone
    field adapters without tripping pydantic's UnsupportedFieldAttributeWarning.
    """


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


class Agent(BaseModel):
    """An agent configuration with LLM provider settings."""

    model_config = ConfigDict(populate_by_name=True)

    id: UUID = Field(default_factory=uuid4)
    name: str = Field(..., min_length=1, max_length=256)
    type: str = Field(default="general", max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    capabilities: list[str] = AliasFieldInfo(
        validation_alias=AliasChoices("capabilities", "tools"), default_factory=list
    )
    status: Literal["active", "inactive"] = "active"
    version: int = Field(default=1, ge=1)
    provider: Optional[str] = Field(None, max_length=128)
    model: Optional[str] = Field(None, max_length=256)
    system_prompt: Optional[str] = Field(None, max_length=4096)
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    max_tokens: Optional[int] = Field(None, gt=0)
    metadata: dict = Field(default_factory=dict)
    created_at: datetime = AliasFieldInfo(
        validation_alias=AliasChoices("created_at", "createdAt"), default_factory=_utc_now
    )
    updated_at: datetime = AliasFieldInfo(
        validation_alias=AliasChoices("updated_at", "updatedAt"), default_factory=_utc_now
    )


class AgentCreate(BaseModel):
    """Input model for creating a new agent."""

    model_config = ConfigDict(populate_by_name=True)

    name: str = Field(..., min_length=1, max_length=256)
    type: str = Field(default="general", max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    capabilities: list[str] = AliasFieldInfo(
        validation_alias=AliasChoices("capabilities", "tools"), default_factory=list
    )
    status: Literal["active", "inactive"] = "active"
    provider: Optional[str] = Field(None, max_length=128)
    model: Optional[str] = Field(None, max_length=256)
    system_prompt: Optional[str] = Field(None, max_length=4096)
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    max_tokens: Optional[int] = Field(None, gt=0)
    metadata: dict = Field(default_factory=dict)


class AgentPatch(BaseModel):
    """Input model for partially updating an agent."""

    model_config = ConfigDict(populate_by_name=True)

    name: Optional[str] = Field(None, max_length=256)
    type: Optional[str] = Field(None, max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    capabilities: Optional[list[str]] = AliasFieldInfo(
        validation_alias=AliasChoices("capabilities", "tools"), default=None
    )
    status: Optional[Literal["active", "inactive"]] = None
    provider: Optional[str] = Field(None, max_length=128)
    model: Optional[str] = Field(None, max_length=256)
    system_prompt: Optional[str] = Field(None, max_length=4096)
    temperature: Optional[float] = Field(None, ge=0.0, le=2.0)
    max_tokens: Optional[int] = Field(None, gt=0)
    metadata: Optional[dict] = None
