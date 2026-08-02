"""Live round-trip tests for the bridge large-content bytes path.

These tests run against the real Rust StorageEngine (temporary directory) and
prove the byte-identity guarantees from the double-encode bug contract
(2026-08-01-bridge-double-encode, PF-02):

- content >= 102400 bytes crosses the bridge as raw bytes and comes back
  byte-identical (REQ-BD-001/REQ-BD-002, EC-BD-001..004);
- search results preserve large content byte-identical (AC-BD-002);
- the update path round-trips large content byte-identical.

Content in these tests is case-stable (lowercase ASCII or CJK) because the
engine pre-lowercases memory content on write; byte-identity is asserted
against the case-stable original to isolate the bridge boundary.
"""

import uuid

import pytest

from contexter_server.core.bridge import StorageEngine, _LARGE_CONTENT_THRESHOLD

_ONE_MIB = 1024 * 1024


@pytest.fixture
def engine(tmp_path):
    eng = StorageEngine(path=str(tmp_path / "db"))
    yield eng
    eng._pool.shutdown(wait=False)


def _memory_payload(content: str) -> dict:
    return {
        "session_id": str(uuid.uuid4()),
        "agent_id": str(uuid.uuid4()),
        "memory_type": "fact",
        "content": content,
    }


@pytest.mark.asyncio
async def test_create_memory_exact_threshold_roundtrip_byte_identical(engine):
    """EC-BD-001: exactly 102400 bytes takes the bytes path, byte-identical."""
    content = "x" * _LARGE_CONTENT_THRESHOLD
    created = await engine.create_memory(_memory_payload(content))
    fetched = await engine.get_memory(created["id"])
    assert fetched is not None
    assert fetched["content"] == content
    assert fetched["content"].encode("utf-8") == content.encode("utf-8")


@pytest.mark.asyncio
async def test_create_memory_just_below_threshold_roundtrip_identical(engine):
    """EC-BD-002: 102399 bytes takes the string path, content identical."""
    content = "x" * (_LARGE_CONTENT_THRESHOLD - 1)
    created = await engine.create_memory(_memory_payload(content))
    fetched = await engine.get_memory(created["id"])
    assert fetched is not None
    assert fetched["content"].encode("utf-8") == content.encode("utf-8")


@pytest.mark.asyncio
async def test_create_memory_1mib_roundtrip_byte_identical(engine):
    """EC-BD-003: 1 MiB content (within the engine 1 MB cap) round-trips."""
    content = "y" * _ONE_MIB
    created = await engine.create_memory(_memory_payload(content))
    fetched = await engine.get_memory(created["id"])
    assert fetched is not None
    assert fetched["content"].encode("utf-8") == content.encode("utf-8")


@pytest.mark.asyncio
async def test_create_memory_multibyte_roundtrip_byte_identical(engine):
    """EC-BD-004: multi-byte content (102402 bytes) round-trips byte-identical."""
    content = "\u4e2d" * 34134
    created = await engine.create_memory(_memory_payload(content))
    fetched = await engine.get_memory(created["id"])
    assert fetched is not None
    assert fetched["content"].encode("utf-8") == content.encode("utf-8")


@pytest.mark.asyncio
async def test_search_returns_large_content_byte_identical(engine):
    """AC-BD-002: search results carry large content byte-identical."""
    keyword = "needle"
    content = (keyword + " ") * 30_000  # 210000 chars, case-stable
    created = await engine.create_memory(_memory_payload(content))
    results = await engine.search_memories({"keywords": keyword}, limit=100, offset=0)
    match = next((m for m in results if m["id"] == created["id"]), None)
    assert match is not None
    assert match["content"].encode("utf-8") == content.encode("utf-8")


@pytest.mark.asyncio
async def test_update_memory_large_content_roundtrip_byte_identical(engine):
    """REQ-BD-001: the update bytes path round-trips large content."""
    created = await engine.create_memory(_memory_payload("initial small content"))
    big = "z" * _LARGE_CONTENT_THRESHOLD
    updated = await engine.update_memory(created["id"], {"content": big})
    assert updated is not None
    fetched = await engine.get_memory(created["id"])
    assert fetched is not None
    assert fetched["content"].encode("utf-8") == big.encode("utf-8")
