"""Skill domain models.

The domain vocabulary is aligned with the Rust engine's serde contract
(``contexter-core/src/models/skill.rs``):

- Engine ``category``  → domain ``type`` (both keys accepted on input)
- Engine ``version``   → domain string form (engine ``u32`` coerced via
  ``field_validator``)
- Engine ``filePath``  → domain ``file_path`` (camelCase key accepted)
- Engine ``createdAt``/``updatedAt`` → accepted alongside the snake_case
  aliases so engine payloads validate directly

``parameters``/``enabled`` remain domain-only fields: the engine has no
storage for them, so reads return their defaults.
"""

from datetime import datetime, timezone
from typing import Optional
from uuid import UUID, uuid4

from pydantic import AliasChoices, BaseModel, ConfigDict, Field, field_validator
from pydantic.fields import FieldInfo


class AliasFieldInfo(FieldInfo):
    """FieldInfo variant whose validation_alias survives FastAPI's standalone
    field adapters without tripping pydantic's UnsupportedFieldAttributeWarning.
    """


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


def _coerce_version(value: object) -> object:
    """Harmonize the engine's u32 version with the domain's string form."""
    if isinstance(value, int):
        return str(value)
    return value


class Skill(BaseModel):
    """A skill defines a capability an agent can use."""

    model_config = ConfigDict(populate_by_name=True)

    id: UUID = Field(default_factory=uuid4)
    name: str = Field(..., min_length=1, max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    type: str = AliasFieldInfo(validation_alias=AliasChoices("type", "category"))
    version: str = Field(default="1", max_length=64)
    file_path: Optional[str] = AliasFieldInfo(
        validation_alias=AliasChoices("file_path", "filePath"), default=None, max_length=4096
    )
    parameters: dict = Field(default_factory=dict)
    enabled: bool = Field(default=True)
    created_at: datetime = AliasFieldInfo(
        validation_alias=AliasChoices("created_at", "createdAt"), default_factory=_utc_now
    )
    updated_at: datetime = AliasFieldInfo(
        validation_alias=AliasChoices("updated_at", "updatedAt"), default_factory=_utc_now
    )

    @field_validator("version", mode="before")
    @classmethod
    def coerce_version(cls, value: object) -> object:
        return _coerce_version(value)


class SkillCreate(BaseModel):
    """Input model for creating a new skill."""

    model_config = ConfigDict(populate_by_name=True)

    name: str = Field(..., min_length=1, max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    type: str = AliasFieldInfo(validation_alias=AliasChoices("type", "category"))
    version: str = Field(default="1", max_length=64)
    file_path: Optional[str] = Field(None, max_length=4096)
    parameters: dict = Field(default_factory=dict)
    enabled: bool = Field(default=True)

    @field_validator("version", mode="before")
    @classmethod
    def coerce_version(cls, value: object) -> object:
        return _coerce_version(value)


class SkillPatch(BaseModel):
    """Input model for partially updating a skill."""

    model_config = ConfigDict(populate_by_name=True)

    name: Optional[str] = Field(None, max_length=256)
    description: Optional[str] = Field(None, max_length=1024)
    type: Optional[str] = AliasFieldInfo(
        validation_alias=AliasChoices("type", "category"), default=None
    )
    version: Optional[str] = Field(None, max_length=64)
    file_path: Optional[str] = Field(None, max_length=4096)
    parameters: Optional[dict] = None
    enabled: Optional[bool] = None

    @field_validator("version", mode="before")
    @classmethod
    def coerce_version(cls, value: object) -> object:
        return _coerce_version(value)
