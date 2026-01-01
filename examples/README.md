# Examples

This directory contains comprehensive examples demonstrating the `zzstat-json` library features.

## 🎯 **Production-Ready Example**

### Archer Entity Example ⭐ **START HERE**

**The most comprehensive and production-ready example** - A complete entity system that demonstrates real-world game stat management patterns.

```bash
cargo run --example archer_entity_example
```

This example is **the best reference** for building a complete stat system. It includes:

- ✅ **Complete Entity Struct** - Full Rust struct implementation with all game stats
- ✅ **Comprehensive Stat System**:
  - Base stats (HP, ATK, Dexterity, Vitality, Intelligence, Accuracy)
  - Experience gain with level scaling
  - Health and mana regeneration
  - Movement speed with agility/dexterity dependencies
  - Status effects (bleed, burn, poison damage)
  - Elemental resistances (fire, ice, lightning, poison)
- ✅ **Full Equipment System**:
  - Weapons with attack bonuses and enemy-type specific damage (`StrongAgainstBeast`, `StrongAgainstUndead`)
  - Armor with vitality and defense bonuses
  - Automatic stat invalidation when equipment changes
- ✅ **Enemy-Type Bonuses** - Spesifik enemy type'lara karşı damage bonusları (breakdown ile gösterim)
- ✅ **Stat Dependency Management** - Vitality değiştiğinde otomatik olarak HP, regeneration ve resistances yeniden hesaplanıyor
- ✅ **Detailed Breakdowns** - Tüm stat'ların hesaplama detayları gösteriliyor

**This example demonstrates exactly how you'd structure stats in a real game project.**

## JSON Configuration Files

- `basic_stats.json` - Direct stat definitions (no templates)
- `warrior.json` - Warrior class templates with stat dependencies
- `mage.json` - Mage class templates
- `rogue.json` - Rogue class with conditional transforms
- `paladin.json` - Paladin tank class with complex dependencies
- `archer.json` - Archer class with accuracy system
- `resistance.json` - Element resistance stats
- `mana_pool.json` - Mana system with regeneration
- `movement_speed.json` - Movement speed calculation
- `experience_gain.json` - Experience gain multiplier
- `health_regeneration.json` - HP regeneration over time
- `status_effects.json` - Damage over time effects
- `strong_against.json` - Damage bonus template
- `strong_against_undead.json` - Undead-specific damage bonus
- `complex_dependencies.json` - Complex multi-stat dependencies

## Other Rust Examples

### Basic Stats Example
```bash
cargo run --example basic_stats_example
```
Demonstrates direct stat definitions without templates. Shows:
- Multiple constant sources
- Scaling sources
- Additive and multiplicative transforms
- Clamp transforms

### Warrior Example
```bash
cargo run --example warrior_example
```
Demonstrates template system with a complete warrior character:
- Template parameters
- Stat dependencies (HP depends on Strength and Vitality)
- Conditional transforms (ATK bonus when Strength >= 50)
- Multiple sources (constant, scaling, map)

### Rogue Example
```bash
cargo run --example rogue_example
```
Demonstrates advanced conditional transforms:
- Multiple conditional bonuses based on Agility thresholds
- Agility-based critical chance
- Class penalties and bonuses
- Nested conditional transforms

### Paladin Example
```bash
cargo run --example paladin_example
```
Demonstrates tank class with complex stat interactions:
- Multiple stat dependencies (HP and Defense depend on 2 stats each)
- Conditional bonuses for high Vitality
- Complex stat interactions

### Resistance Example
```bash
cargo run --example resistance_example
```
Demonstrates element resistance system:
- Multiple similar stats with shared dependencies
- Vitality-based scaling
- Conditional bonuses

### Mana Pool Example
```bash
cargo run --example mana_pool_example
```
Demonstrates resource system:
- Mana pool with Intelligence and Vitality dependencies
- Mana regeneration over time
- Conditional bonuses for high Intelligence

### Complex Dependencies Example
```bash
cargo run --example complex_dependencies_example
```
Demonstrates multi-layer stat dependencies:
- Multi-layer dependency chains (HP → Defense → Strength/Vitality)
- Multiple map transforms on same stat
- Conditional transforms based on dependent stats
- Automatic dependency resolution order

## Features Demonstrated

### Source Types
- **Constant**: Fixed values
- **Scaling**: Level-based scaling (base + scale × level)
- **Map**: Dependent on other stats (sum of dependencies × multiplier)

### Transform Types
- **Additive**: Adds a value
- **Multiplicative**: Multiplies by a value
- **Clamp**: Bounds the value (min/max)
- **Conditional**: Applies different transforms based on stat values
- **Map**: Adds values from dependent stats (sum of dependencies × multiplier)

### Template System
- Parameterizable templates with `{{param_name}}` syntax
- Entity-based stat assignment
- Dependency resolution
- Reusable stat definitions

## Notes

- **Map transforms** (not sources) should be used for stat dependencies - zzstat's dependency graph automatically resolves transform dependencies
- Conditional transforms check stat values at resolution time
- All examples include detailed breakdowns and calculations
- Each example validates expected vs actual values
- When stats change (e.g., equipment), use `resolver.invalidate()` to clear cache and force recalculation
- Entity-based stats use `entity_id:stat_type` format (e.g., `"archer:HP"`)

## Quick Start Recommendation

1. **Start with** `archer_entity_example` - See how a complete system works
2. **Then explore** other examples for specific features
3. **Use** the JSON templates as reference for your own stat definitions

