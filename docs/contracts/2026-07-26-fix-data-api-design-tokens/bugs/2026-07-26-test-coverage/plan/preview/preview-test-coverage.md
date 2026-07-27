# Design Preview — Test Coverage

## New Tests

### 1. `test_os_expanduser_called` (in test_bridge.py)
```python
def test_os_expanduser_called(mocker):
    """Verify os.path.expanduser is called during bridge init with tilde path."""
    mock_expanduser = mocker.patch("os.path.expanduser", side_effect=lambda p: p.replace("~", "/home/test"))
    bridge = ContexterBridge(data_dir="~/.contexter/")
    mock_expanduser.assert_called_once_with("~/.contexter/")
```

### 2. `test_role_default_is_system` (in test_bridge.py or test_models.py)
```python
def test_role_default_is_system():
    """Memory without role defaults to 'system'."""
    mem = Memory(id=uuid4(), session_id=None, agent_id=None, content="test")
    assert mem.role == "system"
```

## Design Decisions (Resolved from Iter 1)
- **S-01**: MemoryCreate/SessionCreate not updated — deliberate design (orphan fields per D-A2). Not a bug. ✓
- **S-02**: role default — deliberate design per D-A3. Not a bug. ✓
- **N-01**: hex casing — spec-compliant (V2-DEEP uses lowercase). Not a bug. ✓
- **F-01/F-02**: accepted trade-offs. ✓
