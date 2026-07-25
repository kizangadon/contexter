"""Tests for the Onboarding API router and model validation."""

from unittest.mock import AsyncMock

import pytest
from fastapi.testclient import TestClient
from pydantic import ValidationError

from contexter_server.api.onboarding import WizardData


class TestWizardDataModel:
    """Model validation tests for WizardData."""

    def test_valid_wizard_data(self):
        """A WizardData with all required fields passes."""
        wd = WizardData(
            responses={"project_name": "my-project"},
            completed_step="project_setup",
        )
        assert wd.responses == {"project_name": "my-project"}
        assert wd.completed_step == "project_setup"

    def test_wizard_data_empty_responses(self):
        """WizardData with empty responses dict is valid."""
        wd = WizardData(responses={}, completed_step="done")
        assert wd.responses == {}

    def test_wizard_data_missing_responses(self):
        """WizardData without responses raises ValidationError."""
        with pytest.raises(ValidationError) as exc:
            WizardData(completed_step="done")
        assert "responses" in str(exc.value)

    def test_wizard_data_missing_completed_step(self):
        """WizardData without completed_step raises ValidationError."""
        with pytest.raises(ValidationError) as exc:
            WizardData(responses={"key": "value"})
        assert "completed_step" in str(exc.value)

    def test_wizard_data_empty_completed_step(self):
        """WizardData with empty completed_step is rejected."""
        with pytest.raises(ValidationError):
            WizardData(responses={}, completed_step="")


class TestOnboardingAPI:
    """API endpoint tests for onboarding router."""

    def test_get_status(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.get_setting.return_value = None
        resp = client.get("/api/v1/onboarding/status")
        assert resp.status_code == 200
        assert resp.json()["completed"] is False

    def test_get_status_completed(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.get_setting.return_value = "true"
        resp = client.get("/api/v1/onboarding/status")
        assert resp.status_code == 200
        assert resp.json()["completed"] is True

    def test_submit_wizard(self, client: TestClient, mock_engine: AsyncMock):
        resp = client.post("/api/v1/onboarding/wizard", json={
            "responses": {"project_name": "my-project"},
            "completed_step": "project_setup",
        })
        assert resp.status_code == 200
        assert resp.json()["status"] == "ok"

    def test_get_progress(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.get_setting.return_value = None
        mock_engine.list_agents.return_value = []
        mock_engine.list_sessions.return_value = []
        resp = client.get("/api/v1/onboarding/progress")
        assert resp.status_code == 200
        assert "percentage" in resp.json()

    def test_submit_wizard_missing_fields_422(self, client: TestClient, mock_engine: AsyncMock):
        """Missing WizardData required fields returns 422."""
        resp = client.post("/api/v1/onboarding/wizard", json={
            "project_name": "my-project",
        })
        assert resp.status_code == 422

    def test_submit_wizard_empty_completed_step_422(self, client: TestClient, mock_engine: AsyncMock):
        """Empty completed_step returns 422."""
        resp = client.post("/api/v1/onboarding/wizard", json={
            "responses": {"project_name": "my-project"},
            "completed_step": "",
        })
        assert resp.status_code == 422
