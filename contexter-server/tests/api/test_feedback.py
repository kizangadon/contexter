"""Tests for the Feedback API router and models."""

import pytest
from pydantic import ValidationError

from contexter_server.models.feedback import BugReport, FeatureSuggestion


class TestBugReportModel:
    """Model validation tests for BugReport."""

    def test_valid_bug_report(self):
        """A BugReport with all required fields passes validation."""
        report = BugReport(title="Crash on startup", description="App crashes immediately")
        assert report.title == "Crash on startup"
        assert report.description == "App crashes immediately"
        assert report.severity == "medium"
        assert report.category == "general"
        assert report.email is None

    def test_valid_bug_report_full(self):
        """A BugReport with all optional fields constructs correctly."""
        report = BugReport(
            title="Login failure",
            description="Cannot log in with SSO",
            email="user@example.com",
            severity="high",
            category="auth",
        )
        assert report.title == "Login failure"
        assert report.email == "user@example.com"
        assert report.severity == "high"
        assert report.category == "auth"

    def test_bug_report_missing_title(self):
        """BugReport without title raises ValidationError."""
        with pytest.raises(ValidationError) as exc:
            BugReport(description="Missing title")
        assert "title" in str(exc.value)

    def test_bug_report_missing_description(self):
        """BugReport without description raises ValidationError."""
        with pytest.raises(ValidationError) as exc:
            BugReport(title="No desc")
        assert "description" in str(exc.value)

    def test_bug_report_empty_strings_rejected(self):
        """BugReport with empty title or description is rejected."""
        with pytest.raises(ValidationError):
            BugReport(title="", description="Something broke")
        with pytest.raises(ValidationError):
            BugReport(title="Valid", description="")


class TestFeatureSuggestionModel:
    """Model validation tests for FeatureSuggestion."""

    def test_valid_suggestion(self):
        """A FeatureSuggestion with all required fields passes validation."""
        suggestion = FeatureSuggestion(title="Add dark mode", description="Would help at night")
        assert suggestion.title == "Add dark mode"
        assert suggestion.category == "general"

    def test_valid_suggestion_full(self):
        """A FeatureSuggestion with all optional fields constructs correctly."""
        suggestion = FeatureSuggestion(
            title="CLI autocomplete",
            description="Tab completion for commands",
            email="dev@example.com",
            severity="low",
            category="cli",
        )
        assert suggestion.email == "dev@example.com"
        assert suggestion.severity == "low"
        assert suggestion.category == "cli"

    def test_suggestion_missing_title(self):
        """FeatureSuggestion without title raises ValidationError."""
        with pytest.raises(ValidationError) as exc:
            FeatureSuggestion(description="Just an idea")
        assert "title" in str(exc.value)

    def test_suggestion_empty_strings_rejected(self):
        """FeatureSuggestion with empty title is rejected."""
        with pytest.raises(ValidationError):
            FeatureSuggestion(title="", description="Should not work")


class TestFeedbackAPI:
    """API endpoint tests for feedback router."""

    def test_report_bug_201(self, client):
        resp = client.post("/api/v1/feedback/bug", json={
            "title": "Test bug",
            "description": "Something broke",
        })
        assert resp.status_code == 201
        assert resp.json()["type"] == "bug"

    def test_suggest_feature_201(self, client):
        resp = client.post("/api/v1/feedback/suggest", json={
            "title": "New feature",
            "description": "Would be nice to have",
        })
        assert resp.status_code == 201
        assert resp.json()["type"] == "suggestion"

    def test_report_bug_missing_title_422(self, client):
        """Missing required field 'title' returns 422."""
        resp = client.post("/api/v1/feedback/bug", json={
            "description": "Something broke",
        })
        assert resp.status_code == 422

    def test_report_bug_missing_description_422(self, client):
        """Missing required field 'description' returns 422."""
        resp = client.post("/api/v1/feedback/bug", json={
            "title": "Test bug",
        })
        assert resp.status_code == 422

    def test_suggest_feature_missing_title_422(self, client):
        """Feature suggestion without title returns 422."""
        resp = client.post("/api/v1/feedback/suggest", json={
            "description": "Nice to have",
        })
        assert resp.status_code == 422
