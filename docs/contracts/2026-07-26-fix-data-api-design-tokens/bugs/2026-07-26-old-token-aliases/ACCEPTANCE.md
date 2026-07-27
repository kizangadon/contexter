# AC-001: Old token names resolve to new values
Given tokens.css has old name `--color-border`, When reading its value, Then it equals `var(--color-border-default)`.

# AC-002: Build succeeds
Given the aliases are added, When `npm run build` is run, Then it exits with code 0.
