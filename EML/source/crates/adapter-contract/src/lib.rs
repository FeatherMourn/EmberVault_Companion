//! Machine-readable contracts shared by EML discovery and runtime bridges.
//!
//! This crate is deliberately runtime-neutral: it describes what an adapter
//! needs and what it proved, but does not perform game operations.

use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::{Path, PathBuf}};
use std::io::{BufRead, BufReader, Write};

pub const CONTRACT_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogEntry {
    pub id: String,
    pub qualified_type: String,
    pub fields: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResourceCatalog {
    pub schema_version: String,
    pub fingerprint: Option<BuildFingerprint>,
    pub entries: Vec<CatalogEntry>,
    pub dependencies: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogIndexEntry {
    pub id: String,
    pub qualified_type: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogIndex {
    pub schema_version: String,
    pub fingerprint: Option<BuildFingerprint>,
    pub entries: Vec<CatalogIndexEntry>,
}

impl CatalogIndex {
    pub fn from_catalog(catalog: &ResourceCatalog) -> Self {
        let mut entries = catalog.entries.iter().map(|entry| CatalogIndexEntry {
            id: entry.id.clone(),
            qualified_type: entry.qualified_type.clone(),
            fields: entry.fields.clone(),
        }).collect::<Vec<_>>();
        entries.sort_by(|left, right| left.id.cmp(&right.id));
        Self { schema_version: catalog.schema_version.clone(), fingerprint: catalog.fingerprint.clone(), entries }
    }

    pub fn find(&self, id: &str) -> Option<&CatalogIndexEntry> {
        self.entries.binary_search_by(|entry| entry.id.as_str().cmp(id)).ok().map(|index| &self.entries[index])
    }

    pub fn by_type(&self, qualified_type: &str) -> Vec<&CatalogIndexEntry> {
        self.entries.iter().filter(|entry| entry.qualified_type == qualified_type).collect()
    }
}

#[derive(Debug, Clone)]
pub struct CatalogStore {
    root: PathBuf,
}

impl CatalogStore {
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn write(&self, catalog: &ResourceCatalog) -> Result<CatalogIndex, CatalogError> {
        fs::create_dir_all(&self.root)?;
        catalog.save_json(self.root.join("catalog.json"))?;
        let index = CatalogIndex::from_catalog(catalog);
        let index_path = self.root.join("index.json");
        let temporary = self.root.join("index.json.tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(&index)?)?;
        fs::rename(temporary, index_path)?;
        Ok(index)
    }

    pub fn load_catalog(&self) -> Result<ResourceCatalog, CatalogError> {
        ResourceCatalog::load_json(self.root.join("catalog.json"))
    }

    pub fn load_index(&self) -> Result<CatalogIndex, CatalogError> {
        Ok(serde_json::from_slice(&fs::read(self.root.join("index.json"))?)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogDiff {
    pub from_fingerprint: Option<BuildFingerprint>,
    pub to_fingerprint: Option<BuildFingerprint>,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    pub added_dependencies: Vec<(String, String)>,
    pub removed_dependencies: Vec<(String, String)>,
}

impl CatalogDiff {
    pub fn between(from: &ResourceCatalog, to: &ResourceCatalog) -> Self {
        let from_entries = from.entries.iter().map(|entry| (&entry.id, entry)).collect::<std::collections::BTreeMap<_, _>>();
        let to_entries = to.entries.iter().map(|entry| (&entry.id, entry)).collect::<std::collections::BTreeMap<_, _>>();
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();

        for (id, entry) in &to_entries {
            match from_entries.get(id) {
                None => added.push((*id).clone()),
                Some(previous) if *previous != *entry => changed.push((*id).clone()),
                Some(_) => {}
            }
        }
        for id in from_entries.keys() {
            if !to_entries.contains_key(id) {
                removed.push((*id).clone());
            }
        }

        let from_dependencies = from.dependencies.iter().cloned().collect::<BTreeSet<_>>();
        let to_dependencies = to.dependencies.iter().cloned().collect::<BTreeSet<_>>();
        let mut added_dependencies = to_dependencies.difference(&from_dependencies).cloned().collect::<Vec<_>>();
        let mut removed_dependencies = from_dependencies.difference(&to_dependencies).cloned().collect::<Vec<_>>();
        added_dependencies.sort();
        removed_dependencies.sort();
        added.sort();
        removed.sort();
        changed.sort();

        Self {
            from_fingerprint: from.fingerprint.clone(),
            to_fingerprint: to.fingerprint.clone(),
            added,
            removed,
            changed,
            added_dependencies,
            removed_dependencies,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && self.added_dependencies.is_empty()
            && self.removed_dependencies.is_empty()
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), CatalogError> {
        let path = path.as_ref();
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        fs::rename(temporary, path)?;
        Ok(())
    }
}

impl ResourceCatalog {
    pub fn new(fingerprint: Option<BuildFingerprint>) -> Self {
        Self {
            schema_version: "1".into(),
            fingerprint,
            entries: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn from_type_registry(
        registry: &kfc::reflection::TypeRegistry,
        source: impl Into<String>,
        fingerprint: Option<BuildFingerprint>,
    ) -> Self {
        let source = source.into();
        let mut catalog = Self::new(fingerprint);
        for metadata in registry.iter() {
            let id = metadata.qualified_name.clone();
            let fields = metadata.struct_fields.keys().cloned().collect::<Vec<_>>();
            catalog.add_entry(CatalogEntry {
                id: id.clone(),
                qualified_type: metadata.qualified_name.clone(),
                fields,
                source: source.clone(),
            });
            for field in metadata.struct_fields.values() {
                if let Some(target) = registry.get(field.r#type) {
                    catalog.add_dependency(id.clone(), target.qualified_name.clone());
                }
            }
        }
        catalog
    }

    pub fn add_entry(&mut self, entry: CatalogEntry) {
        self.entries.retain(|existing| existing.id != entry.id);
        self.entries.push(entry);
    }

    pub fn add_dependency(&mut self, from: impl Into<String>, to: impl Into<String>) {
        let edge = (from.into(), to.into());
        if !self.dependencies.contains(&edge) {
            self.dependencies.push(edge);
        }
    }

    pub fn type_names(&self) -> BTreeSet<&str> {
        self.entries.iter().map(|entry| entry.qualified_type.as_str()).collect()
    }

    pub fn dependencies_of(&self, id: &str) -> BTreeSet<&str> {
        self.dependencies
            .iter()
            .filter(|(from, _)| from == id)
            .map(|(_, to)| to.as_str())
            .collect()
    }

    pub fn dependents_of(&self, id: &str) -> BTreeSet<&str> {
        self.dependencies
            .iter()
            .filter(|(_, to)| to == id)
            .map(|(from, _)| from.as_str())
            .collect()
    }

    pub fn transitive_dependencies_of(&self, id: &str) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut pending = self.dependencies_of(id).into_iter().map(str::to_owned).collect::<Vec<_>>();
        while let Some(current) = pending.pop() {
            if result.insert(current.clone()) {
                pending.extend(self.dependencies_of(&current).into_iter().map(str::to_owned));
            }
        }
        result
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), CatalogError> {
        let path = path.as_ref();
        let temp = path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(self)?;
        fs::write(&temp, data)?;
        fs::rename(temp, path)?;
        Ok(())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("catalog I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("catalog JSON failed: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShroudForgePackageSpec {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub action_id: String,
    pub action_label: String,
}

impl ShroudForgePackageSpec {
    pub fn validate(&self) -> Result<(), String> {
        for (label, value) in [
            ("id", self.id.as_str()),
            ("name", self.name.as_str()),
            ("description", self.description.as_str()),
            ("action id", self.action_id.as_str()),
            ("action label", self.action_label.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("package {label} must not be empty"));
            }
        }
        if self.id.contains(['/', '\\', ':']) || self.action_id.contains(['/', '\\', ':']) {
            return Err("package and action ids must be filesystem-safe".into());
        }
        Ok(())
    }

    pub fn write_read_only_probe(&self, root: impl AsRef<Path>) -> Result<std::path::PathBuf, PackageError> {
        self.validate().map_err(PackageError::Invalid)?;
        let package = root.as_ref().join(&self.id);
        let source = package.join("src");
        fs::create_dir_all(&source)?;
        let manifest = serde_json::json!({
            "id": self.id,
            "name": self.name,
            "version": self.version.to_string(),
            "capabilities": ["read-only-probe"],
            "description": self.description,
        });
        let extended = serde_json::json!({
            "schemaVersion": 1,
            "enabled": false,
            "targets": ["client"],
            "launcher": "SF",
            "groups": [{"label": "Probe", "description": "Read-only generated probe.", "actions": [{"id": self.action_id, "label": self.action_label, "style": "secondary"}]}]
        });
        fs::write(package.join("mod.json"), serde_json::to_vec_pretty(&manifest)?)?;
        fs::write(package.join("extended.mod.json"), serde_json::to_vec_pretty(&extended)?)?;
        let lua = format!(
            "-- Generated read-only probe.\nshroudforge.ui.on_action(\"{}\", function()\n  shroudforge.log.info(\"{} | collection_status=completed | read_only=true\")\nend)\n",
            self.action_id, self.id
        );
        fs::write(source.join("mod.lua"), lua)?;
        Ok(package)
    }

    pub fn validate_staged_package(path: impl AsRef<Path>) -> Result<(), PackageError> {
        let path = path.as_ref();
        for relative in ["mod.json", "extended.mod.json", "src/mod.lua"] {
            if !path.join(relative).is_file() {
                return Err(PackageError::Invalid(format!("missing staged package file: {relative}")));
            }
        }
        let manifest: serde_json::Value = serde_json::from_slice(&fs::read(path.join("mod.json"))?)?;
        if manifest["capabilities"].as_array().map_or(true, |values| !values.iter().any(|value| value == "read-only-probe")) {
            return Err(PackageError::Invalid("package is missing read-only-probe capability".into()));
        }
        let lua = fs::read_to_string(path.join("src/mod.lua"))?;
        if !lua.contains("read_only=true") {
            return Err(PackageError::Invalid("probe entry point is not marked read-only".into()));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("invalid package: {0}")]
    Invalid(String),
    #[error("package I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("package JSON failed: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum OperationState {
    Blocked,
    Prepared,
    Applied,
    Verified,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationRecord {
    pub operation_id: String,
    pub adapter_id: String,
    pub state: OperationState,
    pub fingerprint: BuildFingerprint,
    pub message: String,
}

pub struct OperationJournal;

impl OperationJournal {
    pub fn append(path: impl AsRef<Path>, record: &OperationRecord) -> Result<(), CatalogError> {
        let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
        file.sync_data()?;
        Ok(())
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Vec<OperationRecord>, CatalogError> {
        let file = fs::File::open(path)?;
        let mut records = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            if !line.trim().is_empty() {
                records.push(serde_json::from_str(&line)?);
            }
        }
        Ok(records)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardedOperation {
    pub operation_id: String,
    pub adapter_id: String,
    pub fingerprint: BuildFingerprint,
    pub gate: OperationGate,
    pub state: OperationState,
    pub message: String,
}

impl GuardedOperation {
    pub fn new(
        operation_id: impl Into<String>,
        adapter_id: impl Into<String>,
        fingerprint: BuildFingerprint,
        gate: OperationGate,
    ) -> Self {
        Self {
            operation_id: operation_id.into(),
            adapter_id: adapter_id.into(),
            fingerprint,
            gate,
            state: OperationState::Blocked,
            message: "guarded operation has not passed preflight".into(),
        }
    }

    pub fn preflight(&mut self) -> Result<(), String> {
        if self.gate.permission != Permission::LiveConfirmed
            || !self.gate.approved
            || !self.gate.backup_verified
            || !self.gate.build_matches
            || !self.gate.confirmation_received
        {
            self.state = OperationState::Blocked;
            self.message = format!("preflight blocked: {}", self.gate.missing_requirements().join(", "));
            return Err(self.message.clone());
        }
        self.state = OperationState::Prepared;
        self.message = "preflight passed; ready for one guarded apply".into();
        Ok(())
    }

    pub fn mark_applied(&mut self) -> Result<(), String> {
        if self.state != OperationState::Prepared {
            return Err("operation must pass preflight before apply".into());
        }
        self.state = OperationState::Applied;
        self.message = "apply completed; readback required".into();
        Ok(())
    }

    pub fn verify_readback(&mut self) -> Result<(), String> {
        if self.state != OperationState::Applied {
            return Err("readback requires an applied operation".into());
        }
        self.gate.readback_verified = true;
        self.state = OperationState::Verified;
        self.message = "readback verified; rollback required".into();
        Ok(())
    }

    pub fn verify_rollback(&mut self) -> Result<(), String> {
        if self.state != OperationState::Verified {
            return Err("rollback verification requires successful readback".into());
        }
        self.gate.rollback_verified = true;
        self.state = OperationState::RolledBack;
        self.message = "rollback verified; guarded operation complete".into();
        Ok(())
    }

    pub fn record(&self) -> OperationRecord {
        OperationRecord {
            operation_id: self.operation_id.clone(),
            adapter_id: self.adapter_id.clone(),
            state: self.state.clone(),
            fingerprint: self.fingerprint.clone(),
            message: self.message.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RunState {
    Created,
    Running,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeRun {
    pub run_id: String,
    pub probe_id: String,
    pub state: RunState,
    pub attempts: u32,
    pub max_attempts: u32,
    pub retry_policy: RetryPolicy,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl RetryPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_attempts == 0 || self.initial_backoff_ms == 0 || self.max_backoff_ms < self.initial_backoff_ms {
            return Err("retry policy must have positive attempts and ordered backoff bounds".into());
        }
        Ok(())
    }

    pub fn backoff_ms(&self, attempt: u32) -> u64 {
        let shift = attempt.min(63);
        self.initial_backoff_ms.saturating_mul(1u64 << shift).min(self.max_backoff_ms)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 3, initial_backoff_ms: 250, max_backoff_ms: 5_000 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationGate {
    pub permission: Permission,
    pub approved: bool,
    pub backup_verified: bool,
    pub build_matches: bool,
    pub confirmation_received: bool,
    pub readback_verified: bool,
    pub rollback_verified: bool,
}

impl OperationGate {
    pub fn new(permission: Permission) -> Self {
        Self {
            permission,
            approved: false,
            backup_verified: false,
            build_matches: false,
            confirmation_received: false,
            readback_verified: false,
            rollback_verified: false,
        }
    }

    pub fn ready_for_write(&self) -> bool {
        self.permission == Permission::LiveConfirmed
            && self.approved
            && self.backup_verified
            && self.build_matches
            && self.confirmation_received
            && self.readback_verified
            && self.rollback_verified
    }

    pub fn missing_requirements(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.permission != Permission::LiveConfirmed { missing.push("live-confirmed permission"); }
        if !self.approved { missing.push("adapter approval"); }
        if !self.backup_verified { missing.push("verified backup"); }
        if !self.build_matches { missing.push("matching build"); }
        if !self.confirmation_received { missing.push("explicit confirmation"); }
        if !self.readback_verified { missing.push("readback verification"); }
        if !self.rollback_verified { missing.push("rollback verification"); }
        missing
    }
}

impl ProbeRun {
    pub fn new(run_id: impl Into<String>, probe_id: impl Into<String>, max_attempts: u32) -> Result<Self, String> {
        if max_attempts == 0 {
            return Err("max attempts must be greater than zero".into());
        }
        let mut retry_policy = RetryPolicy::default();
        retry_policy.max_attempts = max_attempts;
        Ok(Self {
            run_id: run_id.into(),
            probe_id: probe_id.into(),
            state: RunState::Created,
            attempts: 0,
            max_attempts,
            retry_policy,
            error: None,
        })
    }

    pub fn retry_backoff_ms(&self) -> u64 {
        self.retry_policy.backoff_ms(self.attempts.saturating_sub(1))
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.state == RunState::Created {
            self.attempts += 1;
        } else if self.can_retry() {
            self.attempts += 1;
        } else {
            return Err("probe cannot start from its current state".into());
        }
        self.state = RunState::Running;
        self.error = None;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), String> {
        if self.state != RunState::Running {
            return Err("only a running probe can complete".into());
        }
        self.state = RunState::Completed;
        Ok(())
    }

    pub fn fail(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
        self.state = RunState::Failed;
    }

    pub fn can_retry(&self) -> bool {
        matches!(self.state, RunState::Failed | RunState::TimedOut) && self.attempts < self.max_attempts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Permission {
    ReadOnly,
    Staged,
    LiveConfirmed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleState {
    Draft,
    Packaged,
    Installed,
    Executed,
    Verified,
    Blocked,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildFingerprint {
    pub game_build: String,
    pub game_executable_sha256: Option<String>,
    pub runtime_name: String,
    pub runtime_version: String,
    pub kfc_export_id: Option<String>,
}

impl BuildFingerprint {
    pub fn from_executable(
        game_build: impl Into<String>,
        executable: impl AsRef<Path>,
        runtime_name: impl Into<String>,
        runtime_version: impl Into<String>,
        kfc_export_id: Option<String>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            game_build: game_build.into(),
            game_executable_sha256: Some(Self::sha256_file(executable)?),
            runtime_name: runtime_name.into(),
            runtime_version: runtime_version.into(),
            kfc_export_id,
        })
    }

    /// Hashes a game executable so compatibility checks can use file identity,
    /// not only a manually supplied display version.
    pub fn sha256_file(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
        let bytes = fs::read(path)?;
        let digest = Sha256::digest(bytes);
        Ok(format!("{digest:x}"))
    }

    pub fn matches(&self, other: &Self) -> bool {
        self.game_build == other.game_build
            && self.runtime_name == other.runtime_name
            && self.runtime_version == other.runtime_version
            && self.kfc_export_id == other.kfc_export_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceState {
    NotRun,
    Started,
    Partial,
    Completed,
    Failed,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeEvidence {
    pub probe_id: String,
    pub run_id: String,
    pub fingerprint: BuildFingerprint,
    pub state: EvidenceState,
    pub observed_at: String,
    pub summary: String,
    pub limitations: Vec<String>,
}

impl ProbeEvidence {
    pub fn is_current_for(&self, fingerprint: &BuildFingerprint) -> bool {
        self.fingerprint.matches(fingerprint) && self.state == EvidenceState::Completed
    }

    pub fn mark_stale_if_needed(&mut self, fingerprint: &BuildFingerprint) {
        if !self.fingerprint.matches(fingerprint) {
            self.state = EvidenceState::Stale;
        }
    }

    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_shroudforge_log(
        log: &str,
        probe_id: impl Into<String>,
        fingerprint: BuildFingerprint,
    ) -> Option<Self> {
        let probe_id = probe_id.into();
        let marker = log.lines().find(|line| line.contains(&probe_id))?;
        let state = if marker.contains("collection_status=completed") {
            EvidenceState::Completed
        } else if marker.contains("collection_status=failed") {
            EvidenceState::Failed
        } else if marker.contains("collection_status=started") {
            EvidenceState::Started
        } else {
            return None;
        };
        let summary = marker.split(" | ").last().unwrap_or(marker).to_owned();
        Some(Self {
            probe_id,
            run_id: format!("log-import-{}", fingerprint.game_build),
            fingerprint,
            state,
            observed_at: "imported-from-log".into(),
            summary,
            limitations: vec!["Imported from ShroudForge log marker".into()],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayCase {
    pub name: String,
    pub expected_probe_id: String,
    pub expected_state: EvidenceState,
    pub evidence_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayResult {
    pub case_name: String,
    pub passed: bool,
    pub reason: String,
}

impl ReplayCase {
    pub fn run(&self, current: &BuildFingerprint) -> ReplayResult {
        let evidence = match ProbeEvidence::from_json(&self.evidence_json) {
            Ok(value) => value,
            Err(error) => return ReplayResult { case_name: self.name.clone(), passed: false, reason: format!("invalid evidence: {error}") },
        };
        let passed = evidence.probe_id == self.expected_probe_id
            && evidence.state == self.expected_state
            && evidence.is_current_for(current);
        let reason = if passed { "evidence matched replay contract" } else { "evidence did not match replay contract" };
        ReplayResult { case_name: self.name.clone(), passed, reason: reason.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeManifest {
    pub contract_version: String,
    pub id: String,
    pub name: String,
    pub version: Version,
    pub source: String,
    pub read_only: bool,
    pub target: BuildTarget,
    pub capabilities: Vec<String>,
    pub evidence_state: EvidenceState,
}

impl ProbeManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err("probe id and name must not be empty".into());
        }
        if self.source.trim().is_empty() {
            return Err("probe source must not be empty".into());
        }
        if !self.read_only {
            return Err("EML discovery probes must be read-only".into());
        }
        if self.capabilities.is_empty() {
            return Err("probe must declare at least one capability".into());
        }
        Ok(())
    }

    pub fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://embervault.local/schemas/probe-manifest-0.1.json",
            "title": "EML Read-only Probe Manifest",
            "type": "object",
            "required": ["contract_version", "id", "name", "version", "source", "read_only", "target", "capabilities", "evidence_state"],
            "properties": {
                "contract_version": {"type": "string"},
                "id": {"type": "string", "minLength": 1},
                "name": {"type": "string", "minLength": 1},
                "version": {"type": "string"},
                "source": {"type": "string", "minLength": 1},
                "read_only": {"const": true},
                "target": {"type": "object"},
                "capabilities": {"type": "array", "minItems": 1},
                "evidence_state": {"enum": ["not-run", "started", "partial", "completed", "failed", "stale"]}
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildTarget {
    pub game_build: String,
    pub runtime_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterManifest {
    pub contract_version: String,
    pub id: String,
    pub name: String,
    pub version: Version,
    pub source: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<Permission>,
    pub targets: Vec<BuildTarget>,
    pub state: LifecycleState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanPolicy {
    pub max_items: usize,
    pub max_samples_per_type: usize,
    pub page_size: usize,
    pub allow_parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanCursor {
    pub offset: usize,
    pub total: Option<usize>,
    pub page_size: usize,
    pub cancelled: bool,
}

impl ScanCursor {
    pub fn new(policy: &ScanPolicy, total: Option<usize>) -> Result<Self, String> {
        policy.validate()?;
        Ok(Self { offset: 0, total, page_size: policy.page_size, cancelled: false })
    }

    pub fn next_page(&mut self) -> Option<(usize, usize)> {
        if self.cancelled || self.is_complete() {
            return None;
        }
        let start = self.offset;
        let end = self.total.map_or(start + self.page_size, |total| (start + self.page_size).min(total));
        self.offset = end;
        Some((start, end))
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn is_complete(&self) -> bool {
        self.total.is_some_and(|total| self.offset >= total)
    }
}

impl Default for ScanPolicy {
    fn default() -> Self {
        Self {
            max_items: 10_000,
            max_samples_per_type: 25,
            page_size: 250,
            allow_parallel: false,
        }
    }
}

impl ScanPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_items == 0 || self.max_samples_per_type == 0 || self.page_size == 0 {
            return Err("scan limits must be greater than zero".into());
        }
        if self.max_samples_per_type > self.max_items {
            return Err("sample limit cannot exceed total item limit".into());
        }
        Ok(())
    }

    pub fn page_bounds(&self, page: usize) -> Option<(usize, usize)> {
        if self.validate().is_err() {
            return None;
        }
        let start = page.checked_mul(self.page_size)?;
        if start >= self.max_items {
            return None;
        }
        Some((start, (start + self.page_size).min(self.max_items)))
    }
}

impl AdapterManifest {
    pub fn new(id: impl Into<String>, name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            contract_version: CONTRACT_VERSION.to_owned(),
            id: id.into(),
            name: name.into(),
            version: Version::new(0, 1, 0),
            source: source.into(),
            capabilities: Vec::new(),
            permissions: vec![Permission::ReadOnly],
            targets: Vec::new(),
            state: LifecycleState::Draft,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("adapter id must not be empty".into());
        }
        if self.name.trim().is_empty() {
            return Err("adapter name must not be empty".into());
        }
        if self.source.trim().is_empty() {
            return Err("adapter source must not be empty".into());
        }
        if self.capabilities.is_empty() {
            return Err("adapter must declare at least one capability".into());
        }
        if self.targets.is_empty() {
            return Err("adapter must declare at least one build target".into());
        }
        if self.permissions.is_empty() {
            return Err("adapter must declare an explicit permission profile".into());
        }
        Ok(())
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://embervault.local/schemas/adapter-manifest-0.1.json",
            "title": "EML Adapter Manifest",
            "type": "object",
            "required": ["contract_version", "id", "name", "version", "source", "capabilities", "permissions", "targets", "state"],
            "properties": {
                "contract_version": {"type": "string"},
                "id": {"type": "string", "minLength": 1},
                "name": {"type": "string", "minLength": 1},
                "version": {"type": "string"},
                "source": {"type": "string", "minLength": 1},
                "capabilities": {"type": "array", "items": {"type": "string"}, "minItems": 1},
                "permissions": {"type": "array", "minItems": 1},
                "targets": {"type": "array", "minItems": 1},
                "state": {"enum": ["draft", "packaged", "installed", "executed", "verified", "blocked", "unsupported"]}
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manifest_defaults_to_read_only_draft() {
        let manifest = AdapterManifest::new("resource-types", "Resource Types", "eml:kfc4");
        assert_eq!(manifest.state, LifecycleState::Draft);
        assert_eq!(manifest.permissions, vec![Permission::ReadOnly]);
    }

    #[test]
    fn validation_rejects_incomplete_manifest() {
        let manifest = AdapterManifest::new("resource-types", "Resource Types", "eml:kfc4");
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn manifest_round_trips_as_json() {
        let mut manifest = AdapterManifest::new("resource-types", "Resource Types", "eml:kfc4");
        manifest.capabilities.push("resource-inventory".into());
        manifest.targets.push(BuildTarget {
            game_build: "current".into(),
            runtime_version: "shroudforge-current".into(),
        });
        let json = manifest.to_pretty_json().expect("serialize manifest");
        let restored: AdapterManifest = serde_json::from_str(&json).expect("deserialize manifest");
        assert_eq!(manifest, restored);
        assert!(restored.validate().is_ok());
    }

    #[test]
    fn fingerprints_require_matching_runtime_context() {
        let base = BuildFingerprint {
            game_build: "game-build".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "current".into(),
            kfc_export_id: Some("export-a".into()),
        };
        let mut changed = base.clone();
        changed.game_build = "new-build".into();
        assert!(base.matches(&base));
        assert!(!base.matches(&changed));
    }

    #[test]
    fn fingerprint_hashes_a_file_deterministically() {
        let path = std::env::temp_dir().join(format!("eml-fingerprint-{}.bin", std::process::id()));
        fs::write(&path, b"fingerprint fixture").expect("write fixture");
        let first = BuildFingerprint::sha256_file(&path).expect("hash fixture");
        let second = BuildFingerprint::sha256_file(&path).expect("hash fixture again");
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn fingerprint_can_be_built_from_an_executable_path() {
        let path = std::env::temp_dir().join(format!("eml-fingerprint-build-{}.bin", std::process::id()));
        fs::write(&path, b"game executable fixture").expect("write fixture");
        let fingerprint = BuildFingerprint::from_executable(
            "game-build",
            &path,
            "ShroudForge",
            "runtime-build",
            Some("kfc-export".into()),
        )
        .expect("build fingerprint");
        assert_eq!(fingerprint.game_build, "game-build");
        assert!(fingerprint.game_executable_sha256.is_some());
        assert_eq!(fingerprint.kfc_export_id.as_deref(), Some("kfc-export"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn probe_manifest_rejects_write_capability() {
        let probe = ProbeManifest {
            contract_version: CONTRACT_VERSION.into(),
            id: "resource-types".into(),
            name: "Resource Types".into(),
            version: Version::new(1, 0, 0),
            source: "eml:kfc4".into(),
            read_only: false,
            target: BuildTarget {
                game_build: "current".into(),
                runtime_version: "ShroudForge-current".into(),
            },
            capabilities: vec!["resource-inventory".into()],
            evidence_state: EvidenceState::NotRun,
        };
        assert!(probe.validate().is_err());
    }

    #[test]
    fn catalog_round_trips_and_deduplicates() {
        let mut catalog = ResourceCatalog::new(None);
        let entry = CatalogEntry {
            id: "item-1".into(),
            qualified_type: "keen::ItemInfo".into(),
            fields: vec!["itemId".into()],
            source: "kfc4:test".into(),
        };
        catalog.add_entry(entry.clone());
        catalog.add_entry(entry);
        catalog.add_dependency("item-1", "icon-1");
        assert_eq!(catalog.entries.len(), 1);
        assert_eq!(catalog.dependencies.len(), 1);
        assert!(catalog.type_names().contains("keen::ItemInfo"));
        catalog.add_dependency("icon-1", "texture-1");
        assert!(catalog.dependencies_of("item-1").contains("icon-1"));
        assert!(catalog.dependents_of("icon-1").contains("item-1"));
        assert!(catalog.transitive_dependencies_of("item-1").contains("texture-1"));
    }

    #[test]
    fn catalog_persists_atomically() {
        let path = std::env::temp_dir().join(format!("eml-catalog-{}.json", std::process::id()));
        let mut catalog = ResourceCatalog::new(None);
        catalog.add_entry(CatalogEntry {
            id: "resource-1".into(),
            qualified_type: "test::Resource".into(),
            fields: vec![],
            source: "fixture".into(),
        });
        catalog.save_json(&path).expect("save catalog");
        let loaded = ResourceCatalog::load_json(&path).expect("load catalog");
        assert_eq!(catalog, loaded);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn catalog_diff_reports_resource_and_dependency_changes() {
        let mut before = ResourceCatalog::new(Some(BuildFingerprint {
            game_build: "one".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "one".into(),
            kfc_export_id: None,
        }));
        before.add_entry(CatalogEntry {
            id: "item-a".into(),
            qualified_type: "keen::Item".into(),
            fields: vec!["name".into()],
            source: "before".into(),
        });
        before.add_dependency("item-a", "icon-old");

        let mut after = ResourceCatalog::new(Some(BuildFingerprint {
            game_build: "two".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "two".into(),
            kfc_export_id: None,
        }));
        after.add_entry(CatalogEntry {
            id: "item-a".into(),
            qualified_type: "keen::Item".into(),
            fields: vec!["name".into(), "rarity".into()],
            source: "after".into(),
        });
        after.add_entry(CatalogEntry {
            id: "item-b".into(),
            qualified_type: "keen::Item".into(),
            fields: vec!["name".into()],
            source: "after".into(),
        });
        after.add_dependency("item-a", "icon-new");

        let diff = CatalogDiff::between(&before, &after);
        assert_eq!(diff.added, vec!["item-b"]);
        assert_eq!(diff.removed, Vec::<String>::new());
        assert_eq!(diff.changed, vec!["item-a"]);
        assert_eq!(diff.added_dependencies, vec![("item-a".into(), "icon-new".into())]);
        assert_eq!(diff.removed_dependencies, vec![("item-a".into(), "icon-old".into())]);
        assert!(!diff.is_empty());
    }

    #[test]
    fn catalog_store_persists_catalog_and_queryable_index() {
        let root = std::env::temp_dir().join(format!("eml-catalog-store-{}", std::process::id()));
        let mut catalog = ResourceCatalog::new(None);
        catalog.add_entry(CatalogEntry {
            id: "item-a".into(),
            qualified_type: "keen::Item".into(),
            fields: vec!["name".into()],
            source: "fixture".into(),
        });
        catalog.add_entry(CatalogEntry {
            id: "building-a".into(),
            qualified_type: "keen::Building".into(),
            fields: vec!["size".into()],
            source: "fixture".into(),
        });
        let store = CatalogStore::open(&root);
        store.write(&catalog).expect("write catalog store");
        let index = store.load_index().expect("load catalog index");
        assert_eq!(index.find("item-a").unwrap().fields, vec!["name"]);
        assert_eq!(index.by_type("keen::Building").len(), 1);
        assert_eq!(store.load_catalog().unwrap(), catalog);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn catalog_can_be_built_from_kfc_reflection_registry() {
        let registry = kfc::reflection::TypeRegistry::default();
        let catalog = ResourceCatalog::from_type_registry(&registry, "fixture", None);
        assert!(catalog.entries.is_empty());
        assert_eq!(catalog.schema_version, "1");
    }

    #[test]
    fn scan_policy_is_bounded_and_paginates() {
        let policy = ScanPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.page_bounds(0), Some((0, 250)));
        assert_eq!(policy.page_bounds(39), Some((9_750, 10_000)));
        assert_eq!(policy.page_bounds(40), None);
    }

    #[test]
    fn scan_cursor_supports_progress_resume_and_cancel() {
        let policy = ScanPolicy { page_size: 100, ..Default::default() };
        let mut cursor = ScanCursor::new(&policy, Some(225)).expect("create cursor");
        assert_eq!(cursor.next_page(), Some((0, 100)));
        assert_eq!(cursor.next_page(), Some((100, 200)));
        assert_eq!(cursor.next_page(), Some((200, 225)));
        assert!(cursor.is_complete());
        assert_eq!(cursor.next_page(), None);

        let mut cancelled = ScanCursor::new(&policy, None).expect("create open cursor");
        assert_eq!(cancelled.next_page(), Some((0, 100)));
        cancelled.cancel();
        assert_eq!(cancelled.next_page(), None);
    }

    #[test]
    fn adapter_schema_is_machine_readable() {
        let schema = AdapterManifest::json_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["required"].as_array().unwrap().iter().any(|v| v == "targets"));
    }

    #[test]
    fn evidence_becomes_stale_for_a_different_build() {
        let fingerprint = BuildFingerprint {
            game_build: "build-a".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "current".into(),
            kfc_export_id: Some("export-a".into()),
        };
        let mut evidence = ProbeEvidence {
            probe_id: "resource-types".into(),
            run_id: "run-1".into(),
            fingerprint: fingerprint.clone(),
            state: EvidenceState::Completed,
            observed_at: "2026-10-20T00:00:00Z".into(),
            summary: "read-only scan completed".into(),
            limitations: vec![],
        };
        assert!(evidence.is_current_for(&fingerprint));
        let mut changed = fingerprint;
        changed.game_build = "build-b".into();
        evidence.mark_stale_if_needed(&changed);
        assert_eq!(evidence.state, EvidenceState::Stale);
        assert!(!evidence.is_current_for(&changed));
    }

    #[test]
    fn package_generator_writes_only_a_staged_read_only_package() {
        let root = std::env::temp_dir().join(format!("eml-package-{}", std::process::id()));
        let spec = ShroudForgePackageSpec {
            id: "test-probe".into(),
            name: "Test Probe".into(),
            version: Version::new(1, 0, 0),
            description: "Generated test probe".into(),
            action_id: "runProbe".into(),
            action_label: "Run probe".into(),
        };
        let package = spec.write_read_only_probe(&root).expect("write package");
        assert!(package.join("mod.json").is_file());
        assert!(package.join("extended.mod.json").is_file());
        assert!(package.join("src/mod.lua").is_file());
        let lua = fs::read_to_string(package.join("src/mod.lua")).expect("read lua");
        assert!(lua.contains("read_only=true"));
        ShroudForgePackageSpec::validate_staged_package(&package).expect("validate package");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn probe_run_has_bounded_retries_and_terminal_states() {
        let mut run = ProbeRun::new("run-1", "resource-types", 2).expect("create run");
        run.start().expect("start run");
        assert_eq!(run.retry_backoff_ms(), 250);
        run.fail("temporary runtime error");
        assert!(run.can_retry());
        run.start().expect("retry run");
        run.complete().expect("complete run");
        assert!(!run.can_retry());
        assert_eq!(run.attempts, 2);
    }

    #[test]
    fn retry_policy_is_bounded_and_exponential() {
        let policy = RetryPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.backoff_ms(0), 250);
        assert_eq!(policy.backoff_ms(1), 500);
        assert_eq!(policy.backoff_ms(10), 5_000);
        assert!(RetryPolicy { max_attempts: 0, ..policy }.validate().is_err());
    }

    #[test]
    fn operation_gate_stays_closed_until_every_write_requirement_passes() {
        let mut gate = OperationGate::new(Permission::LiveConfirmed);
        assert!(!gate.ready_for_write());
        assert!(gate.missing_requirements().len() >= 6);
        gate.approved = true;
        gate.backup_verified = true;
        gate.build_matches = true;
        gate.confirmation_received = true;
        gate.readback_verified = true;
        gate.rollback_verified = true;
        assert!(gate.ready_for_write());
        assert!(gate.missing_requirements().is_empty());
    }

    #[test]
    fn guarded_operation_enforces_apply_readback_and_rollback_order() {
        let fingerprint = BuildFingerprint {
            game_build: "build-a".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "current".into(),
            kfc_export_id: None,
        };
        let mut gate = OperationGate::new(Permission::LiveConfirmed);
        gate.approved = true;
        gate.backup_verified = true;
        gate.build_matches = true;
        gate.confirmation_received = true;
        let mut operation = GuardedOperation::new("op-1", "adapter-1", fingerprint, gate);
        assert!(operation.preflight().is_ok());
        assert!(operation.mark_applied().is_ok());
        assert!(operation.verify_readback().is_ok());
        assert!(operation.verify_rollback().is_ok());
        assert_eq!(operation.state, OperationState::RolledBack);
        assert!(operation.record().message.contains("complete"));
    }

    #[test]
    fn replay_case_validates_imported_evidence_offline() {
        let fingerprint = BuildFingerprint {
            game_build: "build-a".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "current".into(),
            kfc_export_id: Some("export-a".into()),
        };
        let evidence = ProbeEvidence {
            probe_id: "resource-types".into(),
            run_id: "run-1".into(),
            fingerprint: fingerprint.clone(),
            state: EvidenceState::Completed,
            observed_at: "2026-10-20T00:00:00Z".into(),
            summary: "146 types".into(),
            limitations: vec![],
        };
        let case = ReplayCase {
            name: "resource type completion".into(),
            expected_probe_id: "resource-types".into(),
            expected_state: EvidenceState::Completed,
            evidence_json: evidence.to_json().expect("serialize evidence"),
        };
        assert!(case.run(&fingerprint).passed);
    }

    #[test]
    fn shroudforge_log_import_creates_completed_evidence() {
        let fingerprint = BuildFingerprint {
            game_build: "build-a".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "current".into(),
            kfc_export_id: None,
        };
        let log = "[info] resource-types | collection_status=completed | read_only=true";
        let evidence = ProbeEvidence::from_shroudforge_log(log, "resource-types", fingerprint)
            .expect("import completion marker");
        assert_eq!(evidence.state, EvidenceState::Completed);
        assert!(evidence.summary.contains("read_only=true"));
    }

    #[test]
    fn probe_schema_requires_read_only() {
        let schema = ProbeManifest::json_schema();
        assert_eq!(schema["properties"]["read_only"]["const"], true);
    }

    #[test]
    fn operation_journal_round_trips_append_only_records() {
        let path = std::env::temp_dir().join(format!("eml-journal-{}.jsonl", std::process::id()));
        let fingerprint = BuildFingerprint {
            game_build: "build-a".into(),
            game_executable_sha256: None,
            runtime_name: "ShroudForge".into(),
            runtime_version: "current".into(),
            kfc_export_id: None,
        };
        let record = OperationRecord {
            operation_id: "op-1".into(),
            adapter_id: "example-adapter".into(),
            state: OperationState::Blocked,
            fingerprint,
            message: "backup required".into(),
        };
        OperationJournal::append(&path, &record).expect("append record");
        OperationJournal::append(&path, &record).expect("append second record");
        let records = OperationJournal::read(&path).expect("read journal");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], record);
        let _ = fs::remove_file(path);
    }
}
