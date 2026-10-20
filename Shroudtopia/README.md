# Shroudtopia

This folder contains the Shroudtopia runtime project and its planned integration with EML 2.0.

## Purpose

Upgrade Shroudtopia into a stronger, build-aware runtime for Enshrouded while keeping EML 2.0 as the data, parsing, schema, and mapping layer.

## Planned responsibilities

- Native Shroudtopia module lifecycle and runtime hosting
- EML 2.0 compatibility contracts and manifest translation
- Read-only probes, structured logs, and evidence capture
- Blueprint and world-editing experiments behind explicit permissions
- Build profiles, capability negotiation, and rollback-safe operations

## Boundaries

Shroudtopia remains the in-game runtime. EML 2.0 remains the discovery and data layer. EmberVault Tools coordinates packaging, testing, provenance, and release evidence. Any reused MIT-licensed Shroudtopia code must retain its copyright and license notices.

## Initial milestones

1. Build and validate a native Shroudtopia development module.
2. Load one read-only EML 2.0 probe through the Shroudtopia lifecycle.
3. Add structured result export and build-bound evidence.
4. Add typed resource access and compatibility negotiation.
5. Prototype blueprint viewing and export before attempting placement.

This project is separate from the EmberVault game-mapping snapshots and the earlier ShroudForge integration work.
