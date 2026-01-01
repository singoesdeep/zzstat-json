//! Basic Stats Example
//!
//! This example demonstrates direct stat definitions (without templates).
//! It shows how to define stats directly in JSON and resolve them.
//!
//! Breakdown:
//! - HP: Multiple constant sources + scaling + multiplicative transform + clamp
//! - ATK: Constant + scaling + additive + multiplicative + clamp
//! - DEF: Constant + scaling + additive + clamp
//! - CriticalChance: Constant + scaling + additive + clamp

use std::fs;
use zzstat::{StatContext, StatId};
use zzstat_json::load_from_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Stats Example ===\n");

    // Load JSON configuration
    let json_path = format!("{}/examples/basic_stats.json", env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(&json_path)?;
    let mut resolver = load_from_json(&json)?;
    let context = StatContext::new();

    println!("Loaded basic_stats.json configuration\n");

    // Resolve HP stat
    println!("--- HP Stat Analysis ---");
    let hp_id = StatId::from_str("HP");
    let hp = resolver.resolve(&hp_id, &context)?;

    println!("HP Calculation:");
    println!("  Sources:");
    println!("    - Base HP: 100.0");
    println!("    - Equipment bonus: 50.0");
    println!("    - Level scaling (level 5, 10 per level): 50.0");
    println!("    Total sources: {}", 100.0 + 50.0 + 50.0);
    println!("  Transforms:");
    println!("    - Multiplicative (+20%): × 1.2");
    println!("    - Clamp (min: 50.0)");
    println!("  Final HP: {:.2}", hp.value);
    println!("  Expected: (100 + 50 + 50) × 1.2 = 240.0\n");
    assert!((hp.value - 240.0).abs() < 0.01, "HP calculation failed");

    // Resolve ATK stat
    println!("--- ATK Stat Analysis ---");
    let atk_id = StatId::from_str("ATK");
    let atk = resolver.resolve(&atk_id, &context)?;

    println!("ATK Calculation:");
    println!("  Sources:");
    println!("    - Base ATK: 25.0");
    println!("    - Level scaling (level 5, 3 per level): 15.0");
    println!("    Total sources: {}", 25.0 + 15.0);
    println!("  Transforms:");
    println!("    - Additive (+10 from weapon): + 10.0");
    println!("    - Multiplicative (+10%): × 1.1");
    println!("    - Clamp (max: 200.0)");
    println!("  Final ATK: {:.2}", atk.value);
    println!("  Expected: ((25 + 15) + 10) × 1.1 = 55.0\n");
    assert!((atk.value - 55.0).abs() < 0.01, "ATK calculation failed");

    // Resolve DEF stat
    println!("--- DEF Stat Analysis ---");
    let def_id = StatId::from_str("DEF");
    let def = resolver.resolve(&def_id, &context)?;

    println!("DEF Calculation:");
    println!("  Sources:");
    println!("    - Base DEF: 15.0");
    println!("    - Level scaling (level 5, 2 per level): 10.0");
    println!("    Total sources: {}", 15.0 + 10.0);
    println!("  Transforms:");
    println!("    - Additive (+5 from armor): + 5.0");
    println!("    - Clamp (min: 0.0)");
    println!("  Final DEF: {:.2}", def.value);
    println!("  Expected: (15 + 10) + 5 = 30.0\n");
    assert!((def.value - 30.0).abs() < 0.01, "DEF calculation failed");

    // Resolve CriticalChance stat
    println!("--- CriticalChance Stat Analysis ---");
    let crit_id = StatId::from_str("CriticalChance");
    let crit = resolver.resolve(&crit_id, &context)?;

    println!("CriticalChance Calculation:");
    println!("  Sources:");
    println!("    - Base critical chance: 5.0%");
    println!("    - Level scaling (level 5, 0.5% per level): 2.5%");
    println!("    Total sources: {}", 5.0 + 2.5);
    println!("  Transforms:");
    println!("    - Additive (+2% from equipment): + 2.0");
    println!("    - Clamp (0-100%)");
    println!("  Final CriticalChance: {:.2}%", crit.value);
    println!("  Expected: (5 + 2.5) + 2 = 9.5%\n");
    assert!(
        (crit.value - 9.5).abs() < 0.01,
        "CriticalChance calculation failed"
    );

    println!("✅ All stats resolved successfully!");
    println!("\nSummary:");
    println!("  HP: {:.2}", hp.value);
    println!("  ATK: {:.2}", atk.value);
    println!("  DEF: {:.2}", def.value);
    println!("  CriticalChance: {:.2}%", crit.value);

    Ok(())
}
