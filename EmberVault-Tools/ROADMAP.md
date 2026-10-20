# EmberVault Tools Roadmap

## Purpose

EmberVault Tools is the coordination layer between EML 2.0 and ShroudForge.
It manages adapter development, probe packaging, compatibility, evidence,
backups, and guarded promotion without replacing either runtime or parser.

## Roadmap phases

### Phase 1 — Shared foundation

- Define adapter, probe, evidence, build, permission, and lifecycle contracts.
- Add machine-readable project, package, and capability registries.
- Track discovered, mapped, packaged, installed, executed, and verified states.

### Phase 2 — Project and version management

- Organize EML, ShroudForge, and EmberVault projects independently.
- Track versions, builds, dependencies, profiles, and migration notes.
- Preserve provenance for every imported catalog and generated artifact.

### Phase 3 — Adapter generation

- Import verified EML schemas and resource catalogs.
- Generate review-only adapter candidates with mappings and limitations.
- Validate contracts, required fields, permissions, and target builds.
- Keep generated candidates separate from approved adapters.

### Phase 4 — Probe packaging and execution

- Generate native ShroudForge probe packages.
- Validate manifests and package contents before installation.
- Install into isolated profiles without overwriting existing modules.
- Enable, run, and read results through one guided workflow.

### Phase 5 — Evidence and diagnostics

- Collect structured runtime logs and result markers.
- Record build identity, module version, timestamps, confidence, and limits.
- Distinguish discovery from execution and execution from verification.
- Export diagnostic bundles for review and reproducibility.

### Phase 6 — Compatibility and maintenance

- Maintain game and ShroudForge compatibility matrices.
- Generate build-diff reports when the game changes.
- Detect stale evidence, conflicts, missing dependencies, and failed contracts.
- Preserve regression history and migration guidance.

### Phase 7 — Guided workflows

- Provide a simple path from candidate to verified probe.
- Show the exact next action and the gate blocking progress.
- Keep advanced controls available without overwhelming the main workflow.
- Surface errors in context with recovery guidance.

### Phase 8 — Guarded operations

- Support read-only, staged, and live-confirmed permission profiles.
- Create and verify backups automatically before writes.
- Require exact build matching, explicit confirmation, readback, and rollback.
- Maintain an append-only operation journal.

### Phase 9 — Release and maintenance

- Package reproducible tool releases with versioned contracts.
- Document supported and unsupported capabilities.
- Provide migration checks for schema and runtime changes.
- Keep all results and artifacts provenance-aware.

## Initial deliverables

1. Shared adapter, probe, evidence, and build registries.
2. EML-to-ShroudForge package generation workflow.
3. One guided install, enable, run, and result-review path.
4. Compatibility and stale-evidence reporting.
5. Diagnostic bundle export.
6. Guarded-operation journal with backup and rollback gates.

## Success criteria

EmberVault Tools is successful when a developer can select a verified EML
mapping, generate a native ShroudForge probe, install it safely, run it in an
isolated session, review fresh evidence, and understand exactly what remains
unsupported before any state-changing operation is considered.
