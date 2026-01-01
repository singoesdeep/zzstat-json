//! Resistance Stats Example
//!
//! This example demonstrates element resistance stats that scale with Vitality.
//! Shows how to create multiple similar stats with shared dependencies.
//!
//! Breakdown:
//! - FireResistance: Fire damage reduction, scales with Vitality
//! - IceResistance: Ice damage reduction, scales with Vitality
//! - LightningResistance: Lightning damage reduction, scales with Vitality
//! - PoisonResistance: Poison damage reduction, higher Vitality scaling, has conditional bonus
//!
//! Key Features:
//! - Multiple resistance stats with similar structure
//! - Vitality-based scaling
//! - Conditional bonuses for high Vitality
//! - Clamp bounds (0-90% or 0-95%)

use std::collections::HashMap;
use std::fs;
use zzstat::{StatContext, StatId};
use zzstat_json::StatTemplateManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Resistance Stats Example ===\n");

    // Load template configuration
    let json_path = format!("{}/examples/resistance.json", env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(&json_path)?;
    let manager = StatTemplateManager::from_json(&json)?;
    let mut resolver = zzstat::StatResolver::new();
    let context = StatContext::new();

    println!("Loaded resistance.json template configuration\n");

    // Character parameters
    let level = 12.0;
    let base_vitality = 25.0;
    let vitality_per_level = 2.5;

    // Resistance parameters
    let base_fire_res = 10.0;
    let fire_res_per_level = 1.0;
    let fire_res_bonus = 5.0;

    let base_ice_res = 8.0;
    let ice_res_per_level = 0.8;
    let ice_res_bonus = 3.0;

    let base_lightning_res = 12.0;
    let lightning_res_per_level = 1.2;
    let lightning_res_bonus = 4.0;

    let base_poison_res = 5.0;
    let poison_res_per_level = 0.5;
    let poison_res_bonus = 2.0;

    println!("Character Parameters:");
    println!("  Level: {}", level);
    println!(
        "  Base Vitality: {}, Per Level: {}",
        base_vitality, vitality_per_level
    );
    println!();

    // Load Vitality template from warrior.json (or create a simple one)
    // For this example, we'll use warrior.json's Vitality template
    let warrior_json_path = format!("{}/examples/warrior.json", env!("CARGO_MANIFEST_DIR"));
    let warrior_json = fs::read_to_string(&warrior_json_path)?;
    let warrior_manager = StatTemplateManager::from_json(&warrior_json)?;

    // Apply Vitality first (needed for resistance calculations)
    let mut vitality_params = HashMap::new();
    vitality_params.insert("base_vitality".to_string(), base_vitality);
    vitality_params.insert("vitality_per_level".to_string(), vitality_per_level);
    vitality_params.insert("level".to_string(), level);

    warrior_manager.apply_template(
        &mut resolver,
        "WarriorVitality",
        "character:Vitality",
        &vitality_params,
    )?;

    // Resolve Vitality to get actual value
    let vitality_id = StatId::from_str("character:Vitality");
    let vitality = resolver.resolve(&vitality_id, &context)?;
    let vitality_value = vitality.value;

    println!("Vitality: {:.2}\n", vitality_value);

    // Apply resistance templates (using entity:stat format)
    let mut fire_params = HashMap::new();
    fire_params.insert("base_fire_resistance".to_string(), base_fire_res);
    fire_params.insert("fire_res_per_level".to_string(), fire_res_per_level);
    fire_params.insert("level".to_string(), level);
    fire_params.insert("fire_res_bonus".to_string(), fire_res_bonus);

    manager.apply_template(
        &mut resolver,
        "FireResistance",
        "character:FireResistance",
        &fire_params,
    )?;

    let mut ice_params = HashMap::new();
    ice_params.insert("base_ice_resistance".to_string(), base_ice_res);
    ice_params.insert("ice_res_per_level".to_string(), ice_res_per_level);
    ice_params.insert("level".to_string(), level);
    ice_params.insert("ice_res_bonus".to_string(), ice_res_bonus);

    manager.apply_template(
        &mut resolver,
        "IceResistance",
        "character:IceResistance",
        &ice_params,
    )?;

    let mut lightning_params = HashMap::new();
    lightning_params.insert("base_lightning_resistance".to_string(), base_lightning_res);
    lightning_params.insert(
        "lightning_res_per_level".to_string(),
        lightning_res_per_level,
    );
    lightning_params.insert("level".to_string(), level);
    lightning_params.insert("lightning_res_bonus".to_string(), lightning_res_bonus);

    manager.apply_template(
        &mut resolver,
        "LightningResistance",
        "character:LightningResistance",
        &lightning_params,
    )?;

    let mut poison_params = HashMap::new();
    poison_params.insert("base_poison_resistance".to_string(), base_poison_res);
    poison_params.insert("poison_res_per_level".to_string(), poison_res_per_level);
    poison_params.insert("level".to_string(), level);
    poison_params.insert("poison_res_bonus".to_string(), poison_res_bonus);

    manager.apply_template(
        &mut resolver,
        "PoisonResistance",
        "character:PoisonResistance",
        &poison_params,
    )?;

    println!("--- Resistance Calculations ---\n");

    // Fire Resistance
    let fire_id = StatId::from_str("character:FireResistance");
    let fire_res = resolver.resolve(&fire_id, &context)?;
    let vitality_fire_contribution: f64 = vitality_value * 0.2;
    let base_fire_total: f64 = base_fire_res + fire_res_per_level * level;
    let fire_sources: f64 = base_fire_total + vitality_fire_contribution;
    let fire_after_bonus: f64 = fire_sources + fire_res_bonus;
    let conditional_bonus: f64 = if vitality_value >= 50.0 { 5.0 } else { 0.0 };
    let final_fire: f64 = (fire_after_bonus + conditional_bonus).min(90.0);

    println!("Fire Resistance:");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_fire_res, fire_res_per_level, level, base_fire_total
    );
    println!(
        "  Vitality contribution (Vitality × 0.2%): {:.2} × 0.2 = {:.2}%",
        vitality_value, vitality_fire_contribution
    );
    println!("  Total sources: {:.2}%", fire_sources);
    println!("  Equipment bonus: + {}%", fire_res_bonus);
    println!(
        "  Conditional bonus (Vitality >= 50): {}",
        if vitality_value >= 50.0 {
            "+ 5% (Natural Resistance)"
        } else {
            "+ 0%"
        }
    );
    println!("  Clamped (0-90%): {:.2}%", fire_res.value);
    println!("  Expected (with map sources): {:.2}%", final_fire);
    if (fire_res.value - final_fire).abs() < 0.01 {
        println!("  ✅ Fire Resistance calculation correct!");
    } else {
        println!(
            "  ⚠️  Fire Resistance differs from expected (map sources may need dependency resolution)"
        );
        println!(
            "  Actual: {:.2}%, Expected: {:.2}%, Difference: {:.2}%",
            fire_res.value,
            final_fire,
            (fire_res.value - final_fire).abs()
        );
    }
    println!();

    // Ice Resistance
    let ice_id = StatId::from_str("character:IceResistance");
    let ice_res = resolver.resolve(&ice_id, &context)?;
    let vitality_ice_contribution: f64 = vitality_value * 0.2;
    let base_ice_total: f64 = base_ice_res + ice_res_per_level * level;
    let ice_sources: f64 = base_ice_total + vitality_ice_contribution;
    let final_ice: f64 = (ice_sources + ice_res_bonus).min(90.0);

    println!("Ice Resistance:");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_ice_res, ice_res_per_level, level, base_ice_total
    );
    println!(
        "  Vitality contribution (Vitality × 0.2%): {:.2} × 0.2 = {:.2}%",
        vitality_value, vitality_ice_contribution
    );
    println!("  Total sources: {:.2}%", ice_sources);
    println!("  Equipment bonus: + {}%", ice_res_bonus);
    println!("  Clamped (0-90%): {:.2}%", ice_res.value);
    println!("  Expected (with map sources): {:.2}%", final_ice);
    if (ice_res.value - final_ice).abs() < 0.01 {
        println!("  ✅ Ice Resistance calculation correct!");
    } else {
        println!(
            "  ⚠️  Ice Resistance differs from expected (map sources may need dependency resolution)"
        );
        println!(
            "  Actual: {:.2}%, Expected: {:.2}%, Difference: {:.2}%",
            ice_res.value,
            final_ice,
            (ice_res.value - final_ice).abs()
        );
    }
    println!();

    // Lightning Resistance
    let lightning_id = StatId::from_str("character:LightningResistance");
    let lightning_res = resolver.resolve(&lightning_id, &context)?;
    let vitality_lightning_contribution: f64 = vitality_value * 0.2;
    let base_lightning_total: f64 = base_lightning_res + lightning_res_per_level * level;
    let lightning_sources: f64 = base_lightning_total + vitality_lightning_contribution;
    let final_lightning: f64 = (lightning_sources + lightning_res_bonus).min(90.0);

    println!("Lightning Resistance:");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_lightning_res, lightning_res_per_level, level, base_lightning_total
    );
    println!(
        "  Vitality contribution (Vitality × 0.2%): {:.2} × 0.2 = {:.2}%",
        vitality_value, vitality_lightning_contribution
    );
    println!("  Total sources: {:.2}%", lightning_sources);
    println!("  Equipment bonus: + {}%", lightning_res_bonus);
    println!("  Clamped (0-90%): {:.2}%", lightning_res.value);
    println!("  Expected (with map sources): {:.2}%", final_lightning);
    if (lightning_res.value - final_lightning).abs() < 0.01 {
        println!("  ✅ Lightning Resistance calculation correct!");
    } else {
        println!(
            "  ⚠️  Lightning Resistance differs from expected (map sources may need dependency resolution)"
        );
        println!(
            "  Actual: {:.2}%, Expected: {:.2}%, Difference: {:.2}%",
            lightning_res.value,
            final_lightning,
            (lightning_res.value - final_lightning).abs()
        );
    }
    println!();

    // Poison Resistance (has conditional bonus)
    let poison_id = StatId::from_str("character:PoisonResistance");
    let poison_res = resolver.resolve(&poison_id, &context)?;
    let vitality_poison_contribution: f64 = vitality_value * 0.3; // Higher multiplier
    let base_poison_total: f64 = base_poison_res + poison_res_per_level * level;
    let poison_sources: f64 = base_poison_total + vitality_poison_contribution;
    let poison_after_bonus: f64 = poison_sources + poison_res_bonus;
    let conditional_bonus: f64 = if vitality_value >= 70.0 { 10.0 } else { 0.0 };
    let final_poison: f64 = (poison_after_bonus + conditional_bonus).min(95.0);

    println!("Poison Resistance:");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_poison_res, poison_res_per_level, level, base_poison_total
    );
    println!(
        "  Vitality contribution (Vitality × 0.3%): {:.2} × 0.3 = {:.2}%",
        vitality_value, vitality_poison_contribution
    );
    println!("  Total sources: {:.2}%", poison_sources);
    println!("  Equipment bonus: + {}%", poison_res_bonus);
    println!(
        "  Conditional bonus (Vitality >= 70): {}",
        if vitality_value >= 70.0 {
            "+ 10% (Immunity)"
        } else {
            "+ 0%"
        }
    );
    println!("  Clamped (0-95%): {:.2}%", poison_res.value);
    println!("  Expected (with map sources): {:.2}%", final_poison);
    if (poison_res.value - final_poison).abs() < 0.01 {
        println!("  ✅ Poison Resistance calculation correct!");
    } else {
        println!(
            "  ⚠️  Poison Resistance differs from expected (map sources may need dependency resolution)"
        );
        println!(
            "  Actual: {:.2}%, Expected: {:.2}%, Difference: {:.2}%",
            poison_res.value,
            final_poison,
            (poison_res.value - final_poison).abs()
        );
    }
    println!();

    println!("✅ All resistance stats resolved successfully!");
    println!("\nResistance Summary:");
    println!("  Fire Resistance: {:.2}%", fire_res.value);
    println!("  Ice Resistance: {:.2}%", ice_res.value);
    println!("  Lightning Resistance: {:.2}%", lightning_res.value);
    println!("  Poison Resistance: {:.2}%", poison_res.value);

    Ok(())
}
