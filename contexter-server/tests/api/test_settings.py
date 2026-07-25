"""Tests for the Settings API router and model validation."""

import pytest
from pydantic import ValidationError

from contexter_server.models.settings import SectionUpdate


class TestSectionUpdateModel:
    """Model validation tests for SectionUpdate."""

    def test_valid_section_update(self):
        """A SectionUpdate with valid values passes."""
        update = SectionUpdate(values={"name": "my-project", "description": "Test"})
        assert update.values == {"name": "my-project", "description": "Test"}

    def test_section_update_empty_values(self):
        """SectionUpdate with empty values dict is rejected."""
        with pytest.raises(ValidationError):
            SectionUpdate(values={})

    def test_section_update_missing_values(self):
        """SectionUpdate without values raises ValidationError."""
        with pytest.raises(ValidationError) as exc:
            SectionUpdate()
        assert "values" in str(exc.value)


class TestSettingsAPI:
    """API endpoint tests for settings router."""

    def test_get_section_project(self, client):
        resp = client.get("/api/v1/settings/project")
        assert resp.status_code == 200
        data = resp.json()
        assert "name" in data
        assert isinstance(data["name"], str)

    def test_get_section_storage(self, client):
        resp = client.get("/api/v1/settings/storage")
        assert resp.status_code == 200
        assert "path" in resp.json()

    def test_get_section_rest(self, client):
        resp = client.get("/api/v1/settings/rest")
        assert resp.status_code == 200
        assert resp.json()["port"] == 8051

    def test_get_section_not_found_404(self, client):
        resp = client.get("/api/v1/settings/nonexistent")
        assert resp.status_code == 404

    def test_update_section(self, client):
        resp = client.put("/api/v1/settings/project", json={
            "values": {"name": "updated-project"},
        })
        assert resp.status_code == 200
        assert resp.json()["name"] == "updated-project"

    def test_update_section_not_found_404(self, client):
        resp = client.put("/api/v1/settings/nonexistent", json={
            "values": {"name": "test"},
        })
        assert resp.status_code == 404

    def test_update_section_missing_values_422(self, client):
        """Missing 'values' field returns 422."""
        resp = client.put("/api/v1/settings/project", json={
            "name": "updated-project",
        })
        assert resp.status_code == 422

    def test_update_section_empty_values_422(self, client):
        """Empty 'values' dict returns 422."""
        resp = client.put("/api/v1/settings/project", json={
            "values": {},
        })
        assert resp.status_code == 422
