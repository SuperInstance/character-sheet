//! Bidirectional conversion between CharacterSheet and .nail bundle format.

use crate::models::*;
use std::io::{Read, Write};

/// The .nail bundle structure (as produced by pincher/lever-runner).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailBundle {
    pub manifest: NailManifest,
    pub identity: NailIdentity,
    pub config: NailConfig,
    pub reflexes: Vec<NailReflex>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailManifest {
    pub format_version: u32,
    pub name: String,
    pub class: String,
    pub level: u32,
    pub generation: u32,
    pub parent: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailIdentity {
    pub stats: NailStats,
    pub abilities: Vec<NailAbility>,
    pub biography: Vec<NailBiographyEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailStats {
    pub perception: u32,
    pub dexterity: u32,
    pub intelligence: u32,
    pub wisdom: u32,
    pub charisma: u32,
    pub constitution: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailAbility {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub trust: f64,
    pub level: u32,
    pub mastered: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailBiographyEntry {
    pub level: u32,
    pub event: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailConfig {
    pub model_provider: String,
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub sandbox_enabled: bool,
    pub sandbox_network: bool,
    pub sandbox_fs_write: bool,
    pub sandbox_exec: bool,
    pub trust_auto_approve: f64,
    pub trust_ask_user: f64,
    pub trust_deny: f64,
    pub skill_packs: Vec<NailSkillPack>,
    pub nail_imports: Vec<NailImportEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailSkillPack {
    pub name: String,
    pub version: String,
    pub loaded_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailImportEntry {
    pub source: String,
    pub imported_at: String,
    pub items_consumed: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NailReflex {
    pub id: String,
    pub pattern: String,
    pub action: String,
    pub priority: u32,
}

/// Bidirectional converter between CharacterSheet and .nail format.
pub struct NailConverter;

impl NailConverter {
    /// Convert a CharacterSheet into a NailBundle (lossless).
    pub fn to_nail(sheet: &CharacterSheet) -> NailBundle {
        let manifest = NailManifest {
            format_version: sheet.version,
            name: sheet.name.clone(),
            class: sheet.class.clone(),
            level: sheet.level,
            generation: sheet.generation,
            parent: sheet.parent.clone(),
            created_at: sheet
                .biography
                .first()
                .map(|e| e.timestamp.clone())
                .unwrap_or_default(),
        };

        let identity = NailIdentity {
            stats: NailStats {
                perception: sheet.stats.perception,
                dexterity: sheet.stats.dexterity,
                intelligence: sheet.stats.intelligence,
                wisdom: sheet.stats.wisdom,
                charisma: sheet.stats.charisma,
                constitution: sheet.stats.constitution,
            },
            abilities: sheet
                .abilities
                .iter()
                .map(|a| NailAbility {
                    name: a.name.clone(),
                    kind: match a.kind {
                        AbilityType::Innate => "innate",
                        AbilityType::Learned => "learned",
                        AbilityType::Granted => "granted",
                        AbilityType::Reflex => "reflex",
                    }
                    .into(),
                    trust: a.trust,
                    level: a.level,
                    mastered: a.mastered,
                })
                .collect(),
            biography: sheet
                .biography
                .iter()
                .map(|e| NailBiographyEntry {
                    level: e.level,
                    event: e.event.clone(),
                    timestamp: e.timestamp.clone(),
                })
                .collect(),
        };

        let config = NailConfig {
            model_provider: sheet.equipment.model_config.provider.clone(),
            model_name: sheet.equipment.model_config.model.clone(),
            temperature: sheet.equipment.model_config.temperature,
            max_tokens: sheet.equipment.model_config.max_tokens,
            sandbox_enabled: sheet.equipment.sandbox.enabled,
            sandbox_network: sheet.equipment.sandbox.network,
            sandbox_fs_write: sheet.equipment.sandbox.fs_write,
            sandbox_exec: sheet.equipment.sandbox.exec,
            trust_auto_approve: sheet.equipment.trust_thresholds.auto_approve,
            trust_ask_user: sheet.equipment.trust_thresholds.ask_user,
            trust_deny: sheet.equipment.trust_thresholds.deny,
            skill_packs: sheet
                .inventory
                .skill_packs
                .iter()
                .map(|sp| NailSkillPack {
                    name: sp.name.clone(),
                    version: sp.version.clone(),
                    loaded_at: sp.loaded_at.clone(),
                })
                .collect(),
            nail_imports: sheet
                .inventory
                .nail_imports
                .iter()
                .map(|ni| NailImportEntry {
                    source: ni.source.clone(),
                    imported_at: ni.imported_at.clone(),
                    items_consumed: ni.items_consumed,
                })
                .collect(),
        };

        NailBundle {
            manifest,
            identity,
            config,
            reflexes: Vec::new(),
        }
    }

    /// Parse a NailBundle back into a CharacterSheet (lossless).
    pub fn from_nail(bundle: &NailBundle) -> CharacterSheet {
        CharacterSheet {
            version: bundle.manifest.format_version,
            name: bundle.manifest.name.clone(),
            level: bundle.manifest.level,
            class: bundle.manifest.class.clone(),
            generation: bundle.manifest.generation,
            parent: bundle.manifest.parent.clone(),
            stats: Stats {
                perception: bundle.identity.stats.perception,
                dexterity: bundle.identity.stats.dexterity,
                intelligence: bundle.identity.stats.intelligence,
                wisdom: bundle.identity.stats.wisdom,
                charisma: bundle.identity.stats.charisma,
                constitution: bundle.identity.stats.constitution,
            },
            abilities: bundle
                .identity
                .abilities
                .iter()
                .map(|a| Ability {
                    name: a.name.clone(),
                    kind: match a.kind.as_str() {
                        "innate" => AbilityType::Innate,
                        "learned" => AbilityType::Learned,
                        "granted" => AbilityType::Granted,
                        "reflex" => AbilityType::Reflex,
                        _ => AbilityType::Learned,
                    },
                    trust: a.trust,
                    level: a.level,
                    mastered: a.mastered,
                })
                .collect(),
            equipment: Equipment {
                model_config: ModelConfig {
                    provider: bundle.config.model_provider.clone(),
                    model: bundle.config.model_name.clone(),
                    temperature: bundle.config.temperature,
                    max_tokens: bundle.config.max_tokens,
                },
                sandbox: SandboxSettings {
                    enabled: bundle.config.sandbox_enabled,
                    network: bundle.config.sandbox_network,
                    fs_write: bundle.config.sandbox_fs_write,
                    exec: bundle.config.sandbox_exec,
                },
                trust_thresholds: TrustThresholds {
                    auto_approve: bundle.config.trust_auto_approve,
                    ask_user: bundle.config.trust_ask_user,
                    deny: bundle.config.trust_deny,
                },
            },
            inventory: Inventory {
                skill_packs: bundle
                    .config
                    .skill_packs
                    .iter()
                    .map(|sp| SkillPack {
                        name: sp.name.clone(),
                        version: sp.version.clone(),
                        loaded_at: sp.loaded_at.clone(),
                    })
                    .collect(),
                nail_imports: bundle
                    .config
                    .nail_imports
                    .iter()
                    .map(|ni| NailImport {
                        source: ni.source.clone(),
                        imported_at: ni.imported_at.clone(),
                        items_consumed: ni.items_consumed,
                    })
                    .collect(),
            },
            biography: bundle
                .identity
                .biography
                .iter()
                .map(|e| BiographyEntry {
                    level: e.level,
                    event: e.event.clone(),
                    timestamp: e.timestamp.clone(),
                })
                .collect(),
        }
    }

    /// Serialize a NailBundle to a tar.zst archive (the on-disk .nail format).
    pub fn to_tar_zst(bundle: &NailBundle) -> Result<Vec<u8>, NailError> {
        let manifest_json = serde_json::to_vec_pretty(&bundle.manifest)?;
        let identity_json = serde_json::to_vec_pretty(&bundle.identity)?;
        let config_toml = toml::to_string_pretty(&bundle.config)?;
        let reflexes_json = serde_json::to_vec_pretty(&bundle.reflexes)?;

        // Build tar archive
        let mut tar_buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_buf);
            append_file(&mut builder, "manifest.json", &manifest_json)?;
            append_file(&mut builder, "identity.json", &identity_json)?;
            append_file(&mut builder, "config.toml", config_toml.as_bytes())?;
            append_file(&mut builder, "reflexes.json", &reflexes_json)?;
            builder.finish()?;
        }

        // Compress with zstd
        let mut zst_buf = Vec::new();
        let mut encoder = zstd::Encoder::new(&mut zst_buf, 3)?;
        encoder.write_all(&tar_buf)?;
        encoder.finish()?;
        Ok(zst_buf)
    }

    /// Deserialize a tar.zst archive back into a NailBundle.
    pub fn from_tar_zst(data: &[u8]) -> Result<NailBundle, NailError> {
        let mut decoder = zstd::Decoder::new(data)?;
        let mut tar_buf = Vec::new();
        decoder.read_to_end(&mut tar_buf)?;

        let mut archive = tar::Archive::new(tar_buf.as_slice());
        let mut manifest: Option<NailManifest> = None;
        let mut identity: Option<NailIdentity> = None;
        let mut config: Option<NailConfig> = None;
        let mut reflexes: Vec<NailReflex> = Vec::new();

        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?.to_path_buf();

            match path.to_str() {
                Some("manifest.json") => {
                    let mut buf = Vec::new();
                    entry.read_to_end(&mut buf)?;
                    manifest = Some(serde_json::from_slice(&buf)?);
                }
                Some("identity.json") => {
                    let mut buf = Vec::new();
                    entry.read_to_end(&mut buf)?;
                    identity = Some(serde_json::from_slice(&buf)?);
                }
                Some("config.toml") => {
                    let mut buf = String::new();
                    entry.read_to_string(&mut buf)?;
                    config = Some(toml::from_str(&buf)?);
                }
                Some("reflexes.json") => {
                    let mut buf = Vec::new();
                    entry.read_to_end(&mut buf)?;
                    reflexes = serde_json::from_slice(&buf)?;
                }
                _ => {} // skip unknown files
            }
        }

        Ok(NailBundle {
            manifest: manifest.ok_or(NailError::MissingFile("manifest.json"))?,
            identity: identity.ok_or(NailError::MissingFile("identity.json"))?,
            config: config.ok_or(NailError::MissingFile("config.toml"))?,
            reflexes,
        })
    }
}

fn append_file<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    name: &str,
    data: &[u8],
) -> Result<(), NailError> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append_data(&mut header, name, data)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum NailError {
    #[error("Missing required file in nail bundle: {0}")]
    MissingFile(&'static str),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML deserialize error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSe(#[from] toml::ser::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
