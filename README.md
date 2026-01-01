# zzstat-json

JSON-based stat configuration for the `zzstat` crate. Define and manage your game stats easily using JSON files.

## Features

- ✅ JSON format for stat definitions
- ✅ **Stat templates** - Parameterizable templates for reusable stat definitions
- ✅ Multiple source types (constant, scaling)
- ✅ Transform support (multiplicative, additive, clamp, conditional, map)
- ✅ Dependency resolution (stats can depend on other stats)
- ✅ **Entity-based stat assignment** - Assign stats to entities using templates
- ✅ **Entity-based stat management** - JSON serialization, entity-based stat management
- ✅ **Flexible structure** - Equipment, buff features are left to the user
- ✅ Full integration with zzstat

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zzstat-json = "0.1"
zzstat = "0.1.4"
```

## Usage

### 1. Basic Usage (Direct Stat Definition)

```rust
use zzstat_json::load_from_json;
use zzstat::{StatId, StatContext};

let json = r#"
{
  "stats": {
    "HP": {
      "sources": [
        {"type": "constant", "value": 100.0, "name": "Base HP value"},
      ],
      "transforms": [{"type": "clamp", "min": 50.0, "max": null, "name": "Minimum HP guarantee"}
      ]
    }
  }
}
"#;

let mut resolver = load_from_json(json)?;
let context = StatContext::new();
let hp_id = StatId::from_str("HP");
let resolved = resolver.resolve(&hp_id, &context)?;

println!("HP: {}", resolved.value); // 225.0 = (100 + 50) * 1.5
```

### 2. Template System (Recommended)

The template system allows you to customize stat definitions with parameters and assign them to different entities.

#### Defining Templates in JSON

```json
{
  "templates": {
    "BaseHP": {
      "description": "Base HP template - affected by Vitality",
      "sources": [
        {
          "type": "constant",
          "value": "{{base_hp}}",
          "name": "Base HP value"
        },
        {
          "type": "scaling",
          "base": 0.0,
          "scale": "{{hp_per_level}}",
          "level": "{{level}}",
          "name": "Level-based HP scaling"
        }
      ],
      "transforms": [
        {
          "type": "map",
          "dependencies": ["Vitality"],
          "multiplier": 4.0,
          "name": "HP bonus from Vitality (Vitality * 4)"
        },
        {
          "type": "clamp",
          "min": 50.0,
          "max": null,
          "name": "Minimum HP guarantee"
        }
      ]
    },
    "Vitality": {
      "description": "Vitality stat template - affects HP",
      "sources": [
        {
          "type": "constant",
          "value": "{{base_vitality}}",
          "name": "Base Vitality value"
        },
        {
          "type": "scaling",
          "base": 0.0,
          "scale": "{{vitality_per_level}}",
          "level": "{{level}}",
          "name": "Level-based Vitality scaling"
        }
      ],
      "transforms": [
        {
          "type": "clamp",
          "min": 1.0,
          "max": 100.0,
          "name": "Vitality bounds (1-100)"
        }
      ]
    }
  }
}
```

#### Using in Code

```rust
use zzstat_json::StatTemplateManager;
use zzstat::{StatId, StatContext};
use std::collections::HashMap;
use std::fs;

// Create template manager from JSON
let json = fs::read_to_string("templates.json")?;
let manager = StatTemplateManager::from_json(&json)?;

// Create stat resolver for a character
let mut resolver = zzstat::StatResolver::new();

// Prepare parameters
let mut hp_params = HashMap::new();
hp_params.insert("base_hp".to_string(), 150.0);
hp_params.insert("hp_per_level".to_string(), 10.0);
hp_params.insert("level".to_string(), 5.0);
hp_params.insert("min_hp".to_string(), 100.0);

let mut vitality_params = HashMap::new();
vitality_params.insert("base_vitality".to_string(), 10.0);
vitality_params.insert("vitality_per_level".to_string(), 1.5);
vitality_params.insert("level".to_string(), 5.0);

// IMPORTANT: Apply Vitality first, then HP (HP depends on Vitality)
// Apply templates to character
manager.apply_template(
    &mut resolver,
    "Vitality",        // Template name
    "player1_Vitality", // Stat name (entity-specific)
    &vitality_params,   // Parameters
)?;

manager.apply_template(
    &mut resolver,
    "BaseHP",          // Template name
    "player1_HP",      // Stat name (entity-specific)
    &hp_params,        // Parameters
)?;

// Resolve stats
let context = StatContext::new();
let hp = resolver.resolve(&StatId::from_str("player1_HP"), &context)?;
let vitality = resolver.resolve(&StatId::from_str("player1_Vitality"), &context)?;
println!("Player1 HP: {}, Vitality: {}", hp.value, vitality.value);
```

#### Using for Multiple Entities

```rust
// Use the same template with different parameters for different characters
let mut resolver = zzstat::StatResolver::new();

// Character 1
let mut params1 = HashMap::new();
params1.insert("base_hp".to_string(), 100.0);
params1.insert("level".to_string(), 5.0);
manager.apply_template(&mut resolver, "BaseHP", "char1_HP", &params1)?;

// Character 2
let mut params2 = HashMap::new();
params2.insert("base_hp".to_string(), 200.0);
params2.insert("level".to_string(), 10.0);
manager.apply_template(&mut resolver, "BaseHP", "char2_HP", &params2)?;
```

### JSON Format

#### Sources

Sources work additively - all source values are summed.

**Constant Source:**
```json
{
  "type": "constant",
  "value": 100.0,
  "name": "Base HP value"  // Optional: Description for readability
}
```

**Scaling Source:**
```json
{
  "type": "scaling",
  "base": 0.0,
  "scale": 10.0,
  "level": 5.0,
  "name": "Level-based scaling"  // Optional
}
```

**Map Transform (Dependent Stats):**
```json
{
  "type": "map",
  "dependencies": ["Vitality"],
  "multiplier": 4.0,
  "name": "HP bonus from Vitality"  // Optional
}
```
A transform that depends on other stats. Sums the values of dependent stats, multiplies by the multiplier, and adds to the current stat value. **Note:** Map dependencies must be defined as transforms (not sources), because zzstat's dependency graph only automatically resolves transform dependencies.

#### Transforms

Transforms are applied in order.

**Multiplicative Transform:**
```json
{
  "type": "multiplicative",
  "value": 1.5,
  "name": "+50% increase"  // Optional: For readability
}
```

**Additive Transform:**
```json
{
  "type": "additive",
  "value": 10.0,
  "name": "+10 bonus"  // Optional
}
```

**Clamp Transform:**
```json
{
  "type": "clamp",
  "min": 50.0,
  "max": 200.0,
  "name": "Value bounds"  // Optional
}
```

**Conditional Transform:**
```json
{
  "type": "conditional",
  "condition_stat": "Vitality",
  "condition_value": 20.0,
  "operator": ">=",
  "then": {
    "type": "multiplicative",
    "value": 1.2
  },
  "else_then": {
    "type": "multiplicative",
    "value": 1.0
  }
}
```
Applies different transforms based on a stat's value. Operators: `>`, `<`, `>=`, `<=`, `==`. `else_then` is optional.

### Example JSON File

```json
{
  "stats": {
    "HP": {
      "sources": [
        {
          "type": "constant",
          "value": 100.0,
          "name": "Base HP value"
        },
        {
          "type": "scaling",
          "base": 0.0,
          "scale": 10.0,
          "level": 5.0,
          "name": "Level-based HP (level 5)"
        }
      ],
      "transforms": [
        {
          "type": "clamp",
          "min": 50.0,
          "max": null,
          "name": "Minimum 50 HP guarantee"
        }
      ]
    },
    "Vitality": {
      "sources": [
        {
          "type": "constant",
          "value": 10.0,
          "name": "Base Vitality value"
        },
        {
          "type": "scaling",
          "base": 0.0,
          "scale": 2.0,
          "level": 5.0,
          "name": "Level-based Vitality (2 per level)"
        }
      ],
      "transforms": [
        {
          "type": "clamp",
          "min": 1.0,
          "max": 100.0,
          "name": "Vitality bounds (1-100)"
        }
      ]
    },
    "ATK": {
      "sources": [
        {
          "type": "constant",
          "value": 25.0,
          "name": "Base attack power"
        },
        {
          "type": "scaling",
          "base": 0.0,
          "scale": 3.0,
          "level": 5.0,
          "name": "Level-based ATK (3 per level)"
        }
      ],
      "transforms": [
        {
          "type": "multiplicative",
          "value": 1.1,
          "name": "+10% ATK bonus"
        },
        {
          "type": "clamp",
          "min": null,
          "max": 200.0,
          "name": "Maximum 200 ATK limit"
        }
      ]
    }
  }
}
```

## API

### Basic Functions

#### `load_from_json(json_content: &str) -> Result<StatResolver, YamlStatError>`

Creates a `StatResolver` from JSON content (for direct stat definitions).

#### `resolve_stat_from_json(json_content: &str, stat_name: &str) -> Result<ResolvedStat, YamlStatError>`

Directly resolves a stat from JSON content.

#### `create_entity_stats(json_content: &str, entity_name: &str, template_name: &str, params: &HashMap<String, f64>) -> Result<StatResolver, YamlStatError>`

Creates a stat resolver for an entity using a template.

### Template Manager

#### `StatTemplateManager::from_json(json_content: &str) -> Result<StatTemplateManager, YamlStatError>`

Creates a template manager from JSON.

#### `apply_template(&self, resolver: &mut StatResolver, template_name: &str, stat_name: &str, params: &HashMap<String, f64>) -> Result<(), YamlStatError>`

Applies a template with parameters to the resolver.

#### `apply_templates(&self, resolver: &mut StatResolver, applications: &[(String, String, HashMap<String, f64>)]) -> Result<(), YamlStatError>`

Applies multiple templates at once.

## Parameter System

In templates, you can use parameters with the `{{param_name}}` syntax:

- **In sources**: `value: "{{base_hp}}"`, `scale: "{{hp_per_level}}"`
- **In transforms**: `value: "{{multiplier}}"`
- **In clamp**: `min: "{{min_value}}"`, `max: "{{max_value}}"`

Parameters are provided in code as `HashMap<String, f64>`.

### 3. Entity-Based Usage (Recommended)

Use `StatTemplateManager` for entity-based stat management. This manager allows you to assign stats to entities using templates:

```rust
use zzstat_json::StatTemplateManager;
use std::collections::HashMap;

// Load templates (JSON)
let manager = StatTemplateManager::from_json(json)?;

// Single resolver - for all entities
let mut resolver = zzstat::StatResolver::new();

// Load entity data from database
let entity_configs = vec![
    EntityStatConfig {
        entity_id: "player_123".to_string(),
        stat_type: "HP".to_string(),
        template_name: "WarriorHP".to_string(),
        params: {
            let mut p = HashMap::new();
            p.insert("base_hp".to_string(), 150.0);
            p.insert("level".to_string(), 10.0);
            p
        },
    },
];

// Load entity stats
manager.load_entity_stats(&mut resolver, entity_configs)?;

// Resolve stat
let hp = manager.resolve_entity_stat(
    &mut resolver,
    "player_123",
    "HP",
    &StatContext::new(),
)?;
```

#### Entity-Based Stat Management

- **Templates**: Can be stored in JSON format
- **Entity Parameters**: Can be used as `EntityStatConfig`
- **Stat ID Format**: `entity_id:stat_type` (e.g., `"player_123:HP"`)
- **Single Resolver**: One resolver is used for all entities (efficient)

#### Equipment and Buff System

Equipment, buff features are left to the user. You can implement your own:

```rust
use zzstat_json::StatTemplateManager;
use zzstat::source::ConstantSource;
use zzstat_json::AdditiveTransform;

// Create manager
let manager = StatTemplateManager::from_json(json)?;
let mut resolver = zzstat::StatResolver::new();

// Load entity
manager.load_entity(&mut resolver, "player_123", configs)?;

// Add equipment stat (your own implementation)
manager.add_source_to_entity(
    &mut resolver,
    "player_123",
    "ATK",
    Box::new(ConstantSource(50.0)),
);

// Add buff (your own implementation)
manager.add_transform_to_entity(
    &mut resolver,
    "player_123",
    "ATK",
    Box::new(AdditiveTransform::new(10.0)),
);

// Resolve stat
let atk = manager.resolve_entity_stat(
    &mut resolver,
    "player_123",
    "ATK",
    &StatContext::new(),
)?;
```

This way, even if your game doesn't have equipment, or you use a different buff system, you can easily add your own implementation.

## Examples

Example files are in the `examples/` directory. See [examples/README.md](examples/README.md) for detailed documentation.

### 🎯 **Production-Ready Example: Archer Entity**

**The most comprehensive example** - A complete, production-ready entity system demonstrating real-world game stat management:

```bash
cargo run --example archer_entity_example
```

This example showcases:
- ✅ **Complete entity struct** with all game stats (HP, ATK, resistances, status effects, etc.)
- ✅ **Full equipment system** - Weapons and armor that modify stats dynamically
- ✅ **Enemy-type specific bonuses** - `StrongAgainstBeast`, `StrongAgainstUndead` with detailed breakdowns
- ✅ **Stat dependency invalidation** - Automatic recalculation when dependencies change
- ✅ **Comprehensive stat system** - Experience gain, health/mana regeneration, movement speed, status effects
- ✅ **Real-world patterns** - Exactly how you'd structure stats in an actual game

This is the **best starting point** for understanding how to build a complete stat system with `zzstat-json`.

### Other Examples

```bash
# Basic stat definitions
cargo run --example basic_stats_example

# Template system with warrior class
cargo run --example warrior_example

# Conditional transforms with rogue
cargo run --example rogue_example

# Complex dependencies with paladin
cargo run --example paladin_example

# Element resistances
cargo run --example resistance_example

# Mana system
cargo run --example mana_pool_example

# Complex multi-layer dependencies
cargo run --example complex_dependencies_example
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) file for details.

## Links

- [zzstat](https://docs.rs/zzstat) - Core stat engine
- [Repository](https://github.com/singoesdeep/zzstat-json)
