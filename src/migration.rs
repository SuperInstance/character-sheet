//! Version migration for character sheet format evolution.

use crate::models::*;

/// Historical v1 sheet format (for migration testing).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CharacterSheetV1 {
    pub name: String,
    pub level: u32,
    pub class: String,
    pub stats: StatsV1,
    pub abilities: Vec<AbilityV1>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct StatsV1 {
    pub perception: u32,
    pub dexterity: u32,
    pub intelligence: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AbilityV1 {
    pub name: String,
    pub trust: f64,
    pub mastered: bool,
}

/// Historical v2 sheet format.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CharacterSheetV2 {
    pub name: String,
    pub level: u32,
    pub class: String,
    pub generation: u32,
    pub parent: Option<String>,
    pub stats: Stats,
    pub abilities: Vec<AbilityV2>,
    pub biography: Vec<BiographyEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AbilityV2 {
    pub name: String,
    pub trust: f64,
    pub level: u32,
    pub mastered: bool,
}

pub struct VersionMigration;

impl VersionMigration {
    /// Migrate a v1 sheet to the current format.
    pub fn v1_to_current(v1: &CharacterSheetV1) -> CharacterSheet {
        let mut sheet = CharacterSheet::new(&v1.name, &v1.class);
        sheet.level = v1.level;

        // v1 only had 3 stats; assign defaults for the new ones
        sheet.stats.perception = v1.stats.perception;
        sheet.stats.dexterity = v1.stats.dexterity;
        sheet.stats.intelligence = v1.stats.intelligence;
        // wisdom, charisma, constitution get defaults (10)

        // Migrate abilities (v1 had no type or level)
        for a in &v1.abilities {
            sheet.add_ability(Ability {
                name: a.name.clone(),
                kind: AbilityType::Innate, // v1 didn't track type
                trust: a.trust,
                level: 1, // unknown in v1
                mastered: a.mastered,
            });
        }

        sheet.biography.push(BiographyEntry {
            level: v1.level,
            event: "Migrated from v1 format".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        sheet
    }

    /// Migrate a v2 sheet to the current format.
    pub fn v2_to_current(v2: &CharacterSheetV2) -> CharacterSheet {
        let mut sheet = CharacterSheet::new(&v2.name, &v2.class);
        sheet.level = v2.level;
        sheet.generation = v2.generation;
        sheet.parent = v2.parent.clone();
        sheet.stats = v2.stats.clone();
        sheet.biography = v2.biography.clone();

        // v2 abilities lacked type
        for a in &v2.abilities {
            sheet.add_ability(Ability {
                name: a.name.clone(),
                kind: AbilityType::Learned,
                trust: a.trust,
                level: a.level,
                mastered: a.mastered,
            });
        }

        // v2 didn't have equipment or inventory — defaults are fine

        sheet.biography.push(BiographyEntry {
            level: v2.level,
            event: "Migrated from v2 format".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        sheet
    }

    /// Auto-detect version and migrate to current.
    pub fn migrate(json: &str) -> Result<CharacterSheet, MigrationError> {
        let value: serde_json::Value = serde_json::from_str(json)?;

        // Detect version by looking at the structure
        if let Some(version) = value.get("version").and_then(|v| v.as_u64()) {
            match version {
                3 => serde_json::from_str(json).map_err(MigrationError::from),
                2 => {
                    let v2: CharacterSheetV2 = serde_json::from_str(json)?;
                    Ok(Self::v2_to_current(&v2))
                }
                _ => Err(MigrationError::UnknownVersion(version as u32)),
            }
        } else {
            // No version field — assume v1
            let v1: CharacterSheetV1 = serde_json::from_str(json)?;
            Ok(Self::v1_to_current(&v1))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Unknown sheet version: {0}")]
    UnknownVersion(u32),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
