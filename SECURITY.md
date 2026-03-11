# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities via [GitHub](https://github.com/jbejenar/zed-stringknife/security).
Do **NOT** open a public issue for security vulnerabilities.

We will acknowledge receipt within 48 hours and provide an initial assessment
within 7 days.

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |

## Security Measures

- `cargo-deny` checks for known advisories in dependencies
- `cargo-audit` runs in CI on every PR
- No `unsafe` code allowed in the `transforms/` crate
- No network calls or filesystem access in transform functions
- WASM extension runs inside Zed's sandboxed extension host
