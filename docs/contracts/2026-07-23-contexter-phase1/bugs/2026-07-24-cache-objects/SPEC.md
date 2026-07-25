# Bug 10: Cache Store Domain Objects Not Bytes (Perf H3)

## Problem
Cache stores `Vec<u8>` (serialized bytes). On cache hit, JSON must be re-parsed into domain objects. This adds unnecessary serialization overhead.

## Fix Requirements
1. Change cache value type from `Vec<u8>` to `Box<dyn Any + Send + Sync>` or use an enum of domain types
2. Store typed domain objects (Session, Memory, Agent, Skill) directly in cache
3. Cache hit returns typed object without JSON deserialization
4. Update all cache get/store call sites in engine
