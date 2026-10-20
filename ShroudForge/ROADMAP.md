# ShroudForge Runtime Roadmap

## Purpose

This roadmap describes how to strengthen ShroudForge as the in-game runtime
for verified Enshrouded modules, probes, and adapters. ShroudForge remains
responsible for loading and running modules; EML provides data and mapping
metadata, while EmberVault tools coordinate development and evidence.

## Design goals

- Make module behavior predictable and build-aware.
- Provide stable lifecycle, action, logging, and result conventions.
- Make read-only probes easy to package, install, run, and verify.
- Separate package discovery from actual runtime execution.
- Fail closed when a build, capability, permission, or dependency is missing.
- Keep state-changing operations explicit, reviewable, and reversible.

## Roadmap phases

### Phase 1 — Runtime baseline

- Record supported ShroudForge and game builds.
- Define module identity, version, target, and dependency metadata.
- Document manifest, source layout, action, and logging conventions.
- Add a compatibility profile and deterministic runtime fingerprint.
- Define supported, experimental, and unsupported capability states.

### Phase 2 — Module SDK

- Provide templates for native ShroudForge modules and read-only probes.
- Standardize startup, shutdown, reload, and error lifecycle events.
- Add action registration with labels, permissions, and clear results.
- Provide structured info, warning, error, and debug logging.
- Add validation before a package can be installed.

### Phase 3 — Capability and compatibility system

- Negotiate capabilities with the current game and runtime build.
- Check dependencies, conflicts, permissions, and supported targets.
- Distinguish discovered, loaded, enabled, executed, and verified states.
- Produce actionable failure messages instead of silent discovery.
- Expose a machine-readable capability report.

### Phase 4 — Probe runtime

- Provide bounded, cancellable probe execution.
- Add standard start, progress, sample, completion, and failure markers.
- Keep large scans from blocking startup or action registration.
- Support result summaries, small samples, and structured evidence output.
- Add a safe batch runner for independent read-only probes.

### Phase 5 — EML bridge

- Accept EML-generated schemas and adapter metadata.
- Generate ShroudForge actions from approved probe contracts.
- Resolve resource and field references through declared mappings.
- Return runtime results with build, module, and evidence identity.
- Keep EML export behavior optional and separate from native logging.

### Phase 6 — In-game UI and diagnostics

- Provide consistent Modloader entries and action panels.
- Show module status, capability readiness, and recent results.
- Add an accessible diagnostic view for errors and blocked operations.
- Support safe reload and refresh behavior where the runtime allows it.
- Make logs and result markers easy for EmberVault to collect.

### Phase 7 — Guarded operations

- Define read-only, staged, and live-confirmed permission profiles.
- Require exact build matching before any write-capable action.
- Require an automatically verified backup and explicit confirmation.
- Require readback verification, transaction journaling, and rollback.
- Refuse arbitrary memory writes and undocumented native operations.

### Phase 8 — Testing and release quality

- Add offline manifest and contract tests.
- Add runtime smoke tests for each supported build.
- Add regression tests for lifecycle, logging, actions, and failures.
- Record compatibility evidence for each release.
- Publish migration notes when module contracts change.

### Phase 9 — Maintenance

- Track game and ShroudForge build changes independently.
- Deprecate old APIs with warnings and migration guidance.
- Maintain a compatibility matrix for modules and capabilities.
- Preserve failed and unsupported results for future diagnosis.
- Keep release artifacts reproducible and provenance-aware.

## Initial deliverables

1. ShroudForge module and probe SDK specification.
2. Clean read-only probe template with bounded execution.
3. Structured lifecycle and result logging convention.
4. Manifest and capability validation tool.
5. Build compatibility and dependency report.
6. EML bridge contract for generated probe packages.
7. EmberVault evidence importer for verified runtime results.

## Success criteria

ShroudForge is successful when a developer can validate a module, install it
into an isolated profile, run a clearly labeled action, receive a fresh
build-matched result, and understand exactly whether the module was discovered,
loaded, executed, or verified.
