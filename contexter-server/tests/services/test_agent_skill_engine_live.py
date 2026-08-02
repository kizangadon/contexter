"""Live acceptance tests: Agent/Skill services against the REAL Rust engine.

These tests reproduce the schema-drift defect end-to-end: the Python domain
models/services must round-trip against ``contexter_core`` (the compiled PyO3
extension) through the async ``StorageEngine`` bridge using a temp store —
never ``~/.contexter``.

Before the fix, ``AgentService.create`` sent a payload without the engine's
required ``type``/``description`` fields (engine error: missing field), and
parsing engine responses failed on the camelCase ``createdAt``/``updatedAt``
keys and the missing ``provider``/``model`` fields (engine never sends them).
"""

import tempfile
from pathlib import Path

import pytest

from contexter_server.core.bridge import StorageEngine
from contexter_server.models.agent import AgentCreate, AgentPatch
from contexter_server.models.skill import SkillCreate, SkillPatch
from contexter_server.services.agent_service import AgentService
from contexter_server.services.skill_service import SkillService


@pytest.fixture
def engine():
    """A real StorageEngine over an isolated temp store (per test)."""
    tmp = tempfile.TemporaryDirectory()
    storage = StorageEngine(path=tmp.name)
    yield storage
    tmp.cleanup()


class TestAgentLiveEngine:
    """AC-AG-001 / AC-AG-002: Agent round-trips against the real engine."""

    @pytest.mark.asyncio
    async def test_create_and_get_round_trip(self, engine):
        """A created agent must be retrievable with all contract fields resolved."""
        service = AgentService(engine)
        data = AgentCreate(
            name="live-agent",
            provider="openai",
            model="gpt-4o",
            type="coding-assistant",
            description="Live acceptance agent",
            system_prompt="Be concise.",
            temperature=0.3,
            max_tokens=1024,
            capabilities=["code", "terminal"],
            metadata={"team": "core"},
        )

        created = await service.create(data)
        assert created.name == "live-agent"
        assert created.provider == "openai"
        assert created.model == "gpt-4o"
        assert created.type == "coding-assistant"
        assert created.description == "Live acceptance agent"
        assert created.capabilities == ["code", "terminal"]
        assert created.status == "active"
        assert created.version == 1
        assert created.system_prompt == "Be concise."
        assert created.temperature == 0.3
        assert created.max_tokens == 1024
        assert created.metadata == {"team": "core"}

        fetched = await service.get(str(created.id))
        assert fetched is not None
        assert fetched.name == "live-agent"
        assert fetched.provider == "openai"
        assert fetched.model == "gpt-4o"
        assert fetched.type == "coding-assistant"
        assert fetched.capabilities == ["code", "terminal"]
        assert fetched.system_prompt == "Be concise."
        assert fetched.temperature == 0.3
        assert fetched.max_tokens == 1024
        assert fetched.metadata == {"team": "core"}

    @pytest.mark.asyncio
    async def test_create_without_provider_model_defaults(self, engine):
        """Agents without LLM settings must still round-trip (provider/model optional)."""
        service = AgentService(engine)
        created = await service.create(AgentCreate(name="bare-agent"))
        assert created.provider is None
        assert created.model is None
        assert created.type == "general"
        assert created.status == "active"

        fetched = await service.get(str(created.id))
        assert fetched is not None
        assert fetched.provider is None
        assert fetched.model is None

    @pytest.mark.asyncio
    async def test_update_preserves_config(self, engine):
        """A name-only update must preserve the LLM settings in the engine config."""
        service = AgentService(engine)
        created = await service.create(
            AgentCreate(name="patch-me", provider="anthropic", model="claude-3")
        )

        updated = await service.update(str(created.id), AgentPatch(name="patched"))
        assert updated is not None
        assert updated.name == "patched"
        assert updated.provider == "anthropic"  # preserved through config merge
        assert updated.model == "claude-3"

        model_updated = await service.update(str(created.id), AgentPatch(model="claude-3-5"))
        assert model_updated is not None
        assert model_updated.provider == "anthropic"
        assert model_updated.model == "claude-3-5"

    @pytest.mark.asyncio
    async def test_update_missing_returns_none(self, engine):
        """Updating a nonexistent agent must return None (404 contract)."""
        service = AgentService(engine)
        result = await service.update("00000000-0000-0000-0000-000000000000", AgentPatch(name="x"))
        assert result is None

    @pytest.mark.asyncio
    async def test_list_agents_round_trip(self, engine):
        """Listed agents must parse with config-resolved provider/model."""
        service = AgentService(engine)
        await service.create(AgentCreate(name="list-a", provider="openai", model="gpt-4"))
        await service.create(AgentCreate(name="list-b", provider="anthropic", model="claude-3"))

        agents = await service.list()
        names = {a.name: a for a in agents}
        assert names["list-a"].provider == "openai"
        assert names["list-b"].provider == "anthropic"


class TestSkillLiveEngine:
    """AC-SK-001 / AC-SK-002: Skill round-trips and type filtering against the real engine."""

    @pytest.mark.asyncio
    async def test_create_and_get_round_trip(self, engine):
        """A created skill must be retrievable with harmonized category/version."""
        service = SkillService(engine)
        created = await service.create(
            SkillCreate(name="live-memory-skill", type="memory", description="Live skill")
        )
        assert created.name == "live-memory-skill"
        assert created.type == "memory"
        assert created.version == "1"  # engine u32 version harmonized to str

        fetched = await service.get(str(created.id))
        assert fetched is not None
        assert fetched.type == "memory"
        assert fetched.version == "1"
        assert fetched.description == "Live skill"

    @pytest.mark.asyncio
    async def test_list_skills_type_filter_applies(self, engine):
        """list_skills(type=...) must actually filter — no silent serde drop."""
        service = SkillService(engine)
        await service.create(SkillCreate(name="memory-skill", type="memory"))
        await service.create(SkillCreate(name="search-skill", type="search"))
        await service.create(SkillCreate(name="reasoning-skill", type="reasoning"))

        memory = await service.list({"type": "memory"})
        assert [s.name for s in memory] == ["memory-skill"]

        search = await service.list({"type": "search"})
        assert [s.name for s in search] == ["search-skill"]

        all_skills = await service.list()
        assert len(all_skills) == 3

    @pytest.mark.asyncio
    async def test_create_with_file_path_round_trip(self, engine):
        """file_path must survive the engine boundary (camelCase filePath)."""
        service = SkillService(engine)
        created = await service.create(
            SkillCreate(name="path-skill", type="custom", file_path="/tmp/skills/path-skill.py")
        )
        assert created.file_path == "/tmp/skills/path-skill.py"

        fetched = await service.get(str(created.id))
        assert fetched is not None
        assert fetched.file_path == "/tmp/skills/path-skill.py"

    @pytest.mark.asyncio
    async def test_update_translates_type(self, engine):
        """Updating a skill's type must persist via the engine's category."""
        service = SkillService(engine)
        created = await service.create(SkillCreate(name="retype-me", type="memory"))

        updated = await service.update(str(created.id), SkillPatch(type="search"))
        assert updated is not None
        assert updated.type == "search"

        fetched = await service.get(str(created.id))
        assert fetched is not None
        assert fetched.type == "search"

    @pytest.mark.asyncio
    async def test_update_missing_returns_none(self, engine):
        """Updating a nonexistent skill must return None (404 contract).

        The engine's ``update_skill`` returns ``None`` for a missing skill
        (``EngineError::NotFound`` → ``Ok(None)``), which the bridge cannot
        JSON-parse (``json.loads(None)`` → ``TypeError``). The service must
        translate that into the caller's 404 contract (``None``) instead of
        propagating the bridge crash.
        """
        service = SkillService(engine)
        result = await service.update(
            "00000000-0000-0000-0000-000000000000", SkillPatch(name="x")
        )
        assert result is None

    @pytest.mark.asyncio
    async def test_list_skills_unknown_type_returns_empty(self, engine):
        """list_skills(type=<unknown>) returns an empty list — never a traceback."""
        service = SkillService(engine)
        await service.create(SkillCreate(name="memory-skill", type="memory"))

        result = await service.list({"type": "nonexistent"})
        assert result == []
