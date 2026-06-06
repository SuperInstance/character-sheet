use character_sheet::*;
use character_sheet::nail::NailConverter;

fn make_test_sheet() -> CharacterSheet {
    let mut sheet = CharacterSheet::new("TestBot", "Scout");
    sheet.generation = 3;
    sheet.parent = Some("ParentBot".into());
    sheet.stats.perception = 15;
    sheet.stats.intelligence = 18;
    sheet.add_ability(Ability {
        name: "git-push".into(),
        kind: AbilityType::Learned,
        trust: 0.95,
        level: 3,
        mastered: true,
    });
    sheet.add_ability(Ability {
        name: "reflex:file-watch".into(),
        kind: AbilityType::Reflex,
        trust: 0.8,
        level: 2,
        mastered: false,
    });
    sheet.level_up("Mastered git-push at level 3");
    sheet.load_skill_pack("github", "1.2.0");
    sheet
}

#[test]
fn test_character_creation() {
    let sheet = CharacterSheet::new("Alpha", "Mage");
    assert_eq!(sheet.name, "Alpha");
    assert_eq!(sheet.class, "Mage");
    assert_eq!(sheet.level, 1);
    assert_eq!(sheet.generation, 1);
    assert!(sheet.parent.is_none());
    assert_eq!(sheet.stats.total(), 60); // 6 * 10
    assert!(sheet.abilities.is_empty());
}

#[test]
fn test_stat_serialization() {
    let stats = Stats {
        perception: 14,
        dexterity: 12,
        intelligence: 20,
        wisdom: 16,
        charisma: 8,
        constitution: 10,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let back: Stats = serde_json::from_str(&json).unwrap();
    assert_eq!(stats, back);
}

#[test]
fn test_ability_roster_serialization() {
    let abilities = vec![
        Ability {
            name: "fireball".into(),
            kind: AbilityType::Learned,
            trust: 0.9,
            level: 5,
            mastered: true,
        },
        Ability {
            name: "heal".into(),
            kind: AbilityType::Granted,
            trust: 0.7,
            level: 3,
            mastered: false,
        },
    ];
    let json = serde_json::to_string(&abilities).unwrap();
    let back: Vec<Ability> = serde_json::from_str(&json).unwrap();
    assert_eq!(abilities, back);
}

#[test]
fn test_add_ability_updates_existing() {
    let mut sheet = CharacterSheet::new("Bot", "Cleric");
    sheet.add_ability(Ability {
        name: "heal".into(),
        kind: AbilityType::Learned,
        trust: 0.5,
        level: 1,
        mastered: false,
    });
    assert_eq!(sheet.abilities.len(), 1);

    sheet.add_ability(Ability {
        name: "heal".into(),
        kind: AbilityType::Learned,
        trust: 0.9,
        level: 3,
        mastered: true,
    });
    assert_eq!(sheet.abilities.len(), 1);
    assert_eq!(sheet.abilities[0].level, 3);
    assert!(sheet.abilities[0].mastered);
}

#[test]
fn test_level_up() {
    let mut sheet = CharacterSheet::new("Bot", "Rogue");
    sheet.level_up("Completed first quest");
    assert_eq!(sheet.level, 2);
    assert_eq!(sheet.biography.len(), 2);
    assert_eq!(sheet.biography[1].event, "Completed first quest");
}

#[test]
fn test_biography_generation() {
    let sheet = make_test_sheet();
    let text = sheet.biography_text();
    assert!(text.contains("Level 1: Character created"));
    assert!(text.contains("Level 2: Mastered git-push"));
}

#[test]
fn test_round_trip_nail_conversion() {
    let sheet = make_test_sheet();
    let bundle = NailConverter::to_nail(&sheet);
    let back = NailConverter::from_nail(&bundle);

    assert_eq!(back.name, sheet.name);
    assert_eq!(back.level, sheet.level);
    assert_eq!(back.class, sheet.class);
    assert_eq!(back.generation, sheet.generation);
    assert_eq!(back.parent, sheet.parent);
    assert_eq!(back.stats, sheet.stats);
    assert_eq!(back.abilities, sheet.abilities);
    assert_eq!(back.equipment, sheet.equipment);
    assert_eq!(back.inventory.skill_packs, sheet.inventory.skill_packs);
    assert_eq!(back.biography.len(), sheet.biography.len());
}

#[test]
fn test_round_trip_tar_zst() {
    let sheet = make_test_sheet();
    let bundle = NailConverter::to_nail(&sheet);
    let bytes = NailConverter::to_tar_zst(&bundle).unwrap();
    let back_bundle = NailConverter::from_tar_zst(&bytes).unwrap();

    assert_eq!(back_bundle.manifest, bundle.manifest);
    assert_eq!(back_bundle.identity, bundle.identity);
    assert_eq!(back_bundle.config, bundle.config);
    assert_eq!(back_bundle.reflexes, bundle.reflexes);
}

#[test]
fn test_export_nail() {
    let sheet = make_test_sheet();
    let bytes = CharacterExporter::to_nail(&sheet).unwrap();
    assert!(!bytes.is_empty());

    // Verify it round-trips through import
    let imported = CharacterImporter::from_nail_bytes(&bytes).unwrap();
    assert_eq!(imported.name, sheet.name);
    assert_eq!(imported.stats, sheet.stats);
}

#[test]
fn test_export_json() {
    let sheet = make_test_sheet();
    let json = CharacterExporter::to_json(&sheet).unwrap();
    assert!(json.contains("\"name\": \"TestBot\""));
    assert!(json.contains("\"class\": \"Scout\""));

    let back: CharacterSheet = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sheet);
}

#[test]
fn test_export_markdown() {
    let sheet = make_test_sheet();
    let md = CharacterExporter::to_markdown(&sheet);
    assert!(md.contains("# TestBot"));
    assert!(md.contains("Level 2 Scout"));
    assert!(md.contains("git-push"));
    assert!(md.contains("Perception"));
    assert!(md.contains("Biography"));
}

#[test]
fn test_export_dna() {
    let sheet = make_test_sheet();
    let dna = CharacterExporter::to_dna(&sheet);
    assert!(!dna.is_empty());
    // First 4 bytes should be version
    let version = u32::from_le_bytes(dna[0..4].try_into().unwrap());
    assert_eq!(version, crate::models::SHEET_VERSION);
}

#[test]
fn test_import_from_json() {
    let sheet = make_test_sheet();
    let json = CharacterExporter::to_json(&sheet).unwrap();
    let imported = CharacterImporter::from_json(&json).unwrap();
    assert_eq!(imported, sheet);
}

#[test]
fn test_import_absorb() {
    let mut target = CharacterSheet::new("Alpha", "Scout");
    let source = make_test_sheet();

    CharacterImporter::absorb(&mut target, &source);

    // Should have gained abilities from source
    assert!(target.abilities.iter().any(|a| a.name == "git-push"));
    assert!(target.abilities.iter().any(|a| a.name == "reflex:file-watch"));

    // Should have the skill pack
    assert!(target.inventory.skill_packs.iter().any(|sp| sp.name == "github"));

    // Should record the absorption
    assert!(target.biography.iter().any(|e| e.event.contains("Absorbed")));
    assert_eq!(target.inventory.nail_imports.len(), 1);
}

#[test]
fn test_import_from_raw_reflexes() {
    let reflexes = vec![
        ("file-change".into(), "commit".into(), 3),
        ("merge-conflict".into(), "resolve".into(), 5),
    ];
    let sheet = CharacterImporter::from_raw_reflexes("ReflexBot", "Warrior", reflexes);

    assert_eq!(sheet.name, "ReflexBot");
    assert_eq!(sheet.class, "Warrior");
    assert_eq!(sheet.abilities.len(), 2);
    assert!(sheet.abilities[0].name.contains("file-change"));
    assert_eq!(sheet.abilities[0].kind, AbilityType::Reflex);
}

#[test]
fn test_migration_v1_to_current() {
    let v1 = migration::CharacterSheetV1 {
        name: "OldBot".into(),
        level: 5,
        class: "Fighter".into(),
        stats: migration::StatsV1 {
            perception: 12,
            dexterity: 14,
            intelligence: 16,
        },
        abilities: vec![migration::AbilityV1 {
            name: "slash".into(),
            trust: 0.8,
            mastered: true,
        }],
    };

    let current = VersionMigration::v1_to_current(&v1);
    assert_eq!(current.name, "OldBot");
    assert_eq!(current.level, 5);
    assert_eq!(current.stats.perception, 12);
    assert_eq!(current.stats.wisdom, 10); // default
    assert_eq!(current.abilities.len(), 1);
    assert_eq!(current.abilities[0].kind, AbilityType::Innate);
    assert!(current.biography.iter().any(|e| e.event.contains("v1")));
}

#[test]
fn test_migration_v2_to_current() {
    let v2 = migration::CharacterSheetV2 {
        name: "MidBot".into(),
        level: 10,
        class: "Rogue".into(),
        generation: 2,
        parent: Some("OldBot".into()),
        stats: Stats::default(),
        abilities: vec![migration::AbilityV2 {
            name: "sneak".into(),
            trust: 0.7,
            level: 4,
            mastered: false,
        }],
        biography: vec![BiographyEntry {
            level: 1,
            event: "Created".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
        }],
    };

    let current = VersionMigration::v2_to_current(&v2);
    assert_eq!(current.name, "MidBot");
    assert_eq!(current.level, 10);
    assert_eq!(current.generation, 2);
    assert_eq!(current.abilities[0].kind, AbilityType::Learned);
    assert!(current.biography.iter().any(|e| e.event.contains("v2")));
}

#[test]
fn test_migration_auto_detect() {
    // v1 (no version field)
    let v1_json = r#"{"name":"V1Bot","level":3,"class":"Cleric","stats":{"perception":10,"dexterity":10,"intelligence":10},"abilities":[]}"#;
    let migrated = VersionMigration::migrate(v1_json).unwrap();
    assert_eq!(migrated.name, "V1Bot");

    // v2
    let v2_json = r#"{"version":2,"name":"V2Bot","level":5,"class":"Mage","generation":1,"parent":null,"stats":{"perception":10,"dexterity":10,"intelligence":10,"wisdom":10,"charisma":10,"constitution":10},"abilities":[],"biography":[]}"#;
    let migrated = VersionMigration::migrate(v2_json).unwrap();
    assert_eq!(migrated.name, "V2Bot");

    // v3 (current)
    let v3_json = r#"{"version":3,"name":"V3Bot","level":7,"class":"Scout","generation":1,"parent":null,"stats":{"perception":10,"dexterity":10,"intelligence":10,"wisdom":10,"charisma":10,"constitution":10},"abilities":[],"equipment":{"model_config":{"provider":"openai","model":"gpt-4","temperature":0.7,"max_tokens":4096},"sandbox":{"enabled":true,"network":false,"fs_write":false,"exec":false},"trust_thresholds":{"auto_approve":0.9,"ask_user":0.5,"deny":0.0}},"inventory":{"skill_packs":[],"nail_imports":[]},"biography":[]}"#;
    let migrated = VersionMigration::migrate(v3_json).unwrap();
    assert_eq!(migrated.name, "V3Bot");
}

#[test]
fn test_equipment_management() {
    let mut sheet = CharacterSheet::new("Bot", "Artificer");

    // Customize equipment
    sheet.equipment.model_config.provider = "anthropic".into();
    sheet.equipment.model_config.model = "claude-3".into();
    sheet.equipment.model_config.temperature = 0.5;
    sheet.equipment.sandbox.network = true;
    sheet.equipment.trust_thresholds.auto_approve = 0.95;

    let json = serde_json::to_string(&sheet).unwrap();
    let back: CharacterSheet = serde_json::from_str(&json).unwrap();

    assert_eq!(back.equipment.model_config.provider, "anthropic");
    assert_eq!(back.equipment.model_config.model, "claude-3");
    assert_eq!(back.equipment.model_config.temperature, 0.5);
    assert!(back.equipment.sandbox.network);
    assert_eq!(back.equipment.trust_thresholds.auto_approve, 0.95);
}
