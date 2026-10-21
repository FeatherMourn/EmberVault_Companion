# Capability Matrix

Snapshot: `1.0`  
Scope: EML 2.0, ShroudForge, EmberVault tooling, and content-generation workflows  
Status: living record

## Evidence levels

| Level | Meaning |
|---|---|
| Static | Source, schema, documentation, or offline catalog evidence |
| Synthetic | Staged fixture or local simulation |
| Observed | Behavior seen in a running game session |
| Verified | Reproducible current-build result with matching provenance |
| Guarded | Controlled operation with permission, backup, readback, and rollback |

## Current matrix

| Area | Capability | Status | Evidence | Limits and next test |
|---|---|---|---|---|
| Loading | Standard EML package discovery | Confirmed | Observed | Must recheck after loader/game updates |
| Lua | Standard EML Lua execution | Confirmed | Observed | Only APIs actually exercised are claimed |
| Assets | Read resource records by type | Confirmed | Verified | Current probe covered item resources |
| Export | Export generated data to CSV | Confirmed | Verified | `eml-item-list.csv` was produced by the live session |
| Logging | ShroudForge log output | Confirmed | Observed | Marker format may vary by runtime version |
| Lifecycle | Package startup and enable status | Confirmed | Observed | Full EML 2.0 lifecycle remains a separate target |
| EML 2.0 | Bridge metadata in a package | Staged only | Synthetic | Older ShroudForge schemas reject extra compatibility fields |
| EML 2.0 | Full runner inside ShroudForge | Unverified | None | Requires a real loader/runtime integration |
| Catalogs | KFC/resource normalization | Confirmed offline | Static/Synthetic | Catalog identity must match the game build |
| Schemas | Typed fields, dependencies, and provenance | Confirmed offline | Static/Synthetic | Not proof of live memory access or writability |
| UI | Native ShroudForge actions and mod status | Confirmed | Observed | Custom fields must use the accepted schema |
| Blueprint metadata | Offline preview and metadata generation | Confirmed offline | Synthetic | Live-world capture is not established here |
| Blueprint export | Export an existing verified snapshot | Conditional | Synthetic | Requires a valid snapshot source |
| World reads | Live voxel/entity/prop reads | Unverified | Failed/blocked provider attempts | Needs an independently validated current-build provider |
| Placement | Live blueprint placement | Unsupported for now | None | Keep disabled until read, permission, and rollback gates pass |
| Writes | Game or save mutation | Unsupported for now | None | No write path is promoted |
| Multiplayer | Authority-aware edits | Unsupported for now | None | Requires explicit authority and rollback evidence |
| Content generation | Manifests, Lua modules, catalogs, exports | Supported | Static/Observed/Verified by area | Generated package still needs loader validation |
| Content generation | New native game assets | Unverified | None | Requires an exposed and documented asset pipeline |
| Guarded operations | Backup/readback/rollback workflow | Designed | Static/Synthetic | No live write capability is enabled |

## Rules

1. A catalog record is not proof that a value is writable.
2. Package discovery is not proof that the package executed.
3. Older-build evidence cannot establish current-build support.
4. Unsupported and unverified capabilities remain visible rather than being
   silently treated as failures or successes.
5. Every promotion must add a history entry naming the probe, build context,
   result, and remaining limitations.
