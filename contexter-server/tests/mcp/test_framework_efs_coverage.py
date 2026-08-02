"""FastMCP filter coverage regression tests (REQ-FC-001..005, EC-FC-001..007).

Bug contract: 2026-08-01-fastmcp-filter-coverage (Auto Bug Loop Iteration 3).

Closes the coverage gaps in ``contexter_server.fastmcp_logging``:

1. ``fastmcp.prompts.function_prompt`` (function_prompt.py:370) was not in
   ``_EMITTER_LOGGERS`` -> prompt error boxes would leak if prompts were
   registered (latent: contexter registers zero prompts).
2. ``fastmcp.server.sampling.run``'s ``Error calling sampling tool ...``
   prefix was not matched (the word "sampling" broke the old prefix match).
3. The schema-validation WARNING ``Invalid arguments for tool ...``
   (server.py:1290) was not suppressed -> validation-failure stderr measured
   486B-767B, width-dependent, carrying a ``server.py:1290`` file:line
   reference from the RichHandler panel.

All logger names and message prefixes below were verified against the
INSTALLED fastmcp 3.4.0 source (``site-packages/fastmcp``):

- emitter loggers (``logger = get_logger(__name__)`` -> same dotted name):
  ``fastmcp.server.server`` (server.py:1284-1297, 1421-1475, 1586-1594),
  ``fastmcp.prompts.function_prompt`` (function_prompt.py:370),
  ``fastmcp.server.sampling.run`` (sampling/run.py:320, 336)
- message prefixes:
  ``Error calling tool `` (server.py:1285, 1297),
  ``Error calling sampling tool `` (sampling/run.py:322, 336),
  ``Error reading resource `` (server.py:1423-1475),
  ``Error rendering prompt `` (server.py:1587-1594, function_prompt.py:370),
  ``Invalid arguments for tool `` (server.py:1291, WARNING)

Drop-policy (REQ-FC-005): covered framework messages are dropped at EVERY
level, including below-WARNING (DEBUG/INFO and FastMCPError ``e.log_level``
paths) — the filter has no level gate, so no covered record passes through.
Contexter's own structlog records (``contexter_server.*``) never match a
framework prefix and keep flowing (REQ-FC-002, REQ-FL-004); the bridge
diagnostics log still receives full tracebacks (REQ-FL-003).
"""

import ast
import logging
from pathlib import Path

import pytest

from contexter_server.core.bridge import StorageEngine
from contexter_server.mcp_server import create_mcp_server
from contexter_server.services.agent_service import AgentService
from contexter_server.services.analytics_service import AnalyticsService
from contexter_server.services.memory_service import MemoryService
from contexter_server.services.session_service import SessionService
from contexter_server.services.skill_service import SkillService

_SID = "00000000-0000-0000-0000-000000000001"
_INVALID_ID = "not-a-uuid"

# REQ-FC-003: validation-class failure section must stay well below 512B.
_VALIDATION_STDERR_BUDGET = 400
# AC-FL-001 budget for failure stderr overall (engine failure + bridge line).
_STDERR_LIMIT = 512

# Rich box drawing characters (AC-FL-001).
_BOX_CHARS = ("╭", "│", "╰")

# The five framework message families emitted by fastmcp's generic
# error-call paths (verified against installed fastmcp 3.4.0).
_FAMILY_MARKERS = (
    "Error calling ",
    "Error reading ",
    "Error rendering ",
    "Invalid arguments for ",
)


# ---------------------------------------------------------------------------
# AST helpers for the emitter-inventory drift test (REQ-FC-004 / EC-FC-004)
# ---------------------------------------------------------------------------


def _static_prefix(node: ast.AST) -> str | None:
    """Return the static prefix of an AST string / f-string expression.

    For a plain constant the whole value is returned; for an f-string only
    the literal parts up to the first interpolation field are returned.
    Returns None when no static prefix can be extracted.
    """
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.JoinedStr):
        parts: list[str] = []
        for value in node.values:
            if isinstance(value, ast.Constant) and isinstance(value.value, str):
                parts.append(value.value)
            else:
                break
        return "".join(parts) if parts else None
    return None


def _module_dotted_name(rel: Path) -> str:
    """Map a package-relative file path to its dotted module name."""
    if rel.name == "__init__.py":
        parts = rel.parts[:-1]
    else:
        parts = rel.parts[:-1] + (rel.stem,)
    return "fastmcp." + ".".join(parts)


def _resolve_module_logger_name(tree: ast.Module, module_name: str) -> str | None:
    """Resolve the module-level ``logger`` name using fastmcp's get_logger rule.

    fastmcp's ``get_logger(name)`` returns ``logging.getLogger(name)`` when
    ``name`` already starts with ``fastmcp.`` and ``logging.getLogger(
    f"fastmcp.{name}")`` otherwise.  A module that assigns
    ``logger = get_logger(__name__)`` therefore logs under its own dotted
    module name.  Returns None when the pattern is not resolvable.
    """
    for node in tree.body:
        if isinstance(node, ast.Assign):
            targets = node.targets
            value = node.value
        elif isinstance(node, ast.AnnAssign):
            targets = [node.target]
            value = node.value
        else:
            continue
        for target in targets:
            if not (isinstance(target, ast.Name) and target.id == "logger"):
                continue
            if value is None or not isinstance(value, ast.Call):
                return None
            fn = value.func
            if not (isinstance(fn, ast.Name) and fn.id in {"get_logger", "getLogger"}):
                return None
            if len(value.args) < 1:
                return None
            arg = value.args[0]
            if isinstance(arg, ast.Constant) and isinstance(arg.value, str):
                name = arg.value
            elif isinstance(arg, ast.Name) and arg.id == "__name__":
                name = module_name
            else:
                return None
            if name.startswith("fastmcp."):
                return name
            return f"fastmcp.{name}"
    return None


def _iter_framework_error_sites(package_dir: Path):
    """Yield (logger_name, path, lineno, static_prefix) for framework sites.

    Scans every ``*.py`` file of the installed fastmcp package for calls on
    the module-level ``logger`` (methods ``error``, ``exception``,
    ``warning``, ``log``) whose first message argument starts with one of the
    framework error-message family markers.  Sites whose logger name cannot
    be resolved are yielded with ``logger_name=None`` so the drift test can
    fail loudly (EC-FC-004).
    """
    for path in sorted(package_dir.rglob("*.py")):
        rel = path.relative_to(package_dir)
        if any(part.startswith("__pycache__") for part in rel.parts):
            continue
        module_name = _module_dotted_name(rel)
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError:
            continue
        logger_name = _resolve_module_logger_name(tree, module_name)
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            func = node.func
            if not (
                isinstance(func, ast.Attribute)
                and isinstance(func.value, ast.Name)
                and func.value.id == "logger"
            ):
                continue
            if func.attr not in {"error", "exception", "warning", "log"}:
                continue
            if func.attr == "log":
                if len(node.args) < 2:
                    continue
                msg_node = node.args[1]
            else:
                if len(node.args) < 1:
                    continue
                msg_node = node.args[0]
            prefix = _static_prefix(msg_node)
            if prefix is None:
                continue
            if not prefix.startswith(_FAMILY_MARKERS):
                continue
            yield logger_name, str(path), node.lineno, prefix


# ---------------------------------------------------------------------------
# Fixtures (mirroring tests/mcp/test_framework_efs_stderr.py)
# ---------------------------------------------------------------------------


@pytest.fixture
def diag_env(tmp_path, monkeypatch):
    """Pin the diagnostics log to the test dir and start with no API key."""
    log_path = tmp_path / "mcp-launch.log"
    monkeypatch.setenv("CONTEXTER_LOG_FILE", str(log_path))
    monkeypatch.delenv("CONTEXTER_API_KEY", raising=False)
    return str(log_path)


@pytest.fixture
def make_server(tmp_path):
    """Factory: real FastMCP server over a real engine + real services."""
    servers = []

    def _make(with_memory: bool = True):
        engine = StorageEngine(str(tmp_path / f"engine-{len(servers)}"))
        services = {
            "memory_service": MemoryService(engine) if with_memory else None,
            "session_service": SessionService(engine) if with_memory else None,
            "agent_service": AgentService(engine),
            "skill_service": SkillService(engine),
            "analytics_service": AnalyticsService(engine),
            "export_service": None,
        }
        mcp = create_mcp_server(**services)
        assert mcp is not None
        servers.append(engine)
        return mcp

    yield _make
    for engine in servers:
        engine._pool.shutdown(wait=True)


def _make_record(
    logger_name: str,
    level: int,
    msg: str,
    args: tuple = (),
    exc_info=None,
) -> logging.LogRecord:
    """Build a LogRecord the way the framework would emit it."""
    return logging.LogRecord(logger_name, level, "server.py", 1290, msg, args, exc_info)


# ---------------------------------------------------------------------------
# Drop-policy (REQ-FC-005): covered messages dropped at every level
# ---------------------------------------------------------------------------


class TestDropPolicyPinned:
    """The filter drops covered messages at every level (REQ-FC-005)."""

    @pytest.fixture
    def filt(self):
        from contexter_server.fastmcp_logging import _SuppressFrameworkTracebackBox

        return _SuppressFrameworkTracebackBox()

    def test_covered_error_records_dropped(self, filt):
        """Every framework prefix is dropped at ERROR, with and without exc_info."""
        prefixes = (
            "Error calling tool 'get_session'",
            "Error calling sampling tool 'search'",
            "Error reading resource 'contexter://session/00000000-0000-0000-0000-000000000001'",
            "Error rendering prompt 'my_prompt'",
            "Invalid arguments for tool 'get_session': []",
        )
        for prefix in prefixes:
            for exc_info in ((ValueError, ValueError("x"), None), None):
                record = _make_record(
                    "fastmcp.server.server", logging.ERROR, prefix, (), exc_info
                )
                assert filt.filter(record) is False, f"{prefix!r} must be dropped"

    def test_covered_warning_records_dropped(self, filt):
        """The schema-validation WARNING (server.py:1290) is dropped at WARNING."""
        record = _make_record(
            "fastmcp.server.server",
            logging.WARNING,
            "Invalid arguments for tool %r: %s",
            (
                "get_session",
                "[{'type': 'string_type', 'loc': ('id',), 'msg': "
                "'Input should be a valid string', 'input': 123}]",
            ),
        )
        assert filt.filter(record) is False, (
            "schema-validation WARNING must be dropped (REQ-FC-002)"
        )

        sampling = _make_record(
            "fastmcp.server.sampling.run",
            logging.WARNING,
            "Error calling sampling tool 'search'",
            (),
            (ValueError, ValueError("x"), None),
        )
        assert filt.filter(sampling) is False

    def test_covered_records_below_warning_dropped(self, filt):
        """Covered messages are dropped at ALL levels (REQ-FC-005, EC-FC-003).

        fastmcp/server/sampling/run.py:322 emits the sampling message at
        ``e.log_level`` (a FastMCPError attribute, any level) — the
        drop-policy applies at every level, not only WARNING+.
        """
        for level in (logging.DEBUG, logging.INFO, logging.WARNING, logging.ERROR):
            record = _make_record(
                "fastmcp.server.server", level, "Error calling tool 'x'"
            )
            assert filt.filter(record) is False, (
                f"covered message must be dropped at level {level}"
            )

    def test_unrelated_and_contexter_records_pass(self, filt):
        """No false suppression: unrelated framework + contexter records pass."""
        passing = (
            _make_record("fastmcp.server", logging.INFO, "Registered %d tools", (3,)),
            _make_record("fastmcp.server", logging.WARNING, "Unrelated warning"),
            _make_record("fastmcp.server", logging.ERROR, "Unrelated error"),
            _make_record(
                "contexter_server.core.bridge",
                logging.ERROR,
                "bridge_call_failed",
                (),
                (ValueError, ValueError("x"), None),
            ),
            _make_record(
                "contexter_server.core.bridge",
                logging.ERROR,
                "handler_error",
                (),
                (ValueError, ValueError("x"), None),
            ),
        )
        for record in passing:
            assert filt.filter(record) is True, f"{record.getMessage()!r} must pass"


# ---------------------------------------------------------------------------
# Emitter coverage: every emitter logger carries the filter (EC-FC-001)
# ---------------------------------------------------------------------------


class TestEmitterCoverageInstalled:
    """True originating loggers carry the filter; records drop before handlers."""

    def test_all_emitter_loggers_carry_filter_after_configure(self):
        """Every name in _EMITTER_LOGGERS has the filter installed (idempotent)."""
        from contexter_server.fastmcp_logging import (
            _EMITTER_LOGGERS,
            _INSTALLED_ATTR,
            _SuppressFrameworkTracebackBox,
            configure_fastmcp_failure_stderr,
        )

        configure_fastmcp_failure_stderr()
        for name in _EMITTER_LOGGERS:
            logger = logging.getLogger(name)
            assert getattr(logger, _INSTALLED_ATTR, False) is True, (
                f"filter marker missing on {name}"
            )
            assert any(
                isinstance(f, _SuppressFrameworkTracebackBox) for f in logger.filters
            ), f"filter instance missing on {name}"

    @pytest.mark.parametrize(
        ("logger_name", "message", "level"),
        [
            # GAP 1: prompt emitter (function_prompt.py:370).
            (
                "fastmcp.prompts.function_prompt",
                "Error rendering prompt 'my_prompt'",
                logging.ERROR,
            ),
            # GAP 2: sampling emitter (sampling/run.py:336).
            (
                "fastmcp.server.sampling.run",
                "Error calling sampling tool 'search'",
                logging.ERROR,
            ),
            # GAP 3: schema-validation WARNING (server.py:1290).
            (
                "fastmcp.server.server",
                "Invalid arguments for tool %r: %s",
                logging.WARNING,
            ),
        ],
    )
    def test_emitter_record_dropped_before_handlers(self, logger_name, message, level):
        """A covered record on its TRUE origin logger reaches no handler."""
        from contexter_server.fastmcp_logging import configure_fastmcp_failure_stderr

        configure_fastmcp_failure_stderr()
        emitted: list[str] = []
        handler = logging.Handler()
        handler.emit = lambda record: emitted.append(record.getMessage())  # type: ignore[method-assign]

        logger = logging.getLogger(logger_name)
        previous_level = logger.level
        logger.setLevel(logging.DEBUG)
        logger.addHandler(handler)
        try:
            args = ("get_session", "[]") if "%r" in message else ()
            logger.log(
                level, message, *args, exc_info=(ValueError, ValueError("x"), None)
            )
            assert emitted == [], (
                f"covered record leaked to handler on {logger_name}: {emitted!r}"
            )
        finally:
            logger.removeHandler(handler)
            logger.setLevel(previous_level)

    def test_unrelated_record_still_reaches_handlers(self):
        """INFO lifecycle records on the same emitter loggers pass through."""
        from contexter_server.fastmcp_logging import configure_fastmcp_failure_stderr

        configure_fastmcp_failure_stderr()
        emitted: list[str] = []
        handler = logging.Handler()
        handler.emit = lambda record: emitted.append(record.getMessage())  # type: ignore[method-assign]

        logger = logging.getLogger("fastmcp.prompts.function_prompt")
        previous_level = logger.level
        logger.setLevel(logging.DEBUG)
        logger.addHandler(handler)
        try:
            logger.info("registered prompt %s", "my_prompt")
            assert emitted == ["registered prompt my_prompt"]
        finally:
            logger.removeHandler(handler)
            logger.setLevel(previous_level)


# ---------------------------------------------------------------------------
# Drift test: emitter inventory of the installed framework (REQ-FC-004)
# ---------------------------------------------------------------------------


class TestEmitterInventoryDrift:
    """The installed fastmcp package's emitter sites are fully covered."""

    def test_emitter_inventory_fully_covered(self):
        """Every family site in the installed package is pinned by the filter.

        Fails loudly if a future fastmcp adds an emitter logger or message
        prefix that the filter does not cover (EC-FC-004).
        """
        import fastmcp

        from contexter_server.fastmcp_logging import (
            _EMITTER_LOGGERS,
            _FRAMEWORK_ERROR_PREFIXES,
        )

        package_dir = Path(fastmcp.__file__).parent
        sites = list(_iter_framework_error_sites(package_dir))

        site_loggers = {name for name, _, _, _ in sites}
        site_prefixes = {prefix for _, _, _, prefix in sites}

        # The three documented emitter loggers must be present (REQ-FC-004).
        assert {
            "fastmcp.server.server",
            "fastmcp.prompts.function_prompt",
            "fastmcp.server.sampling.run",
        } <= site_loggers, f"inventory missing documented emitters: {site_loggers}"

        for logger_name, path, lineno, prefix in sites:
            assert logger_name is not None, (
                f"unresolvable emitter logger at {path}:{lineno} ({prefix!r})"
            )
            assert logger_name in _EMITTER_LOGGERS, (
                f"uncovered emitter logger {logger_name!r} at {path}:{lineno} "
                f"({prefix!r})"
            )
            assert any(prefix.startswith(p) for p in _FRAMEWORK_ERROR_PREFIXES), (
                f"uncovered message prefix {prefix!r} at {path}:{lineno}"
            )

        # Reverse pin: every filter prefix matches at least one live site, so
        # a dead prefix cannot silently accumulate (REQ-FC-004).
        for prefix in _FRAMEWORK_ERROR_PREFIXES:
            assert any(
                site_prefix.startswith(prefix) for site_prefix in site_prefixes
            ), f"dead filter prefix {prefix!r} matches no installed emitter site"


# ---------------------------------------------------------------------------
# Live validation-class margin (REQ-FC-003 / AC-FC-002) and no false
# suppression (AC-FC-004 / REQ-FC-002)
# ---------------------------------------------------------------------------


class TestLiveValidationClass:
    """Schema-validation failures through the real FastMCP path stay clean."""

    @pytest.mark.asyncio
    async def test_schema_validation_failure_stderr_clean_and_bounded(
        self, diag_env, make_server, capfd
    ):
        """Invalid tool arguments: 0 box/file:line/traceback, <=400B (AC-FC-002)."""
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()  # drain startup output
            result = await client.call_tool_mcp("get_session", {"id": 123})

        assert result.isError is True
        stderr = capfd.readouterr().err
        assert "Invalid arguments for tool" not in stderr, (
            "schema-validation WARNING leaked to stderr"
        )
        assert "server.py" not in stderr, "file:line reference leaked to stderr"
        assert "Traceback" not in stderr
        for ch in _BOX_CHARS:
            assert ch not in stderr, f"rich box char {ch!r} present"
        assert len(stderr.encode("utf-8")) <= _VALIDATION_STDERR_BUDGET, (
            f"validation stderr {len(stderr.encode('utf-8'))}B > "
            f"{_VALIDATION_STDERR_BUDGET}B: {stderr!r}"
        )

    @pytest.mark.asyncio
    async def test_engine_failure_no_false_suppression_diagnostics_intact(
        self, diag_env, make_server, capfd, caplog
    ):
        """Engine failure: bridge line kept, stderr bounded, diag traceback intact."""
        caplog.set_level(logging.ERROR)
        mcp = make_server()

        from fastmcp import Client

        async with Client(mcp) as client:
            capfd.readouterr()
            result = await client.call_tool_mcp("get_session", {"id": _INVALID_ID})

        assert result.isError is True
        stderr = capfd.readouterr().err
        # AC-FC-004: contexter's own bounded bridge line is still emitted.
        # In pytest the record is captured by stdlib logging (caplog) because
        # pytest replaces the lastResort stderr handler; in production the
        # record reaches stderr via lastResort. Assert both observable
        # channels: caplog for emission, capfd for bounded/clean stderr.
        bridge_records = [
            r.getMessage()
            for r in caplog.records
            if r.name == "contexter_server.core.bridge"
        ]
        assert any("bridge_call_failed" in msg for msg in bridge_records), (
            "contexter bridge line missing from stdlib logging"
        )
        assert len(stderr.encode("utf-8")) <= _STDERR_LIMIT, (
            f"engine-failure stderr {len(stderr.encode('utf-8'))}B > {_STDERR_LIMIT}B"
        )
        for ch in _BOX_CHARS:
            assert ch not in stderr
        assert "Traceback" not in stderr

        # REQ-FL-003: the diagnostics log still receives the full traceback.
        diag_log = Path(diag_env)
        assert diag_log.exists(), "diagnostics log not written"
        content = diag_log.read_text()
        assert "Traceback" in content, "diagnostics log lost the traceback"
        assert "invalid session id" in content
