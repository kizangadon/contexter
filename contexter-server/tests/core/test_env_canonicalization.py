"""Tests for environment variable canonicalization (AC-EV-001..003).

Ensures the canonical ``CONTEXTER_`` prefix is the only prefix used by
production code, and that the canonical pool-size variable is honored by
the bridge (REQ-EV-001..004).
"""

from pathlib import Path
import re

import pytest

from contexter_server.core.bridge import StorageEngine

#: Root of the production package — scanned for misspelled env prefixes.
_PKG_ROOT = Path(StorageEngine.__module__.split(".")[0]).resolve()
if not _PKG_ROOT.exists():
    _PKG_ROOT = Path(__file__).resolve().parents[2] / "src" / "contexter_server"

#: Matches ``os.environ.get("NAME", ...)`` / ``os.getenv("NAME", ...)`` reads.
_ENV_READ_PATTERN = re.compile(r'os\.(?:environ\.get|getenv)\(\s*["\']([^"\']+)["\']')


def _production_sources() -> list[Path]:
    """Return every ``.py`` file under the production package."""
    return sorted(_PKG_ROOT.rglob("*.py"))


class TestCanonicalPrefixOnly:
    """REQ-EV-001 / AC-EV-002: no production code reads a ``CONtexTER_`` var."""

    def test_no_misspelled_env_prefix_in_production(self) -> None:
        """A repo-wide grep of production code finds zero ``CONtexTER_`` reads."""
        offenders: list[str] = []
        for path in _production_sources():
            for lineno, line in enumerate(path.read_text().splitlines(), 1):
                if "CONtexTER_" in line:
                    offenders.append(f"{path}:{lineno}: {line.strip()}")
        assert not offenders, "misspelled CONtexTER_ prefix found:\n" + "\n".join(offenders)

    def test_no_unprefixed_env_reads_in_production(self) -> None:
        """Production env reads use the canonical ``CONTEXTER_`` prefix only."""
        offenders: list[str] = []
        for path in _production_sources():
            for lineno, line in enumerate(path.read_text().splitlines(), 1):
                for match in _ENV_READ_PATTERN.finditer(line):
                    name = match.group(1)
                    if not name.startswith("CONTEXTER_"):
                        offenders.append(f"{path}:{lineno}: {line.strip()}")
        assert not offenders, "non-canonical env reads found:\n" + "\n".join(offenders)


class TestCanonicalPoolSize:
    """AC-EV-001: the canonical var drives the bridge thread pool size."""

    def test_honors_canonical_var(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setenv("CONTEXTER_BRIDGE_POOL_SIZE", "4")
        engine = StorageEngine(path="/tmp/contexter-test", max_workers=None)
        try:
            assert engine._max_workers == 4
        finally:
            engine._pool.shutdown(wait=False)
