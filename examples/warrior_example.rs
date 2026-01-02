#![allow(deprecated)]

//! Warrior Class Example
//!
//! This example demonstrates the template system with a complete warrior character.
//! It shows how to use templates with parameters and handle stat dependencies.
//!
//! Breakdown:
//! - WarriorHP: Depends on Vitality and Strength, has class bonus
//! - WarriorVitality: Base stat with level scaling
//! - WarriorATK: Depends on Strength, has conditional bonus
//! - WarriorStrength: Base stat with level scaling
//!
//! Key Features:
//! - Template parameters ({{base_hp}}, {{level}}, etc.)
//! - Stat dependencies (HP depends on Vitality and Strength)
//! - Conditional transforms (ATK bonus when Strength >= 50)
//! - Multiple sources (constant, scaling, map)

use std::collections::HashMap;
use std::fs;
use zzstat::{StatContext, StatId};
use zzstat_json::StatTemplateManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Warrior Class Example ===\n");

    // Load template configuration
    let json_path = format!("{}/examples/warrior.json", env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(&json_path)?;
    let manager = StatTemplateManager::from_json(&json)?;
    let mut resolver = zzstat::StatResolver::new();
    let context = StatContext::new();

    println!("Loaded warrior.json template configuration\n");

    // Character parameters
    let level = 10.0;
    let base_strength = 20.0;
    let strength_per_level = 2.5;
    let base_vitality = 15.0;
    let vitality_per_level = 2.0;
    let base_hp = 150.0;
    let hp_per_level = 12.0;
    let base_atk = 30.0;
    let atk_per_level = 4.0;
    let atk_bonus = 15.0; // Equipment bonus
    let atk_multiplier = 1.0;
    let max_atk = 500.0;

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
    println!("  Base ATK: {}, Per Level: {}", base_atk, atk_per_level);
    println!("  Equipment ATK Bonus: {}", atk_bonus);
    println!();

    // Apply templates in dependency order (base stats first, then dependent stats)
    // 1. Strength (no dependencies)
    let mut strength_params = HashMap::new();
    strength_params.insert("base_strength".to_string(), base_strength);
    strength_params.insert("strength_per_level".to_string(), strength_per_level);
    strength_params.insert("level".to_string(), level);

    manager.apply_template(
        &mut resolver,
        "WarriorStrength",
        "warrior:Strength",
        &strength_params,
    )?;

    // 2. Vitality (no dependencies)
    let mut vitality_params = HashMap::new();
    vitality_params.insert("base_vitality".to_string(), base_vitality);
    vitality_params.insert("vitality_per_level".to_string(), vitality_per_level);
    vitality_params.insert("level".to_string(), level);

    manager.apply_template(
        &mut resolver,
        "WarriorVitality",
        "warrior:Vitality",
        &vitality_params,
    )?;

    // 3. HP (depends on Strength and Vitality)
    let mut hp_params = HashMap::new();
    hp_params.insert("base_hp".to_string(), base_hp);
    hp_params.insert("hp_per_level".to_string(), hp_per_level);
    hp_params.insert("level".to_string(), level);

    manager.apply_template(&mut resolver, "WarriorHP", "warrior:HP", &hp_params)?;

    // 4. ATK (depends on Strength)
    let mut atk_params = HashMap::new();
    atk_params.insert("base_atk".to_string(), base_atk);
    atk_params.insert("atk_per_level".to_string(), atk_per_level);
    atk_params.insert("level".to_string(), level);
    atk_params.insert("atk_bonus".to_string(), atk_bonus);
    atk_params.insert("atk_multiplier".to_string(), atk_multiplier);
    atk_params.insert("max_atk".to_string(), max_atk);

    manager.apply_template(&mut resolver, "WarriorATK", "warrior:ATK", &atk_params)?;

    // Resolve stats
    println!("--- Stat Resolution ---\n");

    // Strength
    let strength_id = StatId::from_str("warrior:Strength");
    let strength = resolver.resolve(&strength_id, &context)?;
    println!("Strength:");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_strength,
        strength_per_level,
        level,
        base_strength + strength_per_level * level
    );
    println!("  Clamped (1-200): {:.2}", strength.value);
    println!();

    // Vitality
    let vitality_id = StatId::from_str("warrior:Vitality");
    let vitality = resolver.resolve(&vitality_id, &context)?;
    println!("Vitality:");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_vitality,
        vitality_per_level,
        level,
        base_vitality + vitality_per_level * level
    );
    println!("  Clamped (1-100): {:.2}", vitality.value);
    println!();

    // HP (depends on Strength and Vitality)
    let hp_id = StatId::from_str("warrior:HP");
    let hp = resolver.resolve(&hp_id, &context)?;

    // HP calculation with map sources (depends on Strength and Vitality)
    let strength_contribution = strength.value * 2.0;
    let vitality_contribution = vitality.value * 3.0;
    let base_hp_total = base_hp + hp_per_level * level;
    let hp_sources = base_hp_total + strength_contribution + vitality_contribution;
    let hp_with_bonus = hp_sources * 1.2; // +20% class bonus
    let final_hp = hp_with_bonus.max(100.0); // clamp min: 100.0

    println!("HP:");
    println!(
        "  Base HP: {} + ({} × {}) = {}",
        base_hp, hp_per_level, level, base_hp_total
    );
    println!(
        "  Strength contribution (Strength × 2): {:.2} × 2 = {:.2}",
        strength.value, strength_contribution
    );
    println!(
        "  Vitality contribution (Vitality × 3): {:.2} × 3 = {:.2}",
        vitality.value, vitality_contribution
    );
    println!("  Total sources: {:.2}", hp_sources);
    println!("  Class bonus (+20%): × 1.2");
    println!("  Clamped (min: 100): {:.2}", hp.value);
    println!("  Expected (with map sources): {:.2}", final_hp);
    if (hp.value - final_hp).abs() < 0.01 {
        println!("  ✅ HP calculation correct!");
    } else {
        println!("  ⚠️  HP differs from expected (map sources may need dependency resolution)");
        println!(
            "  Actual: {:.2}, Expected: {:.2}, Difference: {:.2}",
            hp.value,
            final_hp,
            (hp.value - final_hp).abs()
        );
    }
    println!();

    // ATK (depends on Strength, has conditional bonus)
    let atk_id = StatId::from_str("warrior:ATK");
    let atk = resolver.resolve(&atk_id, &context)?;

    let strength_atk_contribution = strength.value * 1.5;
    let base_atk_total = base_atk + atk_per_level * level;
    let atk_sources = base_atk_total + strength_atk_contribution;
    let atk_after_additive = atk_sources + atk_bonus;
    let conditional_multiplier = if strength.value >= 50.0 { 1.15 } else { 1.0 };
    let final_atk = (atk_after_additive * atk_multiplier * conditional_multiplier).min(max_atk);

    println!("ATK:");
    println!(
        "  Base ATK: {} + ({} × {}) = {}",
        base_atk, atk_per_level, level, base_atk_total
    );
    println!(
        "  Strength contribution (Strength × 1.5): {:.2} × 1.5 = {:.2}",
        strength.value, strength_atk_contribution
    );
    println!("  Total sources: {:.2}", atk_sources);
    println!("  Equipment bonus: + {}", atk_bonus);
    println!("  After additive: {:.2}", atk_after_additive);
    println!(
        "  Conditional bonus (Strength >= 50): {}",
        if strength.value >= 50.0 {
            "× 1.15 (+15% High Strength)"
        } else {
            "× 1.0 (no bonus)"
        }
    );
    println!("  Final ATK: {:.2}", atk.value);
    println!("  Expected (with map sources): {:.2}", final_atk);
    if (atk.value - final_atk).abs() < 0.01 {
        println!("  ✅ ATK calculation correct!");
    } else {
        println!("  ⚠️  ATK differs from expected (map sources may need dependency resolution)");
        println!(
            "  Actual: {:.2}, Expected: {:.2}, Difference: {:.2}",
            atk.value,
            final_atk,
            (atk.value - final_atk).abs()
        );
    }
    println!();

    println!("✅ All warrior stats resolved successfully!");
    println!("\nWarrior Character Summary:");
    println!("  Level: {}", level);
    println!("  Strength: {:.2}", strength.value);
    println!("  Vitality: {:.2}", vitality.value);
    println!("  HP: {:.2}", hp.value);
    println!("  ATK: {:.2}", atk.value);

    Ok(())
}
