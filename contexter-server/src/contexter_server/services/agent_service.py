"""Domain service for Agent aggregate operations.

The service is the translation boundary between the domain models and the
Rust engine's serde contract (``contexter-core/src/models/agent.rs``).

Outbound (domain → engine):
- The engine requires ``name``, ``type`` and ``description`` on create.
- Domain LLM settings (``provider``/``model``/``system_prompt``/... ) have no
  engine fields: they are persisted inside the engine's opaque ``config``
  blob. Nested config keys are pre-camelized (``systemPrompt``,
  ``maxTokens``) because the bridge only camelizes top-level payload keys.

Inbound (engine → domain):
- The engine never sends ``provider``/``model`` — they are resolved from
  ``config`` (with a fallback to legacy flat payloads for backward
  compatibility with earlier mock/sample shapes).
"""

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.agent import Agent, AgentCreate, AgentPatch

#: Domain field → engine ``config`` key mapping.
_AGENT_CONFIG_KEYS: dict[str, str] = {
    "provider": "provider",
    "model": "model",
    "system_prompt": "systemPrompt",
    "temperature": "temperature",
    "max_tokens": "maxTokens",
    "metadata": "metadata",
}


class AgentService:
    """Domain service for Agent aggregate operations."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def create(self, data: AgentCreate) -> Agent:
        raw = await self._engine.create_agent(self._to_engine(data))
        return self._from_engine(raw)

    async def get(self, id: str) -> Agent | None:
        raw = await self._engine.get_agent(id)
        return self._from_engine(raw) if raw else None

    async def list(self, filter: dict | None = None) -> list[Agent]:
        raw_list = await self._engine.list_agents(filter)
        return [self._from_engine(r) for r in raw_list]

    async def update(self, id: str, patch: AgentPatch) -> Agent | None:
        changed = patch.model_dump(exclude_unset=True)
        needs_existing = any(key in changed for key in _AGENT_CONFIG_KEYS)
        existing = None
        if needs_existing:
            # Config-backed fields must be merged into the current config
            # blob; without the existing payload the merge is impossible,
            # so a missing agent is a 404 (None).
            existing = await self._engine.get_agent(id)
            if existing is None:
                return None
        try:
            raw = await self._engine.update_agent(id, self._patch_to_engine(patch, existing))
        except TypeError:
            # The engine signals NotFound with a bare None return that the
            # bridge cannot JSON-parse (json.loads(None)) — the entity does
            # not exist, which is the domain 404 contract.
            return None
        return self._from_engine(raw) if raw else None

    async def delete(self, id: str) -> None:
        await self._engine.delete_agent(id)

    # ------------------------------------------------------------------
    # Translation helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _to_engine(data: AgentCreate) -> dict:
        """Translate a domain create payload into the engine's NewAgent shape."""
        config: dict = {}
        if data.provider is not None:
            config["provider"] = data.provider
        if data.model is not None:
            config["model"] = data.model
        if data.system_prompt is not None:
            config["systemPrompt"] = data.system_prompt
        config["temperature"] = data.temperature
        if data.max_tokens is not None:
            config["maxTokens"] = data.max_tokens
        if data.metadata:
            config["metadata"] = data.metadata

        return {
            "name": data.name,
            "type": data.type,
            "description": data.description or "",
            "capabilities": data.capabilities,
            "status": data.status,
            "config": config,
        }

    @staticmethod
    def _patch_to_engine(patch: AgentPatch, existing: dict) -> dict:
        """Translate a domain patch into the engine's AgentPatch shape.

        Config-backed fields are merged into the existing config blob so an
        untouched LLM setting survives a partial update. The caller always
        resolves the existing agent first (a missing agent is a 404), so
        ``existing`` is the engine's current payload.
        """
        changed = patch.model_dump(exclude_unset=True)

        engine_patch: dict = {}
        for key in ("name", "type", "description", "status", "capabilities"):
            if key in changed:
                engine_patch[key] = changed[key]

        if any(key in changed for key in _AGENT_CONFIG_KEYS):
            config = dict(existing.get("config") or {})
            for domain_key, engine_key in _AGENT_CONFIG_KEYS.items():
                if domain_key in changed and changed[domain_key] is not None:
                    config[engine_key] = changed[domain_key]
            engine_patch["config"] = config

        return engine_patch

    @staticmethod
    def _from_engine(raw: dict) -> Agent:
        """Translate an engine Agent payload into a domain Agent.

        ``provider``/``model``/... are promoted from the engine's opaque
        ``config`` blob; legacy flat payloads (top-level ``provider``/
        ``model``/``tools`` keys) are still accepted.
        """
        config = raw.get("config") if isinstance(raw.get("config"), dict) else {}

        payload = {
            "id": raw["id"],
            "name": raw["name"],
            "type": raw.get("type", "general"),
            "description": raw.get("description"),
            "capabilities": raw.get("capabilities", raw.get("tools", [])),
            "status": raw.get("status", "active"),
            "version": raw.get("version", 1),
            "provider": config.get("provider") if "provider" in config else raw.get("provider"),
            "model": config.get("model") if "model" in config else raw.get("model"),
            "system_prompt": (
                config.get("systemPrompt")
                if "systemPrompt" in config
                else raw.get("system_prompt")
            ),
            "temperature": (
                config.get("temperature")
                if "temperature" in config
                else raw.get("temperature", 0.7)
            ),
            "max_tokens": (
                config.get("maxTokens") if "maxTokens" in config else raw.get("max_tokens")
            ),
            "metadata": (
                config.get("metadata") if "metadata" in config else raw.get("metadata", {})
            ),
        }
        if raw.get("created_at") or raw.get("createdAt"):
            payload["created_at"] = raw.get("created_at") or raw.get("createdAt")
        if raw.get("updated_at") or raw.get("updatedAt"):
            payload["updated_at"] = raw.get("updated_at") or raw.get("updatedAt")

        return Agent(**payload)
