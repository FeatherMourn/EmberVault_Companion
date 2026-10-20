# EML 2.0 Roadmap

## Purpose

EML 2.0 is intended to be a stronger, build-aware data and discovery layer
for Enshrouded mod development. It should make KFC resources, reflected
fields, schemas, and adapter metadata easier to inspect and consume without
claiming access to undocumented native hooks.

EML 2.0 does not replace ShroudForge as the in-game runtime. Its output is
consumed by the ShroudForge bridge and EmberVault tools.

## Design goals

- Provide reliable, typed access to discoverable game data.
- Make results reproducible across game builds.
- Keep large scans bounded, cancellable, cacheable, and inspectable.
- Preserve provenance for every catalog, field, and generated mapping.
- Separate read-only discovery from any state-changing operation.
- Produce stable machine-readable contracts for adapters and probes.
- Fail clearly when a capability is unavailable or build-incompatible.

## Roadmap phases

### Phase 1 — Baseline and compatibility

- Record supported game, KFC, and EML versions.
- Define build fingerprints and compatibility profiles.
- Establish the public API surface and deprecation policy.
- Add baseline fixtures from known KFC4 exports.
- Document what is proven, inferred, and unsupported.

### Phase 2 — KFC parser foundation

- Harden KFC3 and KFC4 parsing and validation.
- Provide clear errors for malformed, incomplete, or mismatched exports.
- Support descriptor inspection without modifying source game files.
- Add deterministic catalog and schema output.
- Add parser regression fixtures for each supported build.

### Phase 3 — Typed resource and reflection layer

- Define typed resource, field, reference, part, and relationship models.
- Normalize GUIDs, qualified type names, and part indexes.
- Expose safe field reads with explicit missing, null, and unsupported states.
- Generate schemas and field documentation from reflection metadata.
- Track field provenance back to the source export and build.

### Phase 4 — Scalable discovery

- Add bounded sampling and pagination for large resource families.
- Add caching, incremental refresh, cancellation, and progress reporting.
- Support filtered searches by type, field, relationship, and identifier.
- Generate dependency and reference graphs.
- Keep memory use predictable for large KFC exports.

### Phase 5 — Adapter and probe contracts

- Define machine-readable adapter and probe manifests.
- Generate review-only adapter candidates from verified schemas.
- Declare required fields, supported builds, permissions, and fallbacks.
- Add contract validation before packaging.
- Distinguish discovered, mapped, packaged, executed, and verified states.

### Phase 6 — ShroudForge bridge

- Convert EML-derived read-only probes into native ShroudForge modules.
- Generate standard actions, lifecycle logs, and result markers.
- Capture runtime results with build and version identity.
- Report discovery separately from actual execution.
- Keep EML export behavior optional and isolated from the native runtime path.

### Phase 7 — Evidence and quality

- Attach confidence, limitations, timestamps, and provenance to results.
- Add replayable offline tests from recorded fixtures.
- Add cross-build diff reports and stale-result detection.
- Add fuzz and malformed-input testing for parser boundaries.
- Publish diagnostic bundles that do not contain raw proprietary game data.

### Phase 8 — Controlled operations

- Keep all writes disabled by default.
- Define explicit permission profiles for read-only, staged, and confirmed use.
- Require exact build matching, verified backup, readback, and rollback evidence.
- Expose only documented or independently verified operations.
- Refuse arbitrary pointer access, guessed offsets, and unsupported mutation.

### Phase 9 — Release and maintenance

- Version EML 2.0 independently from ShroudForge and EmberVault.
- Publish migration notes and compatibility matrices.
- Maintain adapters for supported game builds.
- Deprecate old APIs with clear warnings and transition guidance.
- Record regressions and update fixtures when the game changes.

## Initial deliverables

1. Versioned EML 2.0 API and schema specification.
2. KFC4 parser validation and fixture suite.
3. Typed resource and reflection model.
4. Bounded catalog scanner with provenance.
5. Adapter and probe manifest schema.
6. One native ShroudForge read-only probe generated from EML metadata.
7. Compatibility and evidence report format.

## Success criteria

EML 2.0 is successful when a developer can start with a build-matched KFC
export, inspect a resource and its fields, generate a reviewable adapter
candidate, package a native ShroudForge read-only probe, execute it in-game,
and receive evidence that clearly states what was observed and what remains
unverified.
