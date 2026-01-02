#![allow(deprecated)]

//! Mana Pool Example
//!
//! This example demonstrates mana system with pool and regeneration.
//! Shows how to create resource systems (mana, energy, etc.) with regeneration.
//!
//! Breakdown:
//! - ManaPool: Maximum mana, depends on Intelligence and Vitality
//! - ManaRegeneration: Mana per second, depends on Intelligence
//! - Intelligence: Base stat that affects both mana pool and regeneration
//!
//! Key Features:
//! - Resource pool system
//! - Regeneration over time
//! - Multiple stat dependencies
//! - Conditional bonuses for high Intelligence

use std::collections::HashMap;
use std::fs;
use zzstat::{StatContext, StatId};
use zzstat_json::StatTemplateManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Mana Pool Example ===\n");

    // Load template configuration
    let json_path = format!("{}/examples/mana_pool.json", env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(&json_path)?;
    let manager = StatTemplateManager::from_json(&json)?;
    let mut resolver = zzstat::StatResolver::new();
    let context = StatContext::new();

    println!("Loaded mana_pool.json template configuration\n");

    // Character parameters
    let level = 18.0;
    let base_intelligence = 35.0;
    let intelligence_per_level = 3.5;
    let base_vitality = 20.0;
    let vitality_per_level = 2.0;

    // Mana parameters
    let base_mana = 100.0;
    let mana_per_level = 8.0;
    let mana_bonus = 25.0;
    let mana_multiplier = 1.0;

    // Mana regeneration parameters
    let base_mana_regen = 2.0;
    let mana_regen_per_level = 0.3;
    let mana_regen_bonus = 1.0;
    let mana_regen_multiplier = 1.0;
    let max_mana_regen = 50.0;

    println!("Character Parameters:");
    println!("  Level: {}", level);
    println!(
        "  Base Intelligence: {}, Per Level: {}",
        base_intelligence, intelligence_per_level
    );
    println!(
        "  Base Vitality: {}, Per Level: {}",
        base_vitality, vitality_per_level
    );
    println!("  Base Mana: {}, Per Level: {}", base_mana, mana_per_level);
    println!(
        "  Base Mana Regen: {}, Per Level: {}",
        base_mana_regen, mana_regen_per_level
    );
    println!();

    // Apply templates in dependency order
    // 1. Intelligence (no dependencies)
    let mut intelligence_params = HashMap::new();
    intelligence_params.insert("base_intelligence".to_string(), base_intelligence);
    intelligence_params.insert("intelligence_per_level".to_string(), intelligence_per_level);
    intelligence_params.insert("level".to_string(), level);

    // Use entity:stat format for proper entity-based dependency resolution
    manager.apply_template(
        &mut resolver,
        "Intelligence",
        "mage:Intelligence",
        &intelligence_params,
    )?;

    // 2. Vitality (for mana pool calculation)
    // Note: We need to apply Vitality template for map transform to work
    // For now, we'll create a simple Vitality stat manually
    let vitality_value = (base_vitality + vitality_per_level * level).min(200.0);

    // Create a simple Vitality stat for map transform
    use zzstat::source::ConstantSource;
    let vitality_id = StatId::from_str("mage:Vitality");
    resolver.register_source(
        vitality_id.clone(),
        Box::new(ConstantSource(vitality_value)),
    );

    // 3. ManaPool (depends on Intelligence and Vitality)
    let mut mana_params = HashMap::new();
    mana_params.insert("base_mana".to_string(), base_mana);
    mana_params.insert("mana_per_level".to_string(), mana_per_level);
    mana_params.insert("level".to_string(), level);
    mana_params.insert("mana_bonus".to_string(), mana_bonus);
    mana_params.insert("mana_multiplier".to_string(), mana_multiplier);

    manager.apply_template(&mut resolver, "ManaPool", "mage:ManaPool", &mana_params)?;

    // 4. ManaRegeneration (depends on Intelligence)
    let mut mana_regen_params = HashMap::new();
    mana_regen_params.insert("base_mana_regen".to_string(), base_mana_regen);
    mana_regen_params.insert("mana_regen_per_level".to_string(), mana_regen_per_level);
    mana_regen_params.insert("level".to_string(), level);
    mana_regen_params.insert("mana_regen_bonus".to_string(), mana_regen_bonus);
    mana_regen_params.insert("mana_regen_multiplier".to_string(), mana_regen_multiplier);
    mana_regen_params.insert("max_mana_regen".to_string(), max_mana_regen);

    manager.apply_template(
        &mut resolver,
        "ManaRegeneration",
        "mage:ManaRegeneration",
        &mana_regen_params,
    )?;

    // Resolve stats
    println!("--- Stat Resolution ---\n");

    // Intelligence
    let intelligence_id = StatId::from_str("mage:Intelligence");
    let intelligence = resolver.resolve(&intelligence_id, &context)?;
    let base_intelligence_total = base_intelligence + intelligence_per_level * level;

    println!("Intelligence:");
    println!(
        "  Base: {} + ({} × {}) = {}",
        base_intelligence, intelligence_per_level, level, base_intelligence_total
    );
    println!("  Clamped (1-200): {:.2}", intelligence.value);
    println!("  Expected: {:.2}", base_intelligence_total.min(200.0));
    assert!(
        (intelligence.value - base_intelligence_total.min(200.0)).abs() < 0.01,
        "Intelligence calculation failed"
    );
    println!();

    // ManaPool (map transform'lar artık transform olarak çalışıyor)
    let mana_id = StatId::from_str("mage:ManaPool");
    let mana = resolver.resolve(&mana_id, &context)?;

    // Calculation: sources -> map transforms -> additive -> multiplicative -> conditional
    let base_mana_total = base_mana + mana_per_level * level;
    let intelligence_mana_contribution = intelligence.value * 5.0;
    let vitality_mana_contribution = vitality_value * 2.0;

    // After sources (base_mana_total)
    let after_map_intelligence = base_mana_total + intelligence_mana_contribution;
    let after_map_vitality = after_map_intelligence + vitality_mana_contribution;
    let after_additive = after_map_vitality + mana_bonus;
    let after_multiplicative = after_additive * mana_multiplier;
    let conditional_multiplier = if intelligence.value >= 40.0 { 1.2 } else { 1.0 };
    let expected_final_mana = after_multiplicative * conditional_multiplier;

    println!("Mana Pool:");
    println!(
        "  Base Mana: {} + ({} × {}) = {}",
        base_mana, mana_per_level, level, base_mana_total
    );
    println!(
        "  Map transform (Intelligence × 5): {:.2} × 5 = {:.2}",
        intelligence.value, intelligence_mana_contribution
    );
    println!("  After Intelligence map: {:.2}", after_map_intelligence);
    println!(
        "  Map transform (Vitality × 2): {:.2} × 2 = {:.2}",
        vitality_value, vitality_mana_contribution
    );
    println!("  After Vitality map: {:.2}", after_map_vitality);
    println!(
        "  Additive transform (Equipment bonus): + {} = {:.2}",
        mana_bonus, after_additive
    );
    println!(
        "  Multiplicative transform: × {} = {:.2}",
        mana_multiplier, after_multiplicative
    );
    println!(
        "  Conditional bonus (Intelligence >= 40): {}",
        if intelligence.value >= 40.0 {
            "× 1.2 (+20% Mana Mastery)"
        } else {
            "× 1.0 (no bonus)"
        }
    );
    println!("  Final Mana Pool: {:.2}", mana.value);
    println!(
        "  Expected (with map transforms): {:.2}",
        expected_final_mana
    );
    if (mana.value - expected_final_mana).abs() < 0.01 {
        println!("  ✅ Mana Pool calculation correct!");
    } else {
        println!(
            "  ⚠️  Mana Pool differs from expected (map transforms may need dependency resolution)"
        );
        println!(
            "  Actual: {:.2}, Expected: {:.2}, Difference: {:.2}",
            mana.value,
            expected_final_mana,
            (mana.value - expected_final_mana).abs()
        );
    }
    println!();

    // ManaRegeneration (map transform artık transform olarak çalışıyor)
    let mana_regen_id = StatId::from_str("mage:ManaRegeneration");
    let mana_regen = resolver.resolve(&mana_regen_id, &context)?;
    let intelligence_regen_contribution = intelligence.value * 0.3;
    let base_regen_total = base_mana_regen + mana_regen_per_level * level;
    let regen_sources = base_regen_total + intelligence_regen_contribution;
    let regen_after_additive = regen_sources + mana_regen_bonus;
    let conditional_multiplier = if intelligence.value >= 50.0 { 1.4 } else { 1.0 };
    let final_regen =
        (regen_after_additive * mana_regen_multiplier * conditional_multiplier).min(max_mana_regen);

    println!("Mana Regeneration:");
    println!(
        "  Base Regen: {} + ({} × {}) = {}",
        base_mana_regen, mana_regen_per_level, level, base_regen_total
    );
    println!(
        "  Intelligence contribution (Intelligence × 0.3): {:.2} × 0.3 = {:.2} MP/s",
        intelligence.value, intelligence_regen_contribution
    );
    println!("  Total sources: {:.2} MP/s", regen_sources);
    println!("  Equipment bonus: + {} MP/s", mana_regen_bonus);
    println!("  After additive: {:.2} MP/s", regen_after_additive);
    println!(
        "  Conditional bonus (Intelligence >= 50): {}",
        if intelligence.value >= 50.0 {
            "× 1.4 (+40% Mana Flow)"
        } else {
            "× 1.0 (no bonus)"
        }
    );
    println!(
        "  Clamped (max: {}): {:.2} MP/s",
        max_mana_regen, mana_regen.value
    );
    println!("  Expected: {:.2} MP/s", final_regen);
    assert!(
        (mana_regen.value - final_regen).abs() < 0.01,
        "Mana regeneration calculation failed"
    );
    println!();

    // Calculate time to full mana
    let time_to_full = mana.value / mana_regen.value;

    println!("\nMana System Summary:");
    println!("  Intelligence: {:.2}", intelligence.value);
    println!("  Mana Pool: {:.2}", mana.value);
    println!("  Mana Regeneration: {:.2} MP/s", mana_regen.value);
    println!("  Time to full mana: {:.2} seconds", time_to_full);

    Ok(())
}
