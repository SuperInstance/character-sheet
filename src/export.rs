//! Export CharacterSheet to multiple formats.

use crate::models::*;
use crate::nail::NailConverter;

pub struct CharacterExporter;

impl CharacterExporter {
    /// Export as .nail (tar.zst archive).
    pub fn to_nail(sheet: &CharacterSheet) -> Result<Vec<u8>, ExportError> {
        let bundle = NailConverter::to_nail(sheet);
        NailConverter::to_tar_zst(&bundle).map_err(ExportError::from)
    }

    /// Export as .sheet.json (human-readable).
    pub fn to_json(sheet: &CharacterSheet) -> Result<String, ExportError> {
        serde_json::to_string_pretty(sheet).map_err(ExportError::from)
    }

    /// Export as .sheet.md (markdown character card).
    pub fn to_markdown(sheet: &CharacterSheet) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {} — Level {} {}\n\n", sheet.name, sheet.level, sheet.class));

        md.push_str(&format!("**Generation:** {}  \n", sheet.generation));
        if let Some(ref parent) = sheet.parent {
            md.push_str(&format!("**Parent:** {}  \n", parent));
        }
        md.push_str(&format!("**Format Version:** v{}  \n\n", sheet.version));

        md.push_str("## Stats\n\n");
        md.push_str(&format!(
            "| Stat | Value |\n|------|-------|\n| Perception | {} |\n| Dexterity | {} |\n| Intelligence | {} |\n| Wisdom | {} |\n| Charisma | {} |\n| Constitution | {} |\n| **Total** | **{}** |\n\n",
            sheet.stats.perception,
            sheet.stats.dexterity,
            sheet.stats.intelligence,
            sheet.stats.wisdom,
            sheet.stats.charisma,
            sheet.stats.constitution,
            sheet.stats.total(),
        ));

        if !sheet.abilities.is_empty() {
            md.push_str("## Abilities\n\n");
            for a in &sheet.abilities {
                let mastered = if a.mastered { " ★" } else { "" };
                md.push_str(&format!(
                    "- **{}** ({:?}) — Trust: {:.0}%, Level {}{}\n",
                    a.name, a.kind, a.trust * 100.0, a.level, mastered
                ));
            }
            md.push('\n');
        }

        md.push_str(&format!(
            "## Equipment\n\n- Model: {}/{} (temp: {}, max_tokens: {})\n- Sandbox: {} (net: {}, fs: {}, exec: {})\n- Trust thresholds: auto={:.0}% ask={:.0}% deny={:.0}%\n\n",
            sheet.equipment.model_config.provider,
            sheet.equipment.model_config.model,
            sheet.equipment.model_config.temperature,
            sheet.equipment.model_config.max_tokens,
            if sheet.equipment.sandbox.enabled { "ON" } else { "OFF" },
            sheet.equipment.sandbox.network,
            sheet.equipment.sandbox.fs_write,
            sheet.equipment.sandbox.exec,
            sheet.equipment.trust_thresholds.auto_approve * 100.0,
            sheet.equipment.trust_thresholds.ask_user * 100.0,
            sheet.equipment.trust_thresholds.deny * 100.0,
        ));

        if !sheet.inventory.skill_packs.is_empty() {
            md.push_str("## Inventory — Skill Packs\n\n");
            for sp in &sheet.inventory.skill_packs {
                md.push_str(&format!("- {} v{} (loaded {})\n", sp.name, sp.version, sp.loaded_at));
            }
            md.push('\n');
        }

        if !sheet.inventory.nail_imports.is_empty() {
            md.push_str("## Inventory — .nail Imports\n\n");
            for ni in &sheet.inventory.nail_imports {
                md.push_str(&format!("- {} ({} items, imported {})\n", ni.source, ni.items_consumed, ni.imported_at));
            }
            md.push('\n');
        }

        if !sheet.biography.is_empty() {
            md.push_str("## Biography\n\n");
            for entry in &sheet.biography {
                md.push_str(&format!("- **Level {}:** {} _({})_\n", entry.level, entry.event, entry.timestamp));
            }
        }

        md
    }

    /// Export as .dna (minimal — embedding vectors and class only).
    pub fn to_dna(sheet: &CharacterSheet) -> Vec<u8> {
        // Compact binary format:
        // [version: u32 LE][class_len: u16 LE][class_bytes][stat_perception: u32 LE]...[stat_constitution: u32 LE]
        // [ability_count: u16 LE][for each: name_len u16 LE, name_bytes, trust f64 LE, level u32 LE, mastered u8]
        let mut buf = Vec::new();
        buf.extend_from_slice(&sheet.version.to_le_bytes());

        let class_bytes = sheet.class.as_bytes();
        buf.extend_from_slice(&(class_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(class_bytes);

        // Stats as 6x u32 LE
        buf.extend_from_slice(&sheet.stats.perception.to_le_bytes());
        buf.extend_from_slice(&sheet.stats.dexterity.to_le_bytes());
        buf.extend_from_slice(&sheet.stats.intelligence.to_le_bytes());
        buf.extend_from_slice(&sheet.stats.wisdom.to_le_bytes());
        buf.extend_from_slice(&sheet.stats.charisma.to_le_bytes());
        buf.extend_from_slice(&sheet.stats.constitution.to_le_bytes());

        // Abilities count
        buf.extend_from_slice(&(sheet.abilities.len() as u16).to_le_bytes());
        for a in &sheet.abilities {
            let name_bytes = a.name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(name_bytes);
            buf.extend_from_slice(&a.trust.to_le_bytes());
            buf.extend_from_slice(&a.level.to_le_bytes());
            buf.push(if a.mastered { 1 } else { 0 });
        }

        buf
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Nail error: {0}")]
    Nail(#[from] crate::nail::NailError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
