# EDGE CASES — EFS Test Docstring Accuracy

## EC-DC-001 — Keep the discriminating claim
The docstring must not overclaim ("all framework output gone") — it must accurately bound scope: covered framework records only, contexter's own structlog records unaffected.

## EC-DC-002 — Docstring vs code drift
If the docstring describes a mechanism (e.g., RichHandler `exc_info` wrap), ensure it matches the actual implementation; do not invent mechanics.

## EC-DC-003 — Fabricated-ID sweep (in-scope)
All fabricated `REQ-FF-*` references in `test_framework_efs_coverage.py` — module docstring AND the three inline section comments at lines ~248, ~494, ~564 — SHALL be corrected to the real `REQ-FC-*`/`REQ-FL-*` IDs. Do NOT rename valid `REQ-FC-*`/`REQ-FL-*` references (those match real contract IDs verbatim).

## EC-DG-004 — Test remains discriminating
The `test_covered_records_below_warning_dropped` test must still assert drop-at-every-level; do not weaken it to match a wrong docstring.