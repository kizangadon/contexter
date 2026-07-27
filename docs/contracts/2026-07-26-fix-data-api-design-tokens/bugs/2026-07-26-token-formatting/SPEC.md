# SPEC: Token formatting cleanup

## Fixes

### 1. Shadow rgba whitespace
In `tokens.css`, change shadow values to remove spaces after commas in `rgba()`:
- `rgba(0, 0, 0, 0.3)` → `rgba(0,0,0,0.3)`
- `rgba(0, 0, 0, 0.4)` → `rgba(0,0,0,0.4)`
- `rgba(0, 0, 0, 0.5)` → `rgba(0,0,0,0.5)`

### 2. Hex casing consistency
Normalize `#181716` (lowercase) to `#181716` — it's already lowercase in the V2-DEEP design preview spec, so it's correct. But verify all other hex values use consistent casing. The V2-DEEP spec primarily uses UPPERCASE. `#181716` was NOT changed from lowercase in the approved spec, so leave it lowercase per spec literal.

Actually only the `--shadow-accent` value should be checked — verify: `0 0 20px #7C5CFC30` matches the spec (no spaces in rgba-like suffix).

## Verification
```bash
cd /home/don/Code/contexter/contexter-web
grep -n 'rgba(' src/styles/tokens.css
```
All rgba values should have no spaces after commas.
