"""Domain service for onboarding and first-run wizard."""

import asyncio

import structlog

from contexter_server.core.bridge import StorageEngine

logger = structlog.get_logger(__name__)


class OnboardingService:
    """Domain service for first-run onboarding and setup wizard."""

    def __init__(self, engine: StorageEngine) -> None:
        self._engine = engine

    async def get_status(self) -> dict:
        """Check if onboarding has been completed."""
        setting = await self._engine.get_setting("onboarding_completed")
        return {
            "completed": setting == "true",
            "onboarding_completed": setting == "true",
        }

    async def submit_wizard(self, data: dict) -> dict:
        """Save onboarding wizard configuration to settings."""
        # Gather all independent set_setting calls concurrently
        coros = [
            self._engine.set_setting(f"onboarding_{key}", str(value))
            for key, value in data.items()
        ]
        coros.append(self._engine.set_setting("onboarding_completed", "true"))

        results = await asyncio.gather(*coros, return_exceptions=True)
        for key, result in zip(list(data.keys()) + ["onboarding_completed"], results):
            if isinstance(result, Exception):
                logger.warning("setting_failed", key=key, error=str(result))
        return {"status": "ok", "message": "Onboarding configuration saved"}

    async def get_progress(self) -> dict:
        """Get onboarding completion progress as a percentage."""
        # Gather all three independent checks concurrently
        setting, agents, sessions = await asyncio.gather(
            self._engine.get_setting("onboarding_project_name"),
            self._engine.list_agents({}),
            self._engine.list_sessions({}),
            return_exceptions=True,
        )

        # Log any failures from gathered calls
        if isinstance(setting, Exception):
            logger.warning("check_failed", entity="setting", error=str(setting))
        if isinstance(agents, Exception):
            logger.warning("check_failed", entity="agents", error=str(agents))
        if isinstance(sessions, Exception):
            logger.warning("check_failed", entity="sessions", error=str(sessions))

        # Evaluate each check — failures count as "not completed"
        setting_ok = isinstance(setting, str) and setting is not None
        agents_ok = isinstance(agents, list) and len(agents) > 0
        sessions_ok = isinstance(sessions, list) and len(sessions) > 0

        checks = [setting_ok, agents_ok, sessions_ok]
        completed = sum(1 for c in checks if c)
        percentage = int((completed / len(checks)) * 100) if checks else 0
        return {
            "percentage": percentage,
            "steps_completed": completed,
            "steps_total": len(checks),
            "checks": checks,
        }
