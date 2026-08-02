"""Tests for MCP tool authentication helpers."""

import os
from unittest import mock

import pytest

from contexter_server.mcp_tools.auth import MCPAuthError, require_api_key


class TestRequireApiKey:
    """Tests for the require_api_key function."""

    def test_no_key_configured_allows_without_key(self) -> None:
        """When CONTEXTER_API_KEY is not set, any call passes."""
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
        old = os.environ.pop("CONTEXTER_API_KEY", None)
        try:
            require_api_key(api_key="")
        finally:
            if old is not None:
                os.environ["CONTEXTER_API_KEY"] = old

    def test_accepts_valid_key(self) -> None:
        """When CONTEXTER_API_KEY is set, correct key passes."""
        with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": "test-key-123"}):
            require_api_key(api_key="test-key-123")

    def test_rejects_missing_key(self) -> None:
        """When CONTEXTER_API_KEY is set, missing key raises."""
        with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="API key required"):
                require_api_key()

    def test_rejects_none_key(self) -> None:
        """When CONTEXTER_API_KEY is set, None key raises."""
        with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="API key required"):
                require_api_key(api_key=None)

    def test_rejects_empty_key(self) -> None:
        """When CONTEXTER_API_KEY is set, empty string key raises."""
        with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="API key required"):
                require_api_key(api_key="")

    def test_rejects_wrong_key(self) -> None:
        """When CONTEXTER_API_KEY is set, wrong key raises."""
        with mock.patch.dict(os.environ, {"CONTEXTER_API_KEY": "test-key-123"}):
            with pytest.raises(MCPAuthError, match="Invalid API key"):
                require_api_key(api_key="wrong-key")

    def test_exception_is_value_error_subtype(self) -> None:
        """MCPAuthError should be a ValueError subtype for FastMCP compatibility."""
        assert issubclass(MCPAuthError, ValueError)


class TestEnvVarCanonicalName:
    """The MCP layer and the API layer must read the same canonical env var name.

    The frozen contract (SPEC REQ-004, EDGE_CASES) documents ``CONTEXTER_API_KEY``.
    The code used to read the misspelled ``CONtexTER_API_KEY``, so setting the
    documented name silently disabled authentication. Both layers must read
    ``CONTEXTER_API_KEY``.
    """

    def test_mcp_layer_reads_canonical_env_var_name(self) -> None:
        """Setting CONTEXTER_API_KEY must enable MCP auth."""
        with mock.patch.dict(
            os.environ, {"CONTEXTER_API_KEY": "test-key-123"}, clear=True
        ):
            require_api_key(api_key="test-key-123")
            with pytest.raises(MCPAuthError, match="API key required"):
                require_api_key()

    @pytest.mark.asyncio
    async def test_api_layer_reads_canonical_env_var_name(self) -> None:
        """Setting CONTEXTER_API_KEY must enable REST API auth."""
        from fastapi import HTTPException, Request

        from contexter_server.api.deps import get_api_key

        with mock.patch.dict(
            os.environ, {"CONTEXTER_API_KEY": "test-key-123"}, clear=True
        ):
            good = Request(
                {
                    "type": "http",
                    "headers": [(b"authorization", b"Bearer test-key-123")],
                }
            )
            await get_api_key(good)  # must not raise

            missing = Request({"type": "http", "headers": []})
            with pytest.raises(HTTPException) as exc:
                await get_api_key(missing)
            assert exc.value.status_code == 401

    def test_misspelled_legacy_env_var_no_longer_gates_auth(self) -> None:
        """CONtexTER_API_KEY (legacy misspelling) must not gate anything anymore."""
        with mock.patch.dict(
            os.environ, {"CONtexTER_API_KEY": "test-key-123"}, clear=True
        ):
            require_api_key()  # canonical name unset → open mode
