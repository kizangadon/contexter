"""Tests for the Skills API router."""

from unittest.mock import AsyncMock

from fastapi.testclient import TestClient


class TestListSkills:
    def test_list_skills_empty(self, client: TestClient):
        resp = client.get("/api/v1/skills")
        assert resp.status_code == 200
        assert resp.json() == []

    def test_list_skills_with_data(self, client: TestClient, mock_engine: AsyncMock, sample_skill: dict):
        mock_engine.list_skills.return_value = [sample_skill]
        resp = client.get("/api/v1/skills")
        assert resp.status_code == 200
        assert len(resp.json()) == 1


class TestCreateSkill:
    def test_create_skill_201(self, client: TestClient, mock_engine: AsyncMock, any_uuid: str):
        mock_engine.create_skill.return_value = {
            "id": any_uuid,
            "name": "new-skill",
            "description": "A new skill",
            "type": "memory",
            "enabled": True,
            "parameters": {},
            "created_at": "2026-07-25T10:00:00Z",
            "updated_at": "2026-07-25T10:00:00Z",
        }
        resp = client.post("/api/v1/skills", json={
            "name": "new-skill",
            "type": "memory",
        })
        assert resp.status_code == 201
        assert resp.json()["name"] == "new-skill"

    def test_create_skill_422(self, client: TestClient):
        resp = client.post("/api/v1/skills", json={})
        assert resp.status_code == 422


class TestGetSkill:
    def test_get_skill_found(self, client: TestClient, mock_engine: AsyncMock, sample_skill: dict):
        mock_engine.get_skill.return_value = sample_skill
        resp = client.get(f"/api/v1/skills/{sample_skill['id']}")
        assert resp.status_code == 200

    def test_get_skill_404(self, client: TestClient):
        resp = client.get("/api/v1/skills/nonexistent")
        assert resp.status_code == 404


class TestUpdateSkill:
    def test_update_skill(self, client: TestClient, mock_engine: AsyncMock, sample_skill: dict):
        updated = dict(sample_skill, name="updated-skill")
        mock_engine.update_skill.return_value = updated
        resp = client.put(f"/api/v1/skills/{sample_skill['id']}", json={"name": "updated-skill"})
        assert resp.status_code == 200
        assert resp.json()["name"] == "updated-skill"

    def test_update_skill_404(self, client: TestClient, mock_engine: AsyncMock):
        mock_engine.update_skill.return_value = {}
        resp = client.put("/api/v1/skills/nonexistent", json={"name": "Nope"})
        assert resp.status_code == 404


class TestDeleteSkill:
    def test_delete_skill_204(self, client: TestClient):
        resp = client.delete("/api/v1/skills/some-id")
        assert resp.status_code == 204

    def test_delete_skill_idempotent(self, client: TestClient):
        resp = client.delete("/api/v1/skills/nonexistent")
        assert resp.status_code == 204
