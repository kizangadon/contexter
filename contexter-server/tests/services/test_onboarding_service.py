"""Tests for OnboardingService."""

from unittest.mock import AsyncMock

import pytest

from contexter_server.services.onboarding_service import OnboardingService


@pytest.fixture
def mock_engine():
    engine = AsyncMock()
    return engine


@pytest.fixture
def service(mock_engine):
    return OnboardingService(mock_engine)


class TestOnboardingServiceGetStatus:
    """Tests for OnboardingService.get_status."""

    @pytest.mark.asyncio
    async def test_returns_not_completed(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        result = await service.get_status()
        assert result["completed"] is False
        assert result["onboarding_completed"] is False

    @pytest.mark.asyncio
    async def test_returns_completed(self, service, mock_engine):
        mock_engine.get_setting.return_value = "true"
        result = await service.get_status()
        assert result["completed"] is True


class TestOnboardingServiceSubmitWizard:
    """Tests for OnboardingService.submit_wizard."""

    @pytest.mark.asyncio
    async def test_submits_wizard_data(self, service, mock_engine):
        mock_engine.set_setting = AsyncMock()
        data = {"project_name": "my-project", "model": "gpt-4"}
        result = await service.submit_wizard(data)
        assert result["status"] == "ok"
        assert mock_engine.set_setting.await_count == 3  # 2 settings + onboarding_completed
        mock_engine.set_setting.assert_any_call("onboarding_project_name", "my-project")
        mock_engine.set_setting.assert_any_call("onboarding_model", "gpt-4")
        mock_engine.set_setting.assert_any_call("onboarding_completed", "true")

    @pytest.mark.asyncio
    async def test_submits_wizard_empty_data(self, service, mock_engine):
        mock_engine.set_setting = AsyncMock()
        data = {}
        result = await service.submit_wizard(data)
        assert result["status"] == "ok"
        # Only onboarding_completed should be called
        mock_engine.set_setting.assert_awaited_once_with("onboarding_completed", "true")

    @pytest.mark.asyncio
    async def test_submits_wizard_gathers_set_settings(self, service, mock_engine):
        """Verify independent set_setting calls are gathered."""
        mock_engine.set_setting = AsyncMock()
        data = {"a": "1", "b": "2", "c": "3"}
        result = await service.submit_wizard(data)
        assert result["status"] == "ok"
        assert mock_engine.set_setting.await_count == 4  # 3 + onboarding_completed

    @pytest.mark.asyncio
    async def test_handles_setting_failure_in_gather(self, service, mock_engine):
        """One failing set_setting should not stop others."""
        calls = []

        async def set_setting_side_effect(key, value):
            calls.append((key, value))
            if key == "onboarding_b":
                raise Exception("setting b failed")

        mock_engine.set_setting = AsyncMock(side_effect=set_setting_side_effect)
        data = {"a": "1", "b": "2", "c": "3"}
        result = await service.submit_wizard(data)
        assert result["status"] == "ok"


class TestOnboardingServiceGetProgress:
    """Tests for OnboardingService.get_progress."""

    @pytest.mark.asyncio
    async def test_returns_zero_progress(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        mock_engine.list_agents.return_value = []
        mock_engine.list_sessions.return_value = []
        result = await service.get_progress()
        assert result["percentage"] == 0
        assert result["steps_completed"] == 0

    @pytest.mark.asyncio
    async def test_returns_partial_progress(self, service, mock_engine):
        mock_engine.get_setting.return_value = "my-project"
        mock_engine.list_agents.return_value = []
        mock_engine.list_sessions.return_value = []
        result = await service.get_progress()
        # 1 out of 3 steps completed
        assert result["percentage"] == 33
        assert result["steps_completed"] == 1

    @pytest.mark.asyncio
    async def test_returns_full_progress(self, service, mock_engine):
        mock_engine.get_setting.return_value = "my-project"
        mock_engine.list_agents.return_value = [{"id": "a1", "name": "Agent 1", "provider": "openai", "model": "gpt-4"}]
        mock_engine.list_sessions.return_value = [{"id": "s1", "agent_id": "a1", "project": "test", "status": "active"}]
        result = await service.get_progress()
        assert result["percentage"] == 100
        assert result["steps_completed"] == 3

    @pytest.mark.asyncio
    async def test_handles_bridge_error(self, service, mock_engine):
        mock_engine.get_setting.return_value = None
        mock_engine.list_agents.side_effect = Exception("bridge error")
        mock_engine.list_sessions.side_effect = Exception("bridge error")
        result = await service.get_progress()
        assert result["steps_completed"] == 0

    @pytest.mark.asyncio
    async def test_gathers_independent_calls(self, service, mock_engine):
        """Verify get_setting, list_agents, and list_sessions are gathered."""
        mock_engine.get_setting.return_value = "my-project"
        mock_engine.list_agents.return_value = [{"id": "a1"}]
        mock_engine.list_sessions.return_value = [{"id": "s1"}]
        result = await service.get_progress()
        assert result["percentage"] == 100

    @pytest.mark.asyncio
    async def test_handles_partial_gather_failure(self, service, mock_engine):
        """One failing call in gather should not cancel others."""
        mock_engine.get_setting.return_value = "my-project"
        mock_engine.list_agents.side_effect = Exception("agents failed")
        mock_engine.list_sessions.return_value = [{"id": "s1"}]
        result = await service.get_progress()
        # get_setting: "my-project" → truthy → success
        # list_agents: failed → 0 agents → fail
        # list_sessions: 1 session → success
        assert result["steps_completed"] == 2
        assert result["percentage"] == 66
