# Evidence History

## 2026-10-21 — Standard EML export baseline

- **Probe:** `eml-export-item-list`
- **Result:** Loaded under ShroudForge and generated `eml-item-list.csv`.
- **Established:** Standard EML manifest discovery, Lua execution, item
  resource reads, and export behavior.
- **Evidence level:** Verified for the tested environment.
- **Not established:** Full EML 2.0 bridge runtime, live world reads,
  placement, save mutation, or multiplayer authority.

## 2026-10-20 — Bridge schema compatibility check

- **Probe:** EML 2.0 bridge probe
- **Result:** Staged package validated by EmberVault tooling, but the live
  ShroudForge schema rejected the optional compatibility metadata and custom
  capability label.
- **Established:** The current loader requires its accepted manifest schema.
- **Evidence level:** Observed/static.
- **Decision:** Keep EML 2.0 metadata in staging until a real runtime bridge
  exists; use native-compatible manifests for live probes.

## Snapshot policy

Add a new entry for each meaningful probe or build change. Do not overwrite a
prior conclusion; record corrections and regressions as new entries.
