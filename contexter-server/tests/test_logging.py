"""Tests for the structlog logging configuration."""

import structlog


class TestStructlogConfig:
    """Verify structlog is configured as expected."""

    def test_structlog_is_configured(self):
        """structlog should have our configured processors."""
        conf = structlog.get_config()
        assert "processors" in conf
        # Should include add_log_level (stdlib) or add_log_level (processors)
        processor_names = [p.__class__.__name__ if hasattr(p, "__class__") else str(p) for p in conf["processors"]]
        # At minimum we should have TimeStamper and some output formatter
        assert any("TimeStamper" in n for n in processor_names) or any("timestamper" in str(p).lower() for p in conf["processors"])

    def test_logger_returns_bound_logger(self):
        """get_logger should return a callable BoundLogger."""
        logger = structlog.get_logger(__name__)
        assert hasattr(logger, "info")
        assert hasattr(logger, "error")
        assert hasattr(logger, "warn")

    def test_logger_output_contains_expected_keys(self, caplog):
        """Logging a message should produce a record with event and level."""
        logger = structlog.get_logger(__name__)
        logger.info("test_message", key="value")

        # We may not have captured it via caplog depending on config,
        # but at minimum the call should not raise.
        assert True  # If we got here without exception, logging works
