//! # zzstat-json
//!
//! JSON-based stat configuration for the `zzstat` crate. Define and manage your game stats
//! easily using JSON files.
//!
//! ## Features
//!
//! - JSON format for stat definitions
//! - **Stat templates** - Parameterizable templates for reusable stat definitions
//! - Multiple source types (constant, scaling)
//! - Transform support (multiplicative, additive, clamp, conditional, map)
//! - Dependency resolution (stats can depend on other stats)
//! - **Entity-based stat assignment** - Assign stats to entities using templates
//! - **Entity-based stat management** - JSON serialization, entity-based stat management
//! - **Flexible structure** - Equipment, buff features are left to the user
//! - Full integration with zzstat
//!
//! ## Example
//!
//! ```no_run
//! use zzstat_json::load_from_json;
//! use zzstat::{StatId, StatContext};
//!
//! let json = r#"
//! {
//!   "stats": {
//!     "HP": {
//!       "sources": [
//!         {"type": "constant", "value": 100.0},
//!         {"type": "constant", "value": 50.0}
//!       ],
//!       "transforms": [
//!         {"type": "multiplicative", "value": 1.5}
//!       ]
//!     }
//!   }
//! }
//! "#;
//!
//! let mut resolver = load_from_json(json)?;
//! let context = StatContext::new();
//! let hp_id = StatId::from_str("HP");
//! let resolved = resolver.resolve(&hp_id, &context)?;
//!
//! println!("HP: {}", resolved.value); // 225.0 = (100 + 50) * 1.5
//! # Ok::<(), zzstat_json::YamlStatError>(())
//! ```

pub mod config;
pub mod error;
pub mod loader;
pub mod template;
pub mod transform;
pub mod transform_conditional;
pub mod transform_map;

pub use config::StatConfig;
pub use error::YamlStatError;
pub use loader::StatLoader;
pub use template::{EntityParams, EntityStatConfig, StatTemplateManager};
pub use transform::AdditiveTransform;

use zzstat::{StatContext, StatId, StatResolver};

/// Creates a stat resolver from JSON content.
///
/// # Arguments
///
/// * `json_content` - JSON string containing stat definitions
///
/// # Returns
///
/// A `StatResolver` that can resolve the defined stats.
///
/// # Errors
///
/// Returns `YamlStatError` if JSON parsing fails or configuration is invalid.
///
/// # Example
///
/// ```no_run
/// use zzstat_json::load_from_json;
/// use zzstat::{StatId, StatContext};
///
/// let json = r#"
/// {
///   "stats": {
///     "HP": {
///       "sources": [
///         {"type": "constant", "value": 100.0}
///       ]
///     }
///   }
/// }
/// "#;
///
/// let resolver = load_from_json(json)?;
/// # Ok::<(), zzstat_json::YamlStatError>(())
/// ```
pub fn load_from_json(json_content: &str) -> Result<StatResolver, YamlStatError> {
    StatLoader::from_json(json_content)
}

/// Creates a stat resolver from JSON content and resolves a specific stat.
///
/// # Arguments
///
/// * `json_content` - JSON string containing stat definitions
/// * `stat_name` - Name of the stat to resolve
///
/// # Returns
///
/// The resolved stat value.
///
/// # Errors
///
/// Returns `YamlStatError` if JSON parsing fails, configuration is invalid, or stat resolution fails.
///
/// # Example
///
/// ```no_run
/// use zzstat_json::resolve_stat_from_json;
///
/// let json = r#"
/// {
///   "stats": {
///     "HP": {
///       "sources": [
///         {"type": "constant", "value": 100.0}
///       ]
///     }
///   }
/// }
/// "#;
///
/// let resolved = resolve_stat_from_json(json, "HP")?;
/// println!("HP: {}", resolved.value);
/// # Ok::<(), zzstat_json::YamlStatError>(())
/// ```
pub fn resolve_stat_from_json(
    json_content: &str,
    stat_name: &str,
) -> Result<zzstat::ResolvedStat, YamlStatError> {
    let mut resolver = load_from_json(json_content)?;
    let stat_id = StatId::from_str(stat_name);
    let context = StatContext::new();
    Ok(resolver.resolve(&stat_id, &context)?)
}

/// Creates entity stats from templates.
///
/// # Arguments
///
/// * `json_content` - JSON string containing template definitions
/// * `entity_name` - Name of the entity (will be used as stat name prefix)
/// * `template_name` - Name of the template to apply
/// * `params` - Parameters to substitute in the template
///
/// # Returns
///
/// A `StatResolver` with the applied template.
///
/// # Errors
///
/// Returns `YamlStatError` if JSON parsing fails, template is not found, or parameter resolution fails.
///
/// # Example
///
/// ```no_run
/// use zzstat_json::create_entity_stats;
/// use std::collections::HashMap;
///
/// let json = r#"
/// {
///   "templates": {
///     "BaseHP": {
///       "sources": [
///         {"type": "constant", "value": "{{base_hp}}"},
///         {"type": "scaling", "base": 0.0, "scale": 10.0, "level": "{{level}}"}
///       ]
///     }
///   }
/// }
/// "#;
///
/// let mut params = HashMap::new();
/// params.insert("base_hp".to_string(), 100.0);
/// params.insert("level".to_string(), 5.0);
///
/// let resolver = create_entity_stats(json, "player1", "BaseHP", &params)?;
/// # Ok::<(), zzstat_json::YamlStatError>(())
/// ```
pub fn create_entity_stats(
    json_content: &str,
    entity_name: &str,
    template_name: &str,
    params: &std::collections::HashMap<String, f64>,
) -> Result<StatResolver, YamlStatError> {
    let manager = StatTemplateManager::from_json(json_content)?;
    let mut resolver = StatResolver::new();
    manager.apply_template(&mut resolver, template_name, entity_name, params)?;
    Ok(resolver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_json_loading() {
        let json = r#"
{
  "stats": {
    "HP": {
      "sources": [
        {"type": "constant", "value": 100.0},
        {"type": "constant", "value": 50.0}
      ],
      "transforms": [
        {"type": "multiplicative", "value": 1.5}
      ]
    }
  }
}
"#;
        let mut resolver = load_from_json(json).unwrap();
        let context = StatContext::new();
        let hp_id = StatId::from_str("HP");
        let resolved = resolver.resolve(&hp_id, &context).unwrap();
        assert_eq!(resolved.value, 225.0); // (100 + 50) * 1.5
    }

    #[test]
    fn test_additive_transform() {
        let json = r#"
{
  "stats": {
    "ATK": {
      "sources": [
        {"type": "constant", "value": 50.0}
      ],
      "transforms": [
        {"type": "additive", "value": 10.0}
      ]
    }
  }
}
"#;
        let mut resolver = load_from_json(json).unwrap();
        let context = StatContext::new();
        let atk_id = StatId::from_str("ATK");
        let resolved = resolver.resolve(&atk_id, &context).unwrap();
        assert_eq!(resolved.value, 60.0); // 50 + 10
    }

    #[test]
    fn test_template_system() {
        use StatTemplateManager;
        use std::collections::HashMap;

        let json = r#"
{
  "templates": {
    "BaseHP": {
      "sources": [
        {"type": "constant", "value": "{{base_hp}}"},
        {"type": "scaling", "base": 0.0, "scale": "{{hp_per_level}}", "level": "{{level}}"}
      ],
      "transforms": [
        {"type": "multiplicative", "value": "{{multiplier}}"}
      ]
    }
  }
}
"#;
        let manager = StatTemplateManager::from_json(json).unwrap();
        let mut resolver = StatResolver::new();

        let mut params = HashMap::new();
        params.insert("base_hp".to_string(), 100.0);
        params.insert("hp_per_level".to_string(), 10.0);
        params.insert("level".to_string(), 5.0);
        params.insert("multiplier".to_string(), 1.5);

        manager
            .apply_template(&mut resolver, "BaseHP", "player1_HP", &params)
            .unwrap();

        let context = StatContext::new();
        let hp_id = StatId::from_str("player1_HP");
        let resolved = resolver.resolve(&hp_id, &context).unwrap();

        // (100 + 10*5) * 1.5 = 150 * 1.5 = 225
        assert_eq!(resolved.value, 225.0);
    }
}
