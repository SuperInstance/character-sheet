//! Import CharacterSheet from various sources.

use crate::models::*;
use crate::nail::NailConverter;

pub struct CharacterImporter;

impl CharacterImporter {
    /// Import from a .nail tar.zst archive.
    pub fn from_nail_bytes(data: &[u8]) -> Result<CharacterSheet, ImportError> {
        let bundle = NailConverter::from_tar_zst(data)?;
        Ok(NailConverter::from_nail(&bundle))
    }

    /// Import from a .sheet.json string.
    pub fn from_json(json: &str) -> Result<CharacterSheet, ImportError> {
        serde_json::from_str(json).map_err(ImportError::from)
    }

    /// Import from another CharacterSheet (for learning/absorption).
    pub fn absorb(target: &mut CharacterSheet, source: &CharacterSheet) {
        // Absorb abilities the target doesn't have
        for ability in &source.abilities {
            target.add_ability(ability.clone());
        }

        // Absorb skill packs
        for sp in &source.inventory.skill_packs {
            let exists = target.inventory.skill_packs.iter().any(|t| t.name == sp.name);
            if !exists {
                target.inventory.skill_packs.push(sp.clone());
            }
        }

        // Record the absorption in biography
        target.biography.push(BiographyEntry {
            level: target.level,
            event: format!("Absorbed knowledge from {}", source.name),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        // Record nail import
        target.inventory.nail_imports.push(NailImport {
            source: source.name.clone(),
            imported_at: chrono::Utc::now().to_rfc3339(),
            items_consumed: source.abilities.len() as u32,
        });
    }

    /// Bootstrap from a raw reflex database (simplified).
    pub fn from_raw_reflexes(name: &str, class: &str, reflexes: Vec<(String, String, u32)>) -> CharacterSheet {
        let mut sheet = CharacterSheet::new(name, class);

        for (pattern, _action, priority) in reflexes {
            sheet.add_ability(Ability {
                name: format!("reflex:{}", pattern),
                kind: AbilityType::Reflex,
                trust: 0.5,
                level: priority,
                mastered: false,
            });
        }

        sheet.biography.push(BiographyEntry {
            level: 1,
            event: "Bootstrapped from raw reflex database".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        sheet
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Nail error: {0}")]
    Nail(#[from] crate::nail::NailError),
}
