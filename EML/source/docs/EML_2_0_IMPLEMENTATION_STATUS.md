# EML 2.0 Implementation Status

This document maps the EML 2.0 upgrade objective to the current local
implementation. It is intentionally separate from the upstream release and
is maintained on the `embervault-eml-2.0-dev` branch.

## Capability matrix

| Capability | Implementation | Verification |
| --- | --- | --- |
| Build fingerprinting | `BuildFingerprint`, executable SHA-256 hashing, runtime and KFC identity | fingerprint unit tests |
| Probe contracts | `ProbeManifest`, lifecycle state, permissions, build target | manifest validation and schema tests |
| Schema generation | JSON Schema for probe and adapter manifests | machine-readable schema tests; `eml-tool probe-schema` and `adapter-schema` |
| Persistent catalogs | `ResourceCatalog`, `CatalogStore`, sorted `CatalogIndex` | catalog persistence and query tests |
| Bounded scans | `ScanPolicy`, `ScanCursor`, page bounds and cancellation | bounded pagination and cancellation tests |
| Dependency graphs | catalog dependency edges and transitive queries | dependency and catalog tests |
| ShroudForge package generation | staged read-only package writer and validator | package generation and validation tests; `eml-tool stage-probe` |
| Evidence import | ShroudForge log marker parser with build binding | log import test; `eml-tool import-log` |
| Replay testing | `ReplayCase` and current-build evidence matching | offline replay test |
| Reliability safeguards | `RetryPolicy` and `ProbeRun` retry/backoff integration | retry and probe-run tests |
| Guarded operations | `OperationGate`, `GuardedOperation`, journal-compatible records | ordered preflight/apply/readback/rollback test |
| Cross-build comparison | `CatalogDiff` for resources and dependency edges | catalog diff test; `eml-tool diff-catalog` |

## Verification commands

From the repository root:

```text
cargo test -p adapter-contract
cargo check --workspace
cargo check -p eml-tool
```

The CLI is deliberately filesystem-oriented and does not install or enable a
package in a live game. A ShroudForge runtime session is still required to
produce fresh runtime evidence; the repository tools only stage, validate,
import, and review that evidence.

## Safety boundary

The implementation does not perform arbitrary memory injection or bypass
ShroudForge permissions. Write-capable operations remain behind explicit
permission, approval, backup, build-match, readback, and rollback gates.
