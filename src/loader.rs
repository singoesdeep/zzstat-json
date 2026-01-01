use crate::config::{SourceConfig, StatConfig, TransformConfig};
use crate::error::YamlStatError;
use crate::transform::AdditiveTransform;
use std::collections::HashMap;
use zzstat::{
    StatId, StatResolver, StatSource, StatTransform,
    source::ConstantSource,
    transform::{ClampTransform, MultiplicativeTransform},
};

/// Loader that creates stat resolvers from JSON.
pub struct StatLoader;

impl StatLoader {
    /// Creates a StatResolver from JSON content.
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
    pub fn from_json(json_content: &str) -> Result<StatResolver, YamlStatError> {
        let config: StatConfig = serde_json::from_str(json_content)?;
        Self::build_resolver(config)
    }

    /// Builds a resolver from configuration.
    fn build_resolver(config: StatConfig) -> Result<StatResolver, YamlStatError> {
        let mut resolver = StatResolver::new();

        // First, create all stat IDs
        let stat_ids: HashMap<String, StatId> = config
            .stats
            .keys()
            .map(|name| (name.clone(), StatId::from_str(name)))
            .collect();

        // Register sources
        for (stat_name, definition) in &config.stats {
            let stat_id = stat_ids.get(stat_name).ok_or_else(|| {
                YamlStatError::InvalidConfig(format!("Stat not found: {}", stat_name))
            })?;

            for source_config in &definition.sources {
                let source = Self::build_source(source_config, &stat_ids)?;
                resolver.register_source(stat_id.clone(), source);
            }
        }

        // Register transformations
        for (stat_name, definition) in &config.stats {
            let stat_id = stat_ids.get(stat_name).ok_or_else(|| {
                YamlStatError::InvalidConfig(format!("Stat not found: {}", stat_name))
            })?;

            for transform_config in &definition.transforms {
                let transform = Self::build_transform(transform_config, &stat_ids)?;
                resolver.register_transform(stat_id.clone(), transform);
            }
        }

        Ok(resolver)
    }

    /// Creates a StatSource from source configuration.
    fn build_source(
        config: &SourceConfig,
        _stat_ids: &HashMap<String, StatId>,
    ) -> Result<Box<dyn StatSource>, YamlStatError> {
        let empty_params = HashMap::new();

        match config {
            SourceConfig::Constant { value, name: _ } => {
                let resolved_value = value.resolve(&empty_params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Source resolution error: {}", e))
                })?;
                Ok(Box::new(ConstantSource(resolved_value)))
            }

            SourceConfig::Scaling {
                base,
                scale,
                level,
                name: _,
            } => {
                let base_val = base.resolve(&empty_params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Base resolution error: {}", e))
                })?;
                let scale_val = scale.resolve(&empty_params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Scale resolution error: {}", e))
                })?;
                let level_val = level
                    .as_ref()
                    .map(|l| l.resolve(&empty_params))
                    .transpose()
                    .map_err(|e| {
                        YamlStatError::InvalidConfig(format!("Level resolution error: {}", e))
                    })?
                    .unwrap_or(1.0);

                let value = base_val + (scale_val * level_val);
                Ok(Box::new(ConstantSource(value)))
            }
        }
    }

    /// Creates a StatTransform from transform configuration.
    fn build_transform(
        config: &TransformConfig,
        _stat_ids: &HashMap<String, StatId>,
    ) -> Result<Box<dyn StatTransform>, YamlStatError> {
        match config {
            TransformConfig::Multiplicative { value, name: _ } => {
                let empty_params = HashMap::new();
                let resolved_value = value.resolve(&empty_params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Transform resolution error: {}", e))
                })?;
                Ok(Box::new(MultiplicativeTransform::new(resolved_value)))
            }

            TransformConfig::Additive { value, name: _ } => {
                let empty_params = HashMap::new();
                let resolved_value = value.resolve(&empty_params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Transform resolution error: {}", e))
                })?;
                Ok(Box::new(AdditiveTransform::new(resolved_value)))
            }

            TransformConfig::Clamp { min, max, name: _ } => {
                let empty_params = HashMap::new();
                let min_val = min
                    .as_ref()
                    .map(|m| m.resolve(&empty_params))
                    .transpose()
                    .map_err(|e| {
                        YamlStatError::InvalidConfig(format!("Clamp min resolution error: {}", e))
                    })?
                    .unwrap_or(f64::NEG_INFINITY);
                let max_val = max
                    .as_ref()
                    .map(|m| m.resolve(&empty_params))
                    .transpose()
                    .map_err(|e| {
                        YamlStatError::InvalidConfig(format!("Clamp max resolution error: {}", e))
                    })?
                    .unwrap_or(f64::INFINITY);
                Ok(Box::new(ClampTransform::new(min_val, max_val)))
            }

            TransformConfig::Conditional {
                condition_stat,
                condition_value,
                operator,
                then,
                else_then,
            } => {
                use crate::transform_conditional::ConditionalTransform;
                // Empty string for entity_id for global stats
                let empty_params = HashMap::new();
                ConditionalTransform::from_config(
                    condition_stat,
                    *condition_value,
                    operator,
                    then,
                    else_then,
                    &empty_params,
                    "", // Empty string for global stats
                )
                .map(|t| Box::new(t) as Box<dyn StatTransform>)
            }

            TransformConfig::Map {
                dependencies,
                multiplier,
                name: _,
            } => {
                use crate::transform_map::MapTransform;
                let empty_params = HashMap::new();

                let mut dependency_ids = Vec::new();
                for dep_name in dependencies {
                    dependency_ids.push(StatId::from_str(dep_name));
                }

                let multiplier_val = multiplier
                    .as_ref()
                    .map(|m| m.resolve(&empty_params))
                    .transpose()
                    .map_err(|e| {
                        YamlStatError::InvalidConfig(format!("Multiplier resolution error: {}", e))
                    })?
                    .unwrap_or(1.0);

                Ok(Box::new(MapTransform::new(dependency_ids, multiplier_val)))
            }
        }
    }
}
