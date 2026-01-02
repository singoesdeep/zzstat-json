#![allow(deprecated)]

//! Archer Entity Example
//!
//! This example demonstrates a complete entity system with:
//! - Archer entity struct with all stats
//! - Experience gain system
//! - Health regeneration
//! - Mana pool and regeneration
//! - Movement speed
//! - Status effects (bleed, burn, poison)
//! - Elemental resistances
//! - Equipment system (weapon with strong_against, armor with vitality)
//!
//! Key Features:
//! - Complete entity-based stat management
//! - Equipment that modifies stats
//! - Multiple stat dependencies
//! - Real-world game entity example

use std::collections::HashMap;
use std::fs;
use zzstat::{StatContext, StatId, StatResolver};
use zzstat_json::StatTemplateManager;

/// Archer entity with all stats
struct Archer {
    name: String,
    level: f64,
    resolver: StatResolver,
}

/// Equipment item
struct Item {
    name: String,
    item_type: ItemType,
    stat_modifiers: HashMap<String, f64>, // stat_name -> bonus_value
}

enum ItemType {
    Weapon,
    Armor,
    #[allow(dead_code)]
    Accessory,
}

impl Archer {
    /// Creates a new Archer entity
    fn new(name: String, level: f64) -> Result<Self, Box<dyn std::error::Error>> {
        let mut resolver = StatResolver::new();

        // Load all template configurations
        let archer_json = fs::read_to_string(format!(
            "{}/examples/archer.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let experience_json = fs::read_to_string(format!(
            "{}/examples/experience_gain.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let health_regen_json = fs::read_to_string(format!(
            "{}/examples/health_regeneration.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let mana_pool_json = fs::read_to_string(format!(
            "{}/examples/mana_pool.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let movement_speed_json = fs::read_to_string(format!(
            "{}/examples/movement_speed.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let status_effects_json = fs::read_to_string(format!(
            "{}/examples/status_effects.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let resistance_json = fs::read_to_string(format!(
            "{}/examples/resistance.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;

        let archer_manager = StatTemplateManager::from_json(&archer_json)?;
        let experience_manager = StatTemplateManager::from_json(&experience_json)?;
        let health_regen_manager = StatTemplateManager::from_json(&health_regen_json)?;
        let mana_pool_manager = StatTemplateManager::from_json(&mana_pool_json)?;
        let movement_speed_manager = StatTemplateManager::from_json(&movement_speed_json)?;
        let status_effects_manager = StatTemplateManager::from_json(&status_effects_json)?;
        let resistance_manager = StatTemplateManager::from_json(&resistance_json)?;

        let entity_id = "archer";

        // Character base stats
        let base_dexterity = 40.0;
        let dexterity_per_level = 4.0;
        let base_vitality = 20.0;
        let vitality_per_level = 2.0;
        let base_hp = 120.0;
        let hp_per_level = 10.0;
        let base_atk = 35.0;
        let atk_per_level = 4.5;
        let base_accuracy = 75.0;
        let accuracy_bonus = 0.0;
        let base_intelligence = 15.0;
        let intelligence_per_level = 1.5;

        // Apply base stats first (no dependencies)
        let mut dexterity_params = HashMap::new();
        dexterity_params.insert("base_dexterity".to_string(), base_dexterity);
        dexterity_params.insert("dexterity_per_level".to_string(), dexterity_per_level);
        dexterity_params.insert("level".to_string(), level);
        archer_manager.apply_template(
            &mut resolver,
            "ArcherDexterity",
            &format!("{}:Dexterity", entity_id),
            &dexterity_params,
        )?;

        let mut vitality_params = HashMap::new();
        vitality_params.insert("base_vitality".to_string(), base_vitality);
        vitality_params.insert("vitality_per_level".to_string(), vitality_per_level);
        vitality_params.insert("level".to_string(), level);
        archer_manager.apply_template(
            &mut resolver,
            "ArcherVitality",
            &format!("{}:Vitality", entity_id),
            &vitality_params,
        )?;

        // Create Agility stat for movement speed (simple constant for now)
        use zzstat::source::ConstantSource;
        let agility_value = base_dexterity * 0.8; // Agility is 80% of Dexterity for archers
        let agility_id = StatId::from_str(&format!("{}:Agility", entity_id));
        resolver.register_source(agility_id.clone(), Box::new(ConstantSource(agility_value)));

        // Create Intelligence stat for mana pool
        let mut intelligence_params = HashMap::new();
        intelligence_params.insert("base_intelligence".to_string(), base_intelligence);
        intelligence_params.insert("intelligence_per_level".to_string(), intelligence_per_level);
        intelligence_params.insert("level".to_string(), level);
        mana_pool_manager.apply_template(
            &mut resolver,
            "Intelligence",
            &format!("{}:Intelligence", entity_id),
            &intelligence_params,
        )?;

        // Apply dependent stats
        let mut hp_params = HashMap::new();
        hp_params.insert("base_hp".to_string(), base_hp);
        hp_params.insert("hp_per_level".to_string(), hp_per_level);
        hp_params.insert("level".to_string(), level);
        archer_manager.apply_template(
            &mut resolver,
            "ArcherHP",
            &format!("{}:HP", entity_id),
            &hp_params,
        )?;

        let mut atk_params = HashMap::new();
        atk_params.insert("base_atk".to_string(), base_atk);
        atk_params.insert("atk_per_level".to_string(), atk_per_level);
        atk_params.insert("level".to_string(), level);
        atk_params.insert("atk_bonus".to_string(), 0.0);
        atk_params.insert("atk_multiplier".to_string(), 1.0);
        atk_params.insert("max_atk".to_string(), 1000.0);
        archer_manager.apply_template(
            &mut resolver,
            "ArcherATK",
            &format!("{}:ATK", entity_id),
            &atk_params,
        )?;

        let mut accuracy_params = HashMap::new();
        accuracy_params.insert("base_accuracy".to_string(), base_accuracy);
        accuracy_params.insert("accuracy_bonus".to_string(), accuracy_bonus);
        archer_manager.apply_template(
            &mut resolver,
            "ArcherAccuracy",
            &format!("{}:Accuracy", entity_id),
            &accuracy_params,
        )?;

        // Experience Gain
        let mut exp_params = HashMap::new();
        exp_params.insert("level".to_string(), level);
        exp_params.insert("exp_per_level".to_string(), 0.01); // +1% per level
        exp_params.insert("exp_bonus".to_string(), 0.0);
        exp_params.insert("exp_multiplier".to_string(), 1.0);
        exp_params.insert("then_multiplier".to_string(), 1.1); // +10% exp if level >= 10
        // Note: condition_stat and operator are now hardcoded in JSON template
        experience_manager.apply_template(
            &mut resolver,
            "ExperienceGain",
            &format!("{}:ExperienceGain", entity_id),
            &exp_params,
        )?;

        // Health Regeneration
        let mut hp_regen_params = HashMap::new();
        hp_regen_params.insert("base_hp_regen".to_string(), 1.0);
        hp_regen_params.insert("hp_regen_per_level".to_string(), 0.1);
        hp_regen_params.insert("level".to_string(), level);
        hp_regen_params.insert("hp_regen_bonus".to_string(), 0.0);
        hp_regen_params.insert("hp_regen_multiplier".to_string(), 1.0);
        hp_regen_params.insert("max_hp_regen".to_string(), 50.0);
        health_regen_manager.apply_template(
            &mut resolver,
            "HealthRegeneration",
            &format!("{}:HealthRegeneration", entity_id),
            &hp_regen_params,
        )?;

        // Mana Pool
        let mut mana_params = HashMap::new();
        mana_params.insert("base_mana".to_string(), 80.0);
        mana_params.insert("mana_per_level".to_string(), 5.0);
        mana_params.insert("level".to_string(), level);
        mana_params.insert("mana_bonus".to_string(), 0.0);
        mana_params.insert("mana_multiplier".to_string(), 1.0);
        mana_pool_manager.apply_template(
            &mut resolver,
            "ManaPool",
            &format!("{}:ManaPool", entity_id),
            &mana_params,
        )?;

        // Mana Regeneration
        let mut mana_regen_params = HashMap::new();
        mana_regen_params.insert("base_mana_regen".to_string(), 1.5);
        mana_regen_params.insert("mana_regen_per_level".to_string(), 0.2);
        mana_regen_params.insert("level".to_string(), level);
        mana_regen_params.insert("mana_regen_bonus".to_string(), 0.0);
        mana_regen_params.insert("mana_regen_multiplier".to_string(), 1.0);
        mana_regen_params.insert("max_mana_regen".to_string(), 30.0);
        mana_pool_manager.apply_template(
            &mut resolver,
            "ManaRegeneration",
            &format!("{}:ManaRegeneration", entity_id),
            &mana_regen_params,
        )?;

        // Movement Speed
        let mut speed_params = HashMap::new();
        speed_params.insert("base_movement_speed".to_string(), 5.0);
        speed_params.insert("speed_bonus".to_string(), 0.0);
        speed_params.insert("speed_multiplier".to_string(), 1.0);
        speed_params.insert("min_speed".to_string(), 1.0);
        speed_params.insert("max_speed".to_string(), 20.0);
        movement_speed_manager.apply_template(
            &mut resolver,
            "MovementSpeed",
            &format!("{}:MovementSpeed", entity_id),
            &speed_params,
        )?;

        // Status Effects
        let mut bleed_params = HashMap::new();
        bleed_params.insert("base_bleed_damage".to_string(), 2.0);
        bleed_params.insert("bleed_per_level".to_string(), 0.2);
        bleed_params.insert("level".to_string(), level);
        bleed_params.insert("bleed_multiplier".to_string(), 1.0);
        status_effects_manager.apply_template(
            &mut resolver,
            "BleedDamage",
            &format!("{}:BleedDamage", entity_id),
            &bleed_params,
        )?;

        let mut burn_params = HashMap::new();
        burn_params.insert("base_burn_damage".to_string(), 3.0);
        burn_params.insert("burn_per_level".to_string(), 0.3);
        burn_params.insert("level".to_string(), level);
        burn_params.insert("burn_multiplier".to_string(), 1.0);
        status_effects_manager.apply_template(
            &mut resolver,
            "BurnDamage",
            &format!("{}:BurnDamage", entity_id),
            &burn_params,
        )?;

        let mut poison_params = HashMap::new();
        poison_params.insert("base_poison_damage".to_string(), 2.5);
        poison_params.insert("poison_per_level".to_string(), 0.25);
        poison_params.insert("level".to_string(), level);
        poison_params.insert("poison_multiplier".to_string(), 1.0);
        status_effects_manager.apply_template(
            &mut resolver,
            "PoisonDamage",
            &format!("{}:PoisonDamage", entity_id),
            &poison_params,
        )?;

        // Elemental Resistances
        let mut fire_res_params = HashMap::new();
        fire_res_params.insert("base_fire_resistance".to_string(), 5.0);
        fire_res_params.insert("fire_res_per_level".to_string(), 0.3);
        fire_res_params.insert("level".to_string(), level);
        fire_res_params.insert("fire_res_bonus".to_string(), 0.0);
        resistance_manager.apply_template(
            &mut resolver,
            "FireResistance",
            &format!("{}:FireResistance", entity_id),
            &fire_res_params,
        )?;

        let mut ice_res_params = HashMap::new();
        ice_res_params.insert("base_ice_resistance".to_string(), 5.0);
        ice_res_params.insert("ice_res_per_level".to_string(), 0.3);
        ice_res_params.insert("level".to_string(), level);
        ice_res_params.insert("ice_res_bonus".to_string(), 0.0);
        resistance_manager.apply_template(
            &mut resolver,
            "IceResistance",
            &format!("{}:IceResistance", entity_id),
            &ice_res_params,
        )?;

        let mut lightning_res_params = HashMap::new();
        lightning_res_params.insert("base_lightning_resistance".to_string(), 5.0);
        lightning_res_params.insert("lightning_res_per_level".to_string(), 0.3);
        lightning_res_params.insert("level".to_string(), level);
        lightning_res_params.insert("lightning_res_bonus".to_string(), 0.0);
        resistance_manager.apply_template(
            &mut resolver,
            "LightningResistance",
            &format!("{}:LightningResistance", entity_id),
            &lightning_res_params,
        )?;

        let mut poison_res_params = HashMap::new();
        poison_res_params.insert("base_poison_resistance".to_string(), 10.0);
        poison_res_params.insert("poison_res_per_level".to_string(), 0.5);
        poison_res_params.insert("level".to_string(), level);
        poison_res_params.insert("poison_res_bonus".to_string(), 0.0);
        resistance_manager.apply_template(
            &mut resolver,
            "PoisonResistance",
            &format!("{}:PoisonResistance", entity_id),
            &poison_res_params,
        )?;

        Ok(Self {
            name,
            level,
            resolver,
        })
    }

    /// Equips an item, applying its stat modifiers
    fn equip_item(&mut self, item: &Item) -> Result<(), Box<dyn std::error::Error>> {
        let entity_id = "archer";

        match item.item_type {
            ItemType::Weapon => {
                // Apply weapon bonuses (ATK, etc.)
                if let Some(&atk_bonus) = item.stat_modifiers.get("ATK") {
                    use zzstat::transform::AdditiveTransform;
                    let atk_id = StatId::from_str(&format!("{}:ATK", entity_id));
                    self.resolver
                        .register_transform(atk_id, Box::new(AdditiveTransform::new(atk_bonus)));
                }

                // Apply strong_against bonuses (enemy_type -> bonus_value)
                for (key, &bonus_value) in &item.stat_modifiers {
                    if key.starts_with("StrongAgainst") {
                        let enemy_type = key.strip_prefix("StrongAgainst").unwrap_or(key);
                        self.apply_strong_against(enemy_type, bonus_value)?;
                    }
                }
            }
            ItemType::Armor => {
                // Apply armor bonuses (Defense, Vitality, etc.)
                if let Some(&vitality_bonus) = item.stat_modifiers.get("Vitality") {
                    use zzstat::transform::AdditiveTransform;
                    let vitality_id = StatId::from_str(&format!("{}:Vitality", entity_id));
                    self.resolver.register_transform(
                        vitality_id.clone(),
                        Box::new(AdditiveTransform::new(vitality_bonus)),
                    );

                    // Invalidate stats that depend on Vitality so they recalculate
                    let hp_id = StatId::from_str(&format!("{}:HP", entity_id));
                    self.resolver.invalidate(&hp_id);
                    let health_regen_id =
                        StatId::from_str(&format!("{}:HealthRegeneration", entity_id));
                    self.resolver.invalidate(&health_regen_id);
                    let fire_res_id = StatId::from_str(&format!("{}:FireResistance", entity_id));
                    self.resolver.invalidate(&fire_res_id);
                    let ice_res_id = StatId::from_str(&format!("{}:IceResistance", entity_id));
                    self.resolver.invalidate(&ice_res_id);
                    let lightning_res_id =
                        StatId::from_str(&format!("{}:LightningResistance", entity_id));
                    self.resolver.invalidate(&lightning_res_id);
                    let poison_res_id =
                        StatId::from_str(&format!("{}:PoisonResistance", entity_id));
                    self.resolver.invalidate(&poison_res_id);
                    let mana_pool_id = StatId::from_str(&format!("{}:ManaPool", entity_id));
                    self.resolver.invalidate(&mana_pool_id);
                }

                if let Some(&defense_bonus) = item.stat_modifiers.get("Defense") {
                    use zzstat::source::ConstantSource;
                    use zzstat::transform::AdditiveTransform;
                    let defense_id = StatId::from_str(&format!("{}:Defense", entity_id));
                    // Create Defense stat if it doesn't exist
                    if self.resolver.get_breakdown(&defense_id).is_none() {
                        self.resolver
                            .register_source(defense_id.clone(), Box::new(ConstantSource(0.0)));
                    }
                    self.resolver.register_transform(
                        defense_id,
                        Box::new(AdditiveTransform::new(defense_bonus)),
                    );
                }
            }
            ItemType::Accessory => {
                // Apply accessory bonuses
                for (stat_name, bonus) in &item.stat_modifiers {
                    use zzstat::transform::AdditiveTransform;
                    let stat_id = StatId::from_str(&format!("{}:{}", entity_id, stat_name));
                    self.resolver
                        .register_transform(stat_id, Box::new(AdditiveTransform::new(*bonus)));
                }
            }
        }

        Ok(())
    }

    /// Applies strong_against bonus for a specific enemy type
    fn apply_strong_against(
        &mut self,
        enemy_type: &str,
        bonus_value: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let strong_against_json = fs::read_to_string(format!(
            "{}/examples/strong_against.json",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let manager = StatTemplateManager::from_json(&strong_against_json)?;

        let stat_name = format!("archer:StrongAgainst{}", enemy_type);

        let mut params = HashMap::new();
        params.insert("bonus_value".to_string(), bonus_value);
        params.insert("multiplier".to_string(), 1.0);
        // Note: condition_stat and operator are strings in JSON template, not f64 params
        // They're resolved directly from JSON, not from params HashMap
        params.insert("condition_value".to_string(), 0.0);
        params.insert("then_multiplier".to_string(), 1.0);
        params.insert("min_value".to_string(), 0.0);
        params.insert("max_value".to_string(), 100.0);

        manager.apply_template(&mut self.resolver, "StrongAgainst", &stat_name, &params)?;
        Ok(())
    }

    /// Resolves a stat value
    fn get_stat(&mut self, stat_name: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let context = StatContext::new();
        let stat_id = StatId::from_str(&format!("archer:{}", stat_name));
        let resolved = self.resolver.resolve(&stat_id, &context)?;
        Ok(resolved.value)
    }

    /// Prints all stats
    fn print_stats(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("=== {} Stats (Level {}) ===", self.name, self.level);
        println!("  HP: {:.2}", self.get_stat("HP")?);
        println!("  ATK: {:.2}", self.get_stat("ATK")?);
        println!("  Dexterity: {:.2}", self.get_stat("Dexterity")?);
        println!("  Vitality: {:.2}", self.get_stat("Vitality")?);
        println!("  Accuracy: {:.2}%", self.get_stat("Accuracy")?);
        println!("  Intelligence: {:.2}", self.get_stat("Intelligence")?);
        println!(
            "  Experience Gain: {:.2}x",
            self.get_stat("ExperienceGain")?
        );
        println!(
            "  Health Regeneration: {:.2} HP/s",
            self.get_stat("HealthRegeneration")?
        );
        println!("  Mana Pool: {:.2}", self.get_stat("ManaPool")?);
        println!(
            "  Mana Regeneration: {:.2} MP/s",
            self.get_stat("ManaRegeneration")?
        );
        println!("  Movement Speed: {:.2}", self.get_stat("MovementSpeed")?);
        println!("  Bleed Damage: {:.2} DPS", self.get_stat("BleedDamage")?);
        println!("  Burn Damage: {:.2} DPS", self.get_stat("BurnDamage")?);
        println!("  Poison Damage: {:.2} DPS", self.get_stat("PoisonDamage")?);
        println!(
            "  Fire Resistance: {:.2}%",
            self.get_stat("FireResistance")?
        );
        println!("  Ice Resistance: {:.2}%", self.get_stat("IceResistance")?);
        println!(
            "  Lightning Resistance: {:.2}%",
            self.get_stat("LightningResistance")?
        );
        println!(
            "  Poison Resistance: {:.2}%",
            self.get_stat("PoisonResistance")?
        );

        // Show strong_against bonuses with breakdown
        let strong_against_types = ["Beast", "Undead", "Demon", "Dragon"];
        for enemy_type in &strong_against_types {
            let stat_id = StatId::from_str(&format!("archer:StrongAgainst{}", enemy_type));
            if let Some(breakdown) = self.resolver.get_breakdown(&stat_id) {
                println!("  Strong Against {}: {:.2}%", enemy_type, breakdown.value);
                // Show breakdown
                println!("    Breakdown:");
                for (source_name, source_value) in &breakdown.sources {
                    println!("      Source: {} = {:.2}", source_name, source_value);
                }
                for (transform_name, transform_value) in &breakdown.transforms {
                    println!(
                        "      Transform: {} → {:.2}",
                        transform_name, transform_value
                    );
                }
            }
        }
        println!();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Archer Entity Example ===\n");

    // Create archer entity
    let mut archer = Archer::new("Legolas".to_string(), 17.0)?; // Level 17 (+2 level)

    println!("Created Archer: {} (Level {})\n", archer.name, archer.level);
    println!("--- Base Stats (No Equipment) ---");
    archer.print_stats()?;

    // Create and equip weapon with strong_against bonuses
    let mut weapon = Item {
        name: "Elven Longbow".to_string(),
        item_type: ItemType::Weapon,
        stat_modifiers: HashMap::new(),
    };
    weapon.stat_modifiers.insert("ATK".to_string(), 25.0);
    weapon
        .stat_modifiers
        .insert("StrongAgainstBeast".to_string(), 15.0); // +15% damage against Beasts
    weapon
        .stat_modifiers
        .insert("StrongAgainstUndead".to_string(), 30.0); // +30% damage against Undead

    println!("--- Equipping Weapon: {} ---", weapon.name);
    archer.equip_item(&weapon)?;
    archer.print_stats()?;

    // Create and equip armor with vitality bonus
    let mut armor = Item {
        name: "Leather Armor".to_string(),
        item_type: ItemType::Armor,
        stat_modifiers: HashMap::new(),
    };
    armor.stat_modifiers.insert("Vitality".to_string(), 10.0); // +10 Vitality from armor
    armor.stat_modifiers.insert("Defense".to_string(), 15.0); // +15 Defense

    println!("--- Equipping Armor: {} ---", armor.name);
    archer.equip_item(&armor)?;
    archer.print_stats()?;

    // Show how vitality from armor affects other stats
    println!("--- Stat Dependencies Analysis ---");
    println!("Vitality from armor affects:");
    println!("  - HP (via map transform: Vitality × 3)");
    println!("  - Health Regeneration (via map transform: Vitality × 0.5 HP/s)");
    println!("  - Fire/Ice/Lightning Resistance (via map transform: Vitality × 0.2%)");
    println!("  - Poison Resistance (via map transform: Vitality × 0.3%)");
    println!();

    let base_hp = archer.get_stat("HP")?;
    let base_vitality = archer.get_stat("Vitality")?;
    println!("Current Stats:");
    println!(
        "  Vitality: {:.2} (includes +{} from armor)",
        base_vitality, 10.0
    );
    println!("  HP: {:.2} (affected by Vitality)", base_hp);
    println!(
        "  Health Regeneration: {:.2} HP/s (affected by Vitality)",
        archer.get_stat("HealthRegeneration")?
    );
    println!(
        "  Fire Resistance: {:.2}% (affected by Vitality)",
        archer.get_stat("FireResistance")?
    );

    println!("\n✅ Archer entity system working correctly!");
    println!("   All stats are properly linked through dependencies.");

    Ok(())
}
