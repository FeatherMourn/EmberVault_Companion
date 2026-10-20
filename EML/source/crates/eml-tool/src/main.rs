use adapter_contract::{AdapterManifest, BuildFingerprint, CatalogDiff, CatalogStore, ProbeEvidence, ProbeManifest, ResourceCatalog, ShroudForgePackageSpec};
use std::{env, fs, path::PathBuf, process::ExitCode};

fn usage() {
    eprintln!("eml-tool commands:");
    eprintln!("  probe-schema <output.json>");
    eprintln!("  adapter-schema <output.json>");
    eprintln!("  stage-probe <directory> <id> <name> <action-id> <action-label>");
    eprintln!("  validate-package <directory>");
    eprintln!("  index-catalog <catalog.json> <store-directory>");
    eprintln!("  diff-catalog <before.json> <after.json> <output.json>");
    eprintln!("  import-log <log.txt> <probe-id> <fingerprint.json> <evidence.json>");
}

fn required(args: &[String], index: usize) -> Result<&str, String> {
    args.get(index).map(String::as_str).ok_or_else(|| "missing argument".into())
}

fn run(args: &[String]) -> Result<(), String> {
    match required(args, 1)? {
        "probe-schema" => {
            let path = required(args, 2)?;
            fs::write(path, serde_json::to_vec_pretty(&ProbeManifest::json_schema()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
        "adapter-schema" => {
            let path = required(args, 2)?;
            fs::write(path, serde_json::to_vec_pretty(&AdapterManifest::json_schema()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
        "stage-probe" => {
            let root = PathBuf::from(required(args, 2)?);
            let spec = ShroudForgePackageSpec {
                id: required(args, 3)?.to_owned(),
                name: required(args, 4)?.to_owned(),
                version: "0.1.0".parse().map_err(|e| format!("invalid version: {e}"))?,
                description: "Generated read-only EML probe".into(),
                action_id: required(args, 5)?.to_owned(),
                action_label: required(args, 6)?.to_owned(),
            };
            let path = spec.write_read_only_probe(root).map_err(|e| e.to_string())?;
            println!("staged {}", path.display());
        }
        "validate-package" => {
            let path = required(args, 2)?;
            ShroudForgePackageSpec::validate_staged_package(path).map_err(|e| e.to_string())?;
            println!("valid {path}");
        }
        "index-catalog" => {
            let catalog: ResourceCatalog = serde_json::from_slice(&fs::read(required(args, 2)?).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let index = CatalogStore::open(required(args, 3)?).write(&catalog).map_err(|e| e.to_string())?;
            println!("indexed {} entries", index.entries.len());
        }
        "diff-catalog" => {
            let before: ResourceCatalog = serde_json::from_slice(&fs::read(required(args, 2)?).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let after: ResourceCatalog = serde_json::from_slice(&fs::read(required(args, 3)?).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let diff = CatalogDiff::between(&before, &after);
            fs::write(required(args, 4)?, serde_json::to_vec_pretty(&diff).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            println!("diff added={} removed={} changed={}", diff.added.len(), diff.removed.len(), diff.changed.len());
        }
        "import-log" => {
            let log = fs::read_to_string(required(args, 2)?).map_err(|e| e.to_string())?;
            let probe_id = required(args, 3)?;
            let fingerprint: BuildFingerprint = serde_json::from_slice(
                &fs::read(required(args, 4)?).map_err(|e| e.to_string())?,
            ).map_err(|e| e.to_string())?;
            let evidence = ProbeEvidence::from_shroudforge_log(&log, probe_id, fingerprint)
                .ok_or_else(|| "no matching ShroudForge evidence marker found".to_string())?;
            fs::write(required(args, 5)?, evidence.to_json().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            println!("imported {:?} evidence", evidence.state);
        }
        command => return Err(format!("unknown command: {command}")),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        usage();
        return ExitCode::from(2);
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            usage();
            ExitCode::from(1)
        }
    }
}
