"""Tests for MCP tool authentication helpers."""

import os
from unittest import mock

import pytest

from contexter_server.mcp_tools.auth import MCPAuthError, require_api_key


class TestRequireApiKey:
    """Tests for the require_api_key function."""

    def test_no_key_configured_allows_without_key(self) -> None:
        """When CONtexTER_API_KEY is not set, any call passes."""
        with mock.patch.dict(os.environ, {}, clear=True):
            # Should not raise
            require_api_key()
            require_api_key(api_key="some-key")
            require_api_key(api_key="")

    def test_no_key_configured_allows_with_none(self) -> None:
        """When env var is unset, None key is OK."""
        with mock.patch.dict(os.environ, {}, clear=True):
            require_api_key(api_key=None)

    def test_no_key_configured_allows_empty_string(self) -> None:
        """When env var is unset, empty string key is OK."""
        old = os.environ.pop("CONtexTER_API_KEY", None)
        try:
            require_api_key(api_key="")
        finally:
            if old is not None:
                os.environ["CONtexTER_API_KEY"] = old

    def test_accepts_valid_key(self) -> None:
        """When CONtexTER_API_KEY is set, correct key passes."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            require_api_key(api_key="test-key-123")

    def test_rejects_missing_key(self) -> None:
        """When CONtexTER_API_KEY is set, missing key raises."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="API key required"):
                require_api_key()

    def test_rejects_none_key(self) -> None:
        """When CONtexTER_API_KEY is set, None key raises."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="API key required"):
                require_api_key(api_key=None)

    def test_rejects_empty_key(self) -> None:
        """When CONtexTER_API_KEY is set, empty string key raises."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="API key required"):
                require_api_key(api_key="")

    def test_rejects_wrong_key(self) -> None:
        """When CONtexTER_API_KEY is set, wrong key raises."""
        with mock.patch.dict(os.environ, {"CONtexTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="Invalid API key"):
                require_api_key(api_key="wrong-key")

    def test_exception_is_value_error_subtype(self) -> None:
        """MCPAuthError should be a ValueError subtype for FastMCP compatibility."""
        assert issubclass(MCPAuthError, ValueError)
