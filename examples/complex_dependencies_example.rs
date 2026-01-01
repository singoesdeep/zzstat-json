//! Complex Dependencies Example
//!
//! This example demonstrates complex stat dependencies with multiple layers.
//! Shows how stats can depend on each other in a chain and how zzstat's
//! dependency graph automatically resolves them in the correct order.
//!
//! Breakdown:
//! - BaseStrength: Foundation stat (no dependencies)
//! - BaseVitality: Foundation stat (no dependencies)
//! - ComplexDefense: Depends on Strength and Vitality
//! - ComplexHP: Depends on Strength, Vitality, AND Defense (which itself depends on Strength/Vitality)
//! - ComplexATK: Depends on Strength and Defense (which depends on Strength/Vitality)
//!
//! Key Features:
//! - Multi-layer dependencies (HP -> Defense -> Strength/Vitality)
//! - Circular dependency prevention (HP depends on Defense, ATK depends on Defense, but Defense doesn't depend on HP/ATK)
//! - Multiple map transforms on same stat
//! - Conditional transforms based on dependent stats
//! - Dependency resolution order demonstration

use std::collections::HashMap;
use std::fs;
use zzstat::{StatContext, StatId};
use zzstat_json::StatTemplateManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Complex Dependencies Example ===\n");

    // Load template configuration
    let json_path = format!(
        "{}/examples/complex_dependencies.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = fs::read_to_string(&json_path)?;
    let manager = StatTemplateManager::from_json(&json)?;
    let mut resolver = zzstat::StatResolver::new();
    let context = StatContext::new();

    println!("Loaded complex_dependencies.json template configuration\n");

    // Character parameters
    let level = 25.0;
    let base_strength = 30.0;
    let strength_per_level = 3.0;
    let base_vitality = 25.0;
    let vitality_per_level = 2.5;
    let base_hp = 200.0;
    let hp_per_level = 15.0;
    let hp_multiplier = 1.1;
    let min_hp = 100.0;
    let base_defense = 20.0;
    let defense_per_level = 2.0;
    let defense_bonus = 10.0;
    let defense_multiplier = 1.05;
    let base_atk = 50.0;
    let atk_per_level = 5.0;
    let atk_bonus = 20.0;
    let atk_multiplier = 1.0;
    let max_atk = 1000.0;

    println!("Character Parameters:");
    println!("  Level: {}", level);
    println!(
        "  Base Strength: {}, Per Level: {}",
        base_strength, strength_per_level
    );
    println!(
        "  Base Vitality: {}, Per Level: {}",
        base_vitality, vitality_per_level
    );
    println!("  Base HP: {}, Per Level: {}", base_hp, hp_per_level);
    println!(
        "  Base Defense: {}, Per Level: {}",
        base_defense, defense_per_level
    );
    println!("  Base ATK: {}, Per Level: {}", base_atk, atk_per_level);
    println!();

    // Apply templates in dependency order
    // 1. Base stats (no dependencies)
    println!("--- Applying Base Stats ---\n");

    let mut strength_params = HashMap::new();
    strength_params.insert("base_strength".to_string(), base_strength);
    strength_params.insert("strength_per_level".to_string(), strength_per_level);
    strength_params.insert("level".to_string(), level);

    manager.apply_template(
        &mut resolver,
        "BaseStrength",
        "character:Strength",
        &strength_params,
    )?;

    let mut vitality_params = HashMap::new();
    vitality_params.insert("base_vitality".to_string(), base_vitality);
    vitality_params.insert("vitality_per_level".to_string(), vitality_per_level);
    vitality_params.insert("level".to_string(), level);

    manager.apply_template(
        &mut resolver,
        "BaseVitality",
        "character:Vitality",
        &vitality_params,
    )?;

    // 2. Defense (depends on Strength and Vitality)
    println!("--- Applying Defense (depends on Strength, Vitality) ---\n");

    let mut defense_params = HashMap::new();
    defense_params.insert("base_defense".to_string(), base_defense);
    defense_params.insert("defense_per_level".to_string(), defense_per_level);
    defense_params.insert("level".to_string(), level);
    defense_params.insert("defense_bonus".to_string(), defense_bonus);
    defense_params.insert("defense_multiplier".to_string(), defense_multiplier);

    manager.apply_template(
        &mut resolver,
        "ComplexDefense",
        "character:Defense",
        &defense_params,
    )?;

    // 3. HP (depends on Strength, Vitality, AND Defense)
    println!("--- Applying HP (depends on Strength, Vitality, Defense) ---\n");

    let mut hp_params = HashMap::new();
    hp_params.insert("base_hp".to_string(), base_hp);
    hp_params.insert("hp_per_level".to_string(), hp_per_level);
    hp_params.insert("level".to_string(), level);
    hp_params.insert("hp_multiplier".to_string(), hp_multiplier);
    hp_params.insert("min_hp".to_string(), min_hp);

    manager.apply_template(&mut resolver, "ComplexHP", "character:HP", &hp_params)?;

    // 4. ATK (depends on Strength and Defense)
    println!("--- Applying ATK (depends on Strength, Defense) ---\n");

    let mut atk_params = HashMap::new();
    atk_params.insert("base_atk".to_string(), base_atk);
    atk_params.insert("atk_per_level".to_string(), atk_per_level);
    atk_params.insert("level".to_string(), level);
    atk_params.insert("atk_bonus".to_string(), atk_bonus);
    atk_params.insert("atk_multiplier".to_string(), atk_multiplier);
    atk_params.insert("max_atk".to_string(), max_atk);

    manager.apply_template(&mut resolver, "ComplexATK", "character:ATK", &atk_params)?;

    // Resolve stats
    println!("--- Stat Resolution (Dependency Graph Order) ---\n");

    // 1. Strength (no dependencies)
    let strength_id = StatId::from_str("character:Strength");
    let strength = resolver.resolve(&strength_id, &context)?;
    let base_strength_total = base_strength + strength_per_level * level;
    let expected_strength = base_strength_total.min(200.0);

    println!("1. Strength (no dependencies):");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_strength, strength_per_level, level, base_strength_total
    );
    println!("  Clamped (1-200): {:.2}", strength.value);
    println!("  Expected: {:.2}", expected_strength);
    if (strength.value - expected_strength).abs() < 0.01 {
        println!("  ✅ Strength calculation correct!\n");
    } else {
        println!("  ⚠️  Strength differs from expected\n");
    }

    // 2. Vitality (no dependencies)
    let vitality_id = StatId::from_str("character:Vitality");
    let vitality = resolver.resolve(&vitality_id, &context)?;
    let base_vitality_total = base_vitality + vitality_per_level * level;
    let expected_vitality = base_vitality_total.min(200.0);

    println!("2. Vitality (no dependencies):");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_vitality, vitality_per_level, level, base_vitality_total
    );
    println!("  Clamped (1-200): {:.2}", vitality.value);
    println!("  Expected: {:.2}", expected_vitality);
    if (vitality.value - expected_vitality).abs() < 0.01 {
        println!("  ✅ Vitality calculation correct!\n");
    } else {
        println!("  ⚠️  Vitality differs from expected\n");
    }

    // 3. Defense (depends on Strength and Vitality)
    let defense_id = StatId::from_str("character:Defense");
    let defense = resolver.resolve(&defense_id, &context)?;

    // Calculation: sources -> map transforms -> additive -> multiplicative -> conditional -> clamp
    let base_defense_total = base_defense + defense_per_level * level;
    let strength_defense_contribution = strength.value * 1.5;
    let vitality_defense_contribution = vitality.value * 2.0;

    let after_map_strength = base_defense_total + strength_defense_contribution;
    let after_map_vitality = after_map_strength + vitality_defense_contribution;
    let after_additive = after_map_vitality + defense_bonus;
    let after_multiplicative = after_additive * defense_multiplier;
    let conditional_multiplier = if strength.value >= 40.0 { 1.1 } else { 1.0 };
    let expected_defense = after_multiplicative * conditional_multiplier;

    println!("3. Defense (depends on Strength, Vitality):");
    println!(
        "  Base Defense: {} + ({} × {}) = {}",
        base_defense, defense_per_level, level, base_defense_total
    );
    println!(
        "  Map transform (Strength × 1.5): {:.2} × 1.5 = {:.2}",
        strength.value, strength_defense_contribution
    );
    println!("  After Strength map: {:.2}", after_map_strength);
    println!(
        "  Map transform (Vitality × 2.0): {:.2} × 2.0 = {:.2}",
        vitality.value, vitality_defense_contribution
    );
    println!("  After Vitality map: {:.2}", after_map_vitality);
    println!(
        "  Additive transform (Equipment bonus): + {} = {:.2}",
        defense_bonus, after_additive
    );
    println!(
        "  Multiplicative transform: × {} = {:.2}",
        defense_multiplier, after_multiplicative
    );
    println!(
        "  Conditional bonus (Strength >= 40): {}",
        if strength.value >= 40.0 {
            "× 1.1 (+10% Defense)"
        } else {
            "× 1.0 (no bonus)"
        }
    );
    println!("  Final Defense: {:.2}", defense.value);
    println!("  Expected: {:.2}", expected_defense);
    if (defense.value - expected_defense).abs() < 0.01 {
        println!("  ✅ Defense calculation correct!\n");
    } else {
        println!("  ⚠️  Defense differs from expected\n");
    }

    // 4. HP (depends on Strength, Vitality, AND Defense)
    let hp_id = StatId::from_str("character:HP");
    let hp = resolver.resolve(&hp_id, &context)?;

    // Calculation: sources -> map transforms -> multiplicative -> conditional -> clamp
    let base_hp_total = base_hp + hp_per_level * level;
    let strength_hp_contribution = strength.value * 3.0;
    let vitality_hp_contribution = vitality.value * 4.0;
    let defense_hp_contribution = defense.value * 2.0;

    let after_map_strength = base_hp_total + strength_hp_contribution;
    let after_map_vitality = after_map_strength + vitality_hp_contribution;
    let after_map_defense = after_map_vitality + defense_hp_contribution;
    let after_multiplicative = after_map_defense * hp_multiplier;
    let conditional_multiplier = if vitality.value >= 50.0 { 1.15 } else { 1.0 };
    let after_conditional = after_multiplicative * conditional_multiplier;
    let expected_hp = if after_conditional < min_hp {
        min_hp
    } else {
        after_conditional
    };

    println!("4. HP (depends on Strength, Vitality, Defense):");
    println!(
        "  Base HP: {} + ({} × {}) = {}",
        base_hp, hp_per_level, level, base_hp_total
    );
    println!(
        "  Map transform (Strength × 3.0): {:.2} × 3.0 = {:.2}",
        strength.value, strength_hp_contribution
    );
    println!("  After Strength map: {:.2}", after_map_strength);
    println!(
        "  Map transform (Vitality × 4.0): {:.2} × 4.0 = {:.2}",
        vitality.value, vitality_hp_contribution
    );
    println!("  After Vitality map: {:.2}", after_map_vitality);
    println!(
        "  Map transform (Defense × 2.0): {:.2} × 2.0 = {:.2}",
        defense.value, defense_hp_contribution
    );
    println!("  After Defense map: {:.2}", after_map_defense);
    println!(
        "  Multiplicative transform: × {} = {:.2}",
        hp_multiplier, after_multiplicative
    );
    println!(
        "  Conditional bonus (Vitality >= 50): {}",
        if vitality.value >= 50.0 {
            "× 1.15 (+15% HP)"
        } else {
            "× 1.0 (no bonus)"
        }
    );
    println!("  After conditional: {:.2}", after_conditional);
    println!("  Clamped (min: {}): {:.2}", min_hp, hp.value);
    println!("  Expected: {:.2}", expected_hp);
    if (hp.value - expected_hp).abs() < 0.01 {
        println!("  ✅ HP calculation correct!\n");
    } else {
        println!("  ⚠️  HP differs from expected\n");
    }

    // 5. ATK (depends on Strength and Defense)
    let atk_id = StatId::from_str("character:ATK");
    let atk = resolver.resolve(&atk_id, &context)?;

    // Calculation: sources -> map transform -> additive -> multiplicative -> conditionals -> clamp
    let base_atk_total = base_atk + atk_per_level * level;
    let strength_atk_contribution = strength.value * 2.0;

    let after_map = base_atk_total + strength_atk_contribution;
    let after_additive = after_map + atk_bonus;
    let after_multiplicative = after_additive * atk_multiplier;
    let conditional_strength_multiplier = if strength.value >= 50.0 { 1.2 } else { 1.0 };
    let after_conditional_strength = after_multiplicative * conditional_strength_multiplier;
    let conditional_defense_multiplier = if defense.value >= 30.0 { 0.9 } else { 1.0 };
    let after_conditional_defense = after_conditional_strength * conditional_defense_multiplier;
    let expected_atk = if after_conditional_defense > max_atk {
        max_atk
    } else {
        after_conditional_defense
    };

    println!("5. ATK (depends on Strength, Defense):");
    println!(
        "  Base ATK: {} + ({} × {}) = {}",
        base_atk, atk_per_level, level, base_atk_total
    );
    println!(
        "  Map transform (Strength × 2.0): {:.2} × 2.0 = {:.2}",
        strength.value, strength_atk_contribution
    );
    println!("  After map transform: {:.2}", after_map);
    println!(
        "  Additive transform (Equipment bonus): + {} = {:.2}",
        atk_bonus, after_additive
    );
    println!(
        "  Multiplicative transform: × {} = {:.2}",
        atk_multiplier, after_multiplicative
    );
    println!(
        "  Conditional bonus (Strength >= 50): {}",
        if strength.value >= 50.0 {
            "× 1.2 (+20% ATK)"
        } else {
            "× 1.0 (no bonus)"
        }
    );
    println!(
        "  After Strength conditional: {:.2}",
        after_conditional_strength
    );
    println!(
        "  Conditional penalty (Defense >= 30): {}",
        if defense.value >= 30.0 {
            "× 0.9 (-10% ATK - Heavy Armor)"
        } else {
            "× 1.0 (no penalty)"
        }
    );
    println!(
        "  After Defense conditional: {:.2}",
        after_conditional_defense
    );
    println!("  Clamped (max: {}): {:.2}", max_atk, atk.value);
    println!("  Expected: {:.2}", expected_atk);
    if (atk.value - expected_atk).abs() < 0.01 {
        println!("  ✅ ATK calculation correct!\n");
    } else {
        println!("  ⚠️  ATK differs from expected\n");
    }

    // Summary
    println!("=== Complex Dependencies Summary ===");
    println!("Dependency Chain:");
    println!("  Strength (no deps) →");
    println!("  Vitality (no deps) →");
    println!("  Defense (depends on Strength, Vitality) →");
    println!("  HP (depends on Strength, Vitality, Defense)");
    println!("  ATK (depends on Strength, Defense)");
    println!();
    println!("Final Stats:");
    println!("  Strength: {:.2}", strength.value);
    println!("  Vitality: {:.2}", vitality.value);
    println!("  Defense: {:.2}", defense.value);
    println!("  HP: {:.2}", hp.value);
    println!("  ATK: {:.2}", atk.value);
    println!();
    println!("✅ All stats resolved successfully!");
    println!("   zzstat's dependency graph automatically resolved dependencies in correct order.");

    Ok(())
}
