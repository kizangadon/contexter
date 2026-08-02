# ACCEPTANCE — Analytics Count Endpoints
- AC-ACE-001: GIVEN engine with 3 agents and 2 skills seeded, WHEN get_overview runs, THEN counts are 3 and 2.
- AC-ACE-002: GIVEN spy on bridge, WHEN get_overview runs, THEN `count_agents`/`count_skills` are called and `list_agents`/`list_skills` are NOT called.
- AC-ACE-003: GIVEN empty engine, WHEN get_overview runs, THEN counts are 0 (no crash).
- AC-ACE-004: WHEN full suite runs, THEN 0 failures.
