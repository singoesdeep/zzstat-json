use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON configuration structure for stat definitions and templates.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatConfig {
    /// Stat templates (reusable parameterized definitions)
    #[serde(default)]
    pub templates: HashMap<String, StatTemplate>,

    /// Direct stat definitions (for immediate use)
    #[serde(default)]
    pub stats: HashMap<String, StatDefinition>,
}

/// Stat template - parameterizable stat definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatTemplate {
    /// Template description
    #[serde(default)]
    pub description: Option<String>,

    /// Stat sources (additive)
    #[serde(default)]
    pub sources: Vec<SourceConfig>,

    /// Stat transformations
    #[serde(default)]
    pub transforms: Vec<TransformConfig>,
}

/// Single stat definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatDefinition {
    /// Stat sources (additive)
    #[serde(default)]
    pub sources: Vec<SourceConfig>,

    /// Stat transformations
    #[serde(default)]
    pub transforms: Vec<TransformConfig>,
}

/// Source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    /// Constant value source
    #[serde(rename = "constant")]
    Constant {
        /// Value (f64 or "{{param}}" string)
        value: SourceValue,
        /// Description (optional, for readability)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    /// Scaling source
    #[serde(rename = "scaling")]
    Scaling {
        /// Base value
        base: SourceValue,
        /// Scale factor
        scale: SourceValue,
        /// Level (optional, can be taken from context or parameter)
        level: Option<SourceValue>,
        /// Description (optional, for readability)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}

/// Source value - f64 or string (for parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourceValue {
    /// Numeric value
    Number(f64),
    /// String value (for parameters, e.g., "{{level}}")
    String(String),
}

impl SourceValue {
    /// Resolves the value to f64, replacing parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - HashMap of parameter names to values
    ///
    /// # Returns
    ///
    /// Resolved f64 value
    ///
    /// # Errors
    ///
    /// Returns error string if parameter is not found or string cannot be parsed as f64.
    pub fn resolve(&self, params: &HashMap<String, f64>) -> Result<f64, String> {
        match self {
            SourceValue::Number(n) => Ok(*n),
            SourceValue::String(s) => {
                // Resolve {{param}} syntax
                if s.starts_with("{{") && s.ends_with("}}") {
                    let param_name = s[2..s.len() - 2].trim();
                    params
                        .get(param_name)
                        .copied()
                        .ok_or_else(|| format!("Parameter not found: {}", param_name))
                } else {
                    // Parse string as f64
                    s.parse::<f64>()
                        .map_err(|_| format!("Invalid number: {}", s))
                }
            }
        }
    }
}

/// Transform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransformConfig {
    /// Multiplicative transformation
    #[serde(rename = "multiplicative")]
    Multiplicative {
        /// Multiplier value
        value: SourceValue,
        /// Description (optional, for readability)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    /// Additive transformation
    #[serde(rename = "additive")]
    Additive {
        /// Value to add
        value: SourceValue,
        /// Description (optional, for readability)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    /// Clamp transformation
    #[serde(rename = "clamp")]
    Clamp {
        /// Minimum value
        min: Option<SourceValue>,
        /// Maximum value
        max: Option<SourceValue>,
        /// Description (optional, for readability)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    /// Conditional transformation
    #[serde(rename = "conditional")]
    Conditional {
        /// Condition stat name
        condition_stat: String,
        /// Condition value
        condition_value: f64,
        /// Condition operator (>, <, >=, <=, ==)
        operator: String,
        /// Transform to apply when condition is met
        then: Box<TransformConfig>,
        /// Transform to apply when condition is not met (optional)
        else_then: Option<Box<TransformConfig>>,
    },

    /// Map transformation - adds values from dependent stats multiplied by a multiplier
    #[serde(rename = "map")]
    Map {
        /// Dependent stat names
        dependencies: Vec<String>,
        /// Multiplier to apply to the sum of dependent stat values
        /// Can be f64 or "{{param}}" string
        multiplier: Option<SourceValue>,
        /// Description (optional, for readability)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}
