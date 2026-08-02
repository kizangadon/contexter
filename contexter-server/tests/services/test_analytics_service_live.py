"""Live reproduction tests for the analytics telemetry mapping defect.

REQ-AN-004 / AC-AN-001 / AC-AN-002: seed a real ``contexter_core`` engine in
an isolated temp store, then assert ``AnalyticsService`` counters reflect the
seeded data. Before the fix these were structurally 0 because the service read
snake_case keys the engine never emits (``total_sessions``, ``total_bytes``,
...), silently masked by ``_safe_get``.
"""

import tempfile

import pytest

from contexter_server.core.bridge import StorageEngine
from contexter_server.services.analytics_service import AnalyticsService


@pytest.fixture
def engine():
    """A real StorageEngine over an isolated temp store (per test)."""
    tmp = tempfile.TemporaryDirectory()
    storage = StorageEngine(path=tmp.name)
    yield storage
    tmp.cleanup()


async def _seed(engine: StorageEngine, n_agents: int = 3, n_skills: int = 2) -> None:
    """Seed the store with 3 agents, 2 skills, 1 session, and 2 memories.

    Payloads match the engine's required schema (verified live): agents need
    ``type``/``description``; skills need ``category``.
    """
    first_agent_id: str | None = None
    for i in range(n_agents):
        agent = await engine.create_agent(
            {
                "name": f"analytics-repro-agent-{i}",
                "type": "assistant",
                "description": "Analytics telemetry reproduction agent",
                "model": "gpt-4o",
            }
        )
        if first_agent_id is None:
            first_agent_id = agent["id"]
    for i in range(n_skills):
        await engine.create_skill(
            {
                "name": f"analytics-repro-skill-{i}",
                "description": "Analytics telemetry reproduction skill",
                "category": "general",
            }
        )
    session = await engine.create_session(
        {"agent_id": first_agent_id, "project": "analytics-repro"}
    )
    for content in ("first seeded memory", "second seeded memory"):
        await engine.create_memory(
            {
                "session_id": session["id"],
                "agent_id": first_agent_id,
                "memory_type": "fact",
                "content": content,
                "tags": [],
            }
        )


class TestAnalyticsOverviewLive:
    """AC-ACE-001: overview counters reflect seeded data over the real engine."""

    @pytest.mark.asyncio
    async def test_overview_counts_reflect_seeded_data(self, engine):
        await _seed(engine)
        service = AnalyticsService(engine)

        overview = await service.get_overview()

        assert overview.total_sessions == 1
        assert overview.total_memories == 2
        assert overview.total_agents == 3
        assert overview.total_skills == 2
        assert overview.storage_size_bytes > 0  # store CFs exist on disk

    @pytest.mark.asyncio
    async def test_counts_match_list_based_counts(self, engine):
        """EC-ACE-002: count endpoints agree with list-based counts for a
        non-empty store (parity between the two read paths)."""
        await _seed(engine)
        service = AnalyticsService(engine)

        overview = await service.get_overview()
        listed_agents = await engine.list_agents({}, 10_000, 0)
        listed_skills = await engine.list_skills({}, 10_000, 0)

        assert overview.total_agents == len(listed_agents)
        assert overview.total_skills == len(listed_skills)

    @pytest.mark.asyncio
    async def test_empty_engine_returns_zero_counts(self, engine):
        """EC-ACE-001: an empty store yields zero counts without errors."""
        service = AnalyticsService(engine)

        overview = await service.get_overview()

        assert overview.total_sessions == 0
        assert overview.total_memories == 0
        assert overview.total_agents == 0
        assert overview.total_skills == 0


class TestSystemHealthLive:
    """AC-AN-002: health maps real engine telemetry (not structural zeros)."""

    @pytest.mark.asyncio
    async def test_health_populated_from_real_telemetry(self, engine):
        await _seed(engine)
        service = AnalyticsService(engine)

        health = await service.get_health()

        assert health.status == "ok"
        assert health.storage_size_bytes > 0
        assert health.cache_entries == 6  # 3 agents + 2 skills + 1 session cache-resident

    @pytest.mark.asyncio
    async def test_health_graceful_defaults_without_ops(self, engine):
        """EC-AN-004: health on an untouched engine yields graceful defaults."""
        service = AnalyticsService(engine)

        health = await service.get_health()

        assert health.status == "ok"
        assert health.cache_entries == 0
        assert health.storage_size_bytes >= 0
