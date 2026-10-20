# EML 2.0 Action Plan

## Mission

Advance EML from a loader and resource-access toolkit into a build-aware,
observable, and reliable data platform for Enshrouded mod development.

EML 2.0 will discover and describe game data. ShroudForge will remain the
runtime host for in-game modules. EmberVault Tools will coordinate packaging,
testing, evidence, permissions, and rollback.

## Operating principles

- Read-only discovery is the default.
- Every result carries game build, EML version, source, timestamp, and limits.
- Package discovery is not proof of execution.
- Generated adapters require review before approval.
- Writes require exact build matching, backup, confirmation, readback, and
  rollback evidence.
- Undocumented native behavior is recorded as unknown, never guessed.

## Milestone 0 — Baseline and governance

### Actions

- Freeze the v0.2.0 baseline on the `embervault-eml-2.0-dev` branch.
- Record compiler, dependency, platform, and release metadata.
- Turn current warnings into tracked cleanup tasks.
- Define API stability, deprecation, licensing, and contribution rules.
- Create a compatibility profile format for game and runtime builds.

### Exit criteria

- Workspace builds cleanly or every remaining warning is documented.
- A reproducible baseline report is stored with the branch.
- No live game files are changed by development or tests.

## Milestone 1 — Build-aware KFC foundation

### Actions

- Harden KFC3/KFC4 parsing and malformed-input errors.
- Add deterministic catalog and schema exports.
- Record source export identity and parser version.
- Add fixtures for each supported game build.
- Add build fingerprints and cross-build diff output.

### Exit criteria

- The same input produces the same catalog and schema output.
- Invalid and mismatched exports fail with actionable diagnostics.
- A build diff identifies added, removed, and changed resource definitions.

## Milestone 2 — Typed resource and reflection API

### Actions

- Define typed resource, field, reference, part, and relationship models.
- Normalize GUIDs, qualified type names, and part indexes.
- Expose safe missing, null, unsupported, and error states.
- Generate field documentation from reflection metadata.
- Preserve provenance from every field to its source export.

### Exit criteria

- Consumers can inspect resources without relying on stringly typed guesses.
- Field access is covered by unit and fixture tests.
- Unsupported types are reported instead of silently coerced.

## Milestone 3 — Scalable discovery engine

### Actions

- Add bounded sampling, pagination, and filtering.
- Add lazy loading, persistent indexes, caching, and cancellation.
- Add progress reporting and memory limits.
- Generate resource dependency and reference graphs.
- Add offline replay from recorded catalogs.

### Exit criteria

- Large catalogs do not block startup or exhaust memory.
- A scan can be canceled safely and resumed from cached state.
- Results remain deterministic and provenance-aware.

## Milestone 4 — Adapter and probe platform

### Actions

- Define machine-readable adapter and probe manifests.
- Declare capabilities, required fields, permissions, build targets, and
  fallbacks.
- Generate review-only adapter candidates from verified schemas.
- Add contract validation and candidate status transitions.
- Add a CLI for discovery, validation, schema export, and package generation.

### Exit criteria

- Every generated candidate explains its source mapping and limitations.
- Invalid candidates cannot be packaged.
- Candidate generation never grants runtime permissions automatically.

## Milestone 5 — ShroudForge integration

### Actions

- Create native ShroudForge module and probe templates.
- Convert EML-derived probes into ShroudForge packages.
- Standardize lifecycle events, actions, logs, and completion markers.
- Capture runtime results with build and module identity.
- Distinguish discovered, loaded, enabled, executed, and verified states.

### Exit criteria

- A read-only probe can be generated, installed in isolation, run in-game,
  and imported as evidence without manual file editing.
- Failed execution produces a diagnostic result rather than a silent absence.

## Milestone 6 — Reliability and security

### Actions

- Add timeouts, watchdogs, crash isolation, and failure recovery.
- Enforce capability-based permissions and read-only defaults.
- Add dependency, conflict, and compatibility checks.
- Add structured operation journals and diagnostic bundles.
- Refuse arbitrary pointer access and unsupported memory mutation.

### Exit criteria

- A failing probe cannot block unrelated modules.
- Permissions are visible and fail closed.
- Every failure leaves enough evidence to reproduce or diagnose it.

## Milestone 7 — Guarded operations

### Actions

- Define read-only, staged, and live-confirmed operation profiles.
- Automate verified backups before writes.
- Require exact build matching and explicit confirmation.
- Require readback verification after applying a value.
- Require rollback rehearsal and preserved transaction history.

### Exit criteria

- No write-capable adapter can bypass safety gates.
- A controlled operation can be reversed and verified.
- Multiplayer or authority limitations are explicit.

## Milestone 8 — EmberVault Tools integration

### Actions

- Add project, package, profile, and evidence management.
- Provide a guided candidate-to-verified-probe workflow.
- Add compatibility matrices, stale-evidence warnings, and build diffs.
- Add schema browsing and adapter review screens.
- Export reproducible diagnostic and evidence bundles.

### Exit criteria

- A developer can complete the discovery-to-evidence workflow from one tool.
- The UI always shows the next action and the gate blocking progress.
- Mapping data remains separate from the EML source and runtime packages.

## Milestone 9 — Release and long-term evolution

### Actions

- Publish EML 2.0 alpha, beta, and stable milestones.
- Maintain migration notes and compatibility matrices.
- Add regression testing for every supported game build.
- Version the EML API independently from ShroudForge and EmberVault.
- Preserve upstream GPL notices and document all derived code.

### Exit criteria

- Releases are reproducible and source-complete under the applicable license.
- Users can identify supported builds before installation.
- Breaking changes have migration guidance.

## Ultimate capability target

The mature system should let a developer start with a build-matched KFC
export, inspect typed resources and relationships, generate a reviewable
adapter, package a native ShroudForge probe, run it in an isolated session,
collect fresh evidence, compare it across builds, and—only when explicitly
approved—perform a reversible guarded operation.

The system must remain honest about its limits: it cannot guarantee access to
undocumented native hooks, every live object, multiplayer authority, or future
game builds without new evidence.
