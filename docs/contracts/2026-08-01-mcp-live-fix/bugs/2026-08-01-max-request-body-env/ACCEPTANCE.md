# ACCEPTANCE — MAX_REQUEST_BODY Env Canonicalization
- AC-MRB-001: GIVEN env `CONTEXTER_MAX_REQUEST_BODY=1000`, WHEN server reads config, THEN max request body is 1000.
- AC-MRB-002: GIVEN no env set, WHEN server starts, THEN default applies (unchanged).
- AC-MRB-003: WHEN repo grep runs, THEN zero reads of bare `MAX_REQUEST_BODY` env.
