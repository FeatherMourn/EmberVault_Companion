# Ember Vault Companion

Ember Vault Companion is a separate coordination project for building a
stronger, build-aware Enshrouded modding workflow. It develops interoperability
between EML 2.0, ShroudForge, and EmberVault tools without merging loaders.

## Goal

Discover game data, turn it into reviewable adapter definitions, package
read-only probes as native ShroudForge modules, run them in controlled sessions,
and preserve evidence. Unsupported behavior remains clearly labeled.

## Architecture

- **EML 2.0:** KFC parsing, reflection, typed resource catalogs, and schemas.
- **ShroudForge:** in-game runtime, module lifecycle, actions, and logging.
- **EmberVault tools:** package generation, compatibility, evidence, backups,
  and guarded promotion.

EML-derived data is converted into native ShroudForge packages before testing.
The Companion documents and tests the contract between these layers.

## Standard workflow

1. Identify the exact game and ShroudForge build.
2. Import or generate EML/KFC metadata with provenance.
3. Create a bounded read-only probe or adapter candidate.
4. Validate its contract, permissions, manifest, and build compatibility.
5. Package it as a native ShroudForge module.
6. Install it in an isolated profile or disposable world.
7. Run it and collect structured evidence.
8. Review the result before promotion.
9. Require backup, confirmation, readback, and rollback for writes.

## Roadmap

### Phase 1 — Contracts and foundations

Define adapter, probe, capability, evidence, build, permission, and lifecycle
contracts with deterministic validation and machine-readable reports.

### Phase 2 — EML 2.0 data layer

Upgrade KFC parsing and reflection; generate typed catalogs and build-specific
schemas; add bounded scans, caching, sampling, cancellation, and provenance.

### Phase 3 — ShroudForge runtime layer

Create native module templates, standardize actions and lifecycle logging, add
compatibility checks, and convert the existing read-only probes.

### Phase 4 — EML/ShroudForge bridge

Translate EML schemas into ShroudForge packages, validate contents, and read
runtime results back into structured evidence.

### Phase 5 — Control Center integration

Add project selection, package generation, installation, result review,
compatibility matrices, evidence history, and diagnostic bundles.

### Phase 6 — Guarded operations

Add permission profiles, verified backups, readback checks, transaction
journals, and rollback. Enable writes only after contracts pass.

### Phase 7 — Release and maintenance

Publish versioned releases, test supported game and ShroudForge builds, record
regressions, and document unsupported capabilities.

## Boundaries and success criteria

We do not assume every KFC field is safe to edit, that a resource is a native
pointer, or that package discovery means execution was verified. Arbitrary
memory writes remain outside the supported workflow. A capability is successful
when its source, contract, package, fresh build-matched result, and rollback
path are all documented.
