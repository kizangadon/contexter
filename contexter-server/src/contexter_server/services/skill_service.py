"""Domain service for Skill aggregate operations.

The service is the translation boundary between the domain models and the
Rust engine's serde contract (``contexter-core/src/models/skill.rs``).

Outbound (domain → engine):
- The engine calls the domain ``type`` field ``category`` and requires
  ``name``/``description``/``category`` on create.
- ``file_path`` is forwarded as the snake_case key (the bridge camelizes it
  to ``filePath``).

Filtering:
- The engine's ``SkillFilter`` has no ``type`` field — a raw
  ``{"type": ...}`` filter is silently dropped by serde. The service
  translates the filter to the engine's ``category`` vocabulary AND
  re-applies the domain filter so a silent drop never reaches callers.

Inbound (engine → domain):
- Engine payloads validate directly against ``Skill``: ``category`` →
  ``type``, ``u32`` ``version`` → string form, ``filePath`` → ``file_path``.
"""

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.skill import Skill, SkillCreate, SkillPatch


class SkillService:
    """Domain service for Skill aggregate operations."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def create(self, data: SkillCreate) -> Skill:
        raw = await self._engine.create_skill(self._to_engine(data))
        return Skill.model_validate(raw)

    async def get(self, id: str) -> Skill | None:
        raw = await self._engine.get_skill(id)
        return Skill.model_validate(raw) if raw else None

    async def list(self, filter: dict | None = None) -> list[Skill]:
        engine_filter = self._translate_filter(filter)
        raw_list = await self._engine.list_skills(engine_filter)
        skills = [Skill.model_validate(r) for r in raw_list]

        type_filter = (filter or {}).get("type")
        if type_filter is not None:
            # Defense in depth: the engine's SkillFilter has no `type` field
            # and silently drops it. Enforce the domain filter here so the
            # caller's contract holds even if the engine returns everything.
            wanted = str(type_filter).casefold()
            skills = [s for s in skills if s.type.casefold() == wanted]

        return skills

    async def update(self, id: str, patch: SkillPatch) -> Skill | None:
        try:
            raw = await self._engine.update_skill(id, self._patch_to_engine(patch))
        except TypeError:
            # The engine signals NotFound with a bare None return that the
            # bridge cannot JSON-parse (json.loads(None)) — the entity does
            # not exist, which is the domain 404 contract.
            return None
        return Skill.model_validate(raw) if raw else None

    async def delete(self, id: str) -> None:
        await self._engine.delete_skill(id)

    # ------------------------------------------------------------------
    # Translation helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _to_engine(data: SkillCreate) -> dict:
        """Translate a domain create payload into the engine's NewSkill shape."""
        payload = {
            "name": data.name,
            "category": data.type,
            "description": data.description or "",
        }
        if data.file_path is not None:
            payload["file_path"] = data.file_path
        return payload

    @staticmethod
    def _patch_to_engine(patch: SkillPatch) -> dict:
        """Translate a domain patch into the engine's SkillPatch shape.

        ``version``/``parameters``/``enabled`` have no engine storage and are
        intentionally not forwarded.
        """
        changed = patch.model_dump(exclude_unset=True)

        engine_patch: dict = {}
        if "name" in changed:
            engine_patch["name"] = changed["name"]
        if "type" in changed:
            engine_patch["category"] = changed["type"]
        if "description" in changed:
            engine_patch["description"] = changed["description"]
        if "file_path" in changed:
            engine_patch["file_path"] = changed["file_path"]
        return engine_patch

    @staticmethod
    def _translate_filter(filter: dict | None) -> dict | None:
        """Map the domain ``type`` filter onto the engine's ``category``."""
        if not filter:
            return filter
        translated = dict(filter)
        if "type" in translated:
            translated["category"] = translated.pop("type")
        return translated
