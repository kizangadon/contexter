"""Tests for export Pydantic models."""

import uuid
from datetime import datetime

import pytest
from pydantic import ValidationError

from contexter_server.models.export import ExportRequest, ExportStatus


class TestExportModels:
    """Export model validation tests."""

    def test_export_request_defaults(self):
        """ExportRequest defaults."""
        r = ExportRequest()
        assert r.format == "json"
        assert r.entities == []

    def test_export_request_custom(self):
        """ExportRequest with custom values."""
        r = ExportRequest(format="yaml", entities=["sessions", "memories"])
        assert r.format == "yaml"
        assert r.entities == ["sessions", "memories"]

    def test_export_status_defaults(self):
        """ExportStatus defaults."""
        eid = uuid.uuid4()
        s = ExportStatus(id=eid, status="pending")
        assert s.id == eid
        assert s.status == "pending"
        assert s.progress == 0.0
        assert s.format == "json"
        assert isinstance(s.created_at, datetime)
        assert s.completed_at is None
        assert s.error is None
        assert s.file_path is None

    def test_export_status_completed(self):
        """ExportStatus with completion data."""
        eid = uuid.uuid4()
        now = datetime.now()
        s = ExportStatus(
            id=eid,
            status="completed",
            progress=1.0,
            format="csv",
            completed_at=now,
            file_path="/tmp/export.csv",
        )
        assert s.progress == 1.0
        assert s.completed_at == now
        assert s.file_path == "/tmp/export.csv"

    def test_export_status_failed(self):
        """ExportStatus with error."""
        eid = uuid.uuid4()
        s = ExportStatus(id=eid, status="failed", progress=0.5, error="Disk full")
        assert s.status == "failed"
        assert s.error == "Disk full"
