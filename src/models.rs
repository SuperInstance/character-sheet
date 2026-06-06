use serde::{Deserialize, Serialize};

/// Current character sheet format version.
pub const SHEET_VERSION: u32 = 3;

/// The six core stats for an agent character.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stats {
    pub perception: u32,
    pub dexterity: u32,
    pub intelligence: u32,
    pub wisdom: u32,
    pub charisma: u32,
    pub constitution: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            perception: 10,
            dexterity: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            constitution: 10,
        }
    }
}

impl Stats {
    pub fn total(&self) -> u32 {
        self.perception + self.dexterity + self.intelligence
            + self.wisdom + self.charisma + self.constitution
    }
}

/// An ability the character has learned or mastered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ability {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: AbilityType,
    /// 0.0 – 1.0 trust level for this ability.
    pub trust: f64,
    pub level: u32,
    pub mastered: bool,
}

/// Category of ability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityType {
    Innate,
    Learned,
    Granted,
    Reflex,
}

/// Equipment — model config, sandbox settings, trust thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Equipment {
    pub model_config: ModelConfig,
    pub sandbox: SandboxSettings,
    pub trust_thresholds: TrustThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            model: "gpt-4".into(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SandboxSettings {
    pub enabled: bool,
    pub network: bool,
    pub fs_write: bool,
    pub exec: bool,
}

impl Default for SandboxSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            network: false,
            fs_write: false,
            exec: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrustThresholds {
    pub auto_approve: f64,
    pub ask_user: f64,
    pub deny: f64,
}

impl Default for TrustThresholds {
    fn default() -> Self {
        Self {
            auto_approve: 0.9,
            ask_user: 0.5,
            deny: 0.0,
        }
    }
}

/// Inventory — skill packs loaded and .nail imports consumed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Inventory {
    pub skill_packs: Vec<SkillPack>,
    pub nail_imports: Vec<NailImport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillPack {
    pub name: String,
    pub version: String,
    pub loaded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NailImport {
    pub source: String,
    pub imported_at: String,
    pub items_consumed: u32,
}

/// A biography entry — auto-generated from play history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BiographyEntry {
    pub level: u32,
    pub event: String,
    pub timestamp: String,
}

/// The full character sheet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterSheet {
    pub version: u32,
    pub name: String,
    pub level: u32,
    pub class: String,
    pub generation: u32,
    pub parent: Option<String>,
    pub stats: Stats,
    pub abilities: Vec<Ability>,
    pub equipment: Equipment,
    pub inventory: Inventory,
    pub biography: Vec<BiographyEntry>,
}

impl CharacterSheet {
    /// Create a fresh level-1 character.
    pub fn new(name: impl Into<String>, class: impl Into<String>) -> Self {
        Self {
            version: SHEET_VERSION,
            name: name.into(),
            level: 1,
            class: class.into(),
            generation: 1,
            parent: None,
            stats: Stats::default(),
            abilities: Vec::new(),
            equipment: Equipment::default(),
            inventory: Inventory::default(),
            biography: vec![BiographyEntry {
                level: 1,
                event: "Character created".into(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }],
        }
    }

    /// Add an ability to the roster.
    pub fn add_ability(&mut self, ability: Ability) {
        if let Some(existing) = self.abilities.iter_mut().find(|a| a.name == ability.name) {
            *existing = ability;
        } else {
            self.abilities.push(ability);
        }
    }

    /// Level up the character.
    pub fn level_up(&mut self, reason: impl Into<String>) {
        self.level += 1;
        self.biography.push(BiographyEntry {
            level: self.level,
            event: reason.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    /// Generate a human-readable biography string.
    pub fn biography_text(&self) -> String {
        self.biography
            .iter()
            .map(|e| format!("Level {}: {} ({})", e.level, e.event, e.timestamp))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Add a skill pack to inventory.
    pub fn load_skill_pack(&mut self, name: impl Into<String>, version: impl Into<String>) {
        self.inventory.skill_packs.push(SkillPack {
            name: name.into(),
            version: version.into(),
            loaded_at: chrono::Utc::now().to_rfc3339(),
        });
    }
}
