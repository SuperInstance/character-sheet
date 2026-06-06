# character-sheet

The `.nail` bundle format for AI agents — a character sheet that travels with the agent, tracks growth, and persists across sessions.

## Why This Exists

Agents today are stateless. Every session starts from scratch. That's not how characters work — a D&D character carries their history, stats, equipment, and biography from session to session. This crate implements the same idea: a `CharacterSheet` that gets serialized into a `.nail` bundle (a tar.zst archive containing JSON manifests and TOML config) and deserialized back losslessly. The character remembers what it's learned, what equipment it has, and how it grew.

The `.nail` format isn't just serialization — it's a *contract*. The bundle contains four files (`manifest.json`, `identity.json`, `config.toml`, `reflexes.json`) with clear schemas, version tracking, and migration paths. When the format evolves, `VersionMigration` handles the upgrade. When a character needs to be exported for sharing or backup, `CharacterExporter` handles the serialization. When a new session loads a character, `CharacterImporter` handles validation.

## Architecture

```text
CharacterSheet (in-memory)
    │
    ├── NailConverter::to_nail() ──► NailBundle (structured intermediate)
    │                                    │
    │                                    ├── NailConverter::to_tar_zst() ──► .nail file
    │                                    │
    │                                    └── NailConverter::from_tar_zst() ◄── .nail file
    │
    └── NailConverter::from_nail() ◄── NailBundle

.nail bundle structure:
├── manifest.json    # version, name, class, level, generation, parent
├── identity.json    # stats, abilities, biography
├── config.toml      # model config, sandbox settings, trust thresholds, inventory
└── reflexes.json    # reflex patterns and actions
```

### Key Types

- **`CharacterSheet`** — The full character: stats, abilities, equipment, inventory, biography
- **`Stats`** — Six core stats: perception, dexterity, intelligence, wisdom, charisma, constitution
- **`Ability`** — Named ability with type (Innate/Learned/Granted/Reflex), trust level, mastery
- **`Equipment`** — Model config, sandbox settings, trust thresholds
- **`Inventory`** — Loaded skill packs and consumed `.nail` imports
- **`NailConverter`** — Bidirectional lossless conversion between `CharacterSheet` and `NailBundle`
- **`CharacterExporter`** — Serialize to `.nail` tar.zst archives
- **`CharacterImporter`** — Deserialize and validate `.nail` files
- **`VersionMigration`** — Upgrade older format versions to current

### The .nail Bundle Format

A `.nail` file is a zstd-compressed tar archive. Each file has a defined role:

| File | Format | Purpose |
|------|--------|---------|
| `manifest.json` | JSON | Version, identity metadata, lineage |
| `identity.json` | JSON | Stats, abilities, biography entries |
| `config.toml` | TOML | Model config, sandbox, trust thresholds |
| `reflexes.json` | JSON | Reflex patterns (intent→action pairs) |

JSON for structured data that needs deep nesting. TOML for config that humans might edit by hand. The separation is intentional.

## Usage

```rust
use character_sheet::models::CharacterSheet;
use character_sheet::nail::NailConverter;

// Create a fresh character
let mut sheet = CharacterSheet::new("Alice", "Assistant");
sheet.level_up("First successful interaction");

// Add abilities
sheet.add_ability(character_sheet::models::Ability {
    name: "greeting".into(),
    kind: character_sheet::models::AbilityType::Innate,
    trust: 0.95,
    level: 1,
    mastered: false,
});

// Load a skill pack
sheet.load_skill_pack("web-search", "2.1.0");

// Convert to .nail bundle
let bundle = NailConverter::to_nail(&sheet);

// Serialize to disk format (tar.zst)
let bytes = NailConverter::to_tar_zst(&bundle).unwrap();

// Round-trip: deserialize back
let bundle2 = NailConverter::from_tar_zst(&bytes).unwrap();
let sheet2 = NailConverter::from_nail(&bundle2);
assert_eq!(sheet, sheet2); // Lossless!

// Read the biography
println!("{}", sheet.biography_text());
// "Level 1: Character created (2026-01-15T10:30:00Z)"
// "Level 2: First successful interaction (2026-01-15T10:35:00Z)"
```

## API Reference

### `CharacterSheet`
- `CharacterSheet::new(name, class)` — Create a level-1 character with default stats
- `.add_ability(ability)` — Add or replace an ability by name
- `.level_up(reason)` — Increment level, append biography entry
- `.load_skill_pack(name, version)` — Add to inventory
- `.biography_text()` — Human-readable biography string

### `Stats`
- `Stats::default()` — All stats start at 10
- `.total()` — Sum of all six stats

### `Ability`
- Fields: `name`, `kind` (`AbilityType`), `trust` (0.0–1.0), `level`, `mastered`

### `AbilityType`
- `Innate` — Built-in from creation
- `Learned` — Acquired through XP
- `Granted` — Given by a parent/system
- `Reflex` — Automatic pattern-response

### `Equipment`
- `model_config` — `ModelConfig { provider, model, temperature, max_tokens }`
- `sandbox` — `SandboxSettings { enabled, network, fs_write, exec }`
- `trust_thresholds` — `TrustThresholds { auto_approve, ask_user, deny }`

### `NailConverter`
- `to_nail(sheet)` → `NailBundle` — Lossless conversion
- `from_nail(bundle)` → `CharacterSheet` — Lossless conversion
- `to_tar_zst(bundle)` → `Vec<u8>` — Serialize to `.nail` file bytes
- `from_tar_zst(data)` → `NailBundle` — Deserialize from `.nail` file bytes

### `NailError`
- `MissingFile(&str)` — Required file absent from bundle
- `Json`, `TomlDe`, `TomlSe`, `Io` — Wrapped serialization/IO errors

## The Deeper Idea

The character sheet is the persistent identity of an agent. It's not just config — it's a living document that records growth. The `biography` field auto-generates entries on level-ups, creating a narrative of the character's development. The `generation` and `parent` fields track lineage: when one character spawns another, the chain is traceable.

The `.nail` format is designed for interoperability. Other tools in the ecosystem (pincher, flux-core) can read and write `.nail` bundles without depending on this crate — the format is just tar.zst with known schemas. This crate provides the canonical Rust implementation.

Equipment as model config is deliberate: swapping models mid-session is like changing weapons mid-dungeon. The character adapts, but the history remains.

## Related Crates

- [`character-encounter`](../character-encounter) — The encounter engine that runs against loaded character sheets
- [`pincher-flux-bridge`](../pincher-flux-bridge) — Converts reflex actions from `.nail` bundles into flux IR
- [`ternary-auto-vectorizer`](../ternary-auto-vectorizer) — Compiler optimization for ternary operations used in ability matching
