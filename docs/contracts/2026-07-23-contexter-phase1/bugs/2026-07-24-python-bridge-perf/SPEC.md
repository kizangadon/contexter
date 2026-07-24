# Bug 12: Python Bridge Performance (Perf M3, M5)

## Problem
Python bridge has double JSON serialization overhead for every call. `max_workers=4` is hardcoded in ThreadPoolExecutor.

## Fix Requirements
1. Expose `max_workers` as parameter in `core_bridge.py` Engine constructor (default: 4)
2. For memories > 100KB in Python bridge, pass as `PyBytes` instead of serializing to JSON string
