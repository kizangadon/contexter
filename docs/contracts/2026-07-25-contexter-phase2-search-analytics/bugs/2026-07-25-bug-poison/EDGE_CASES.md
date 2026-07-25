# Edge Cases — Bug-Poison

- EC-01: Poisoned lock contains corrupt data — recovery returns stale data but doesn't panic
- EC-02: All mutexes poisoned simultaneously — each recovers independently
