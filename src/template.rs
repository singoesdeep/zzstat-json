use crate::config::{SourceConfig, StatConfig, StatTemplate, TransformConfig};
use crate::error::YamlStatError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zzstat::{StatId, StatResolver, StatSource, StatTransform};

/// Entity stat configuration (can be stored in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityStatConfig {
    /// Entity ID
    pub entity_id: String,
    /// Stat type (e.g., "HP", "ATK")
    pub stat_type: String,
    /// Template name to use
    pub template_name: String,
    /// Template parameters
    pub params: HashMap<String, f64>,
}

/// Entity parameters (loaded from database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityParams {
    /// Entity ID
    pub entity_id: String,
    /// Parameters (to be used in templates)
    pub params: HashMap<String, f64>,
}

/// Stat template manager - manages templates and entity-based stat management
pub struct StatTemplateManager {
    pub(crate) templates: HashMap<String, StatTemplate>,
    /// Entity stat configurations (for caching)
    entity_configs: HashMap<String, Vec<EntityStatConfig>>,
}

impl StatTemplateManager {
    /// Creates a template manager from JSON content.
    ///
    /// # Arguments
    ///
    /// * `json_content` - JSON string containing template definitions
    ///
    /// # Returns
    ///
    /// A `StatTemplateManager` instance.
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if JSON parsing fails.
    pub fn from_json(json_content: &str) -> Result<Self, YamlStatError> {
        let config: StatConfig = serde_json::from_str(json_content)?;
        Self::from_config(config)
    }

    /// Creates a template manager from StatConfig.
    ///
    /// # Arguments
    ///
    /// * `config` - StatConfig containing templates
    ///
    /// # Returns
    ///
    /// A `StatTemplateManager` instance.
    pub fn from_config(config: StatConfig) -> Result<Self, YamlStatError> {
        Ok(Self {
            templates: config.templates,
            entity_configs: HashMap::new(),
        })
    }

    /// Serializes templates to JSON format (for saving to database).
    ///
    /// # Returns
    ///
    /// JSON string containing templates.
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if serialization fails.
    pub fn templates_to_json(&self) -> Result<String, YamlStatError> {
        let config = StatConfig {
            templates: self.templates.clone(),
            stats: HashMap::new(),
        };
        serde_json::to_string(&config)
            .map_err(|e| YamlStatError::InvalidConfig(format!("JSON serialize error: {}", e)))
    }

    /// Creates a stat ID for an entity (in entity_id:stat_type format).
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity identifier
    /// * `stat_type` - Stat type name
    ///
    /// # Returns
    ///
    /// Formatted stat ID string.
    pub fn entity_stat_id(entity_id: &str, stat_type: &str) -> String {
        format!("{}:{}", entity_id, stat_type)
    }

    /// Converts entity stat ID to StatId.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity identifier
    /// * `stat_type` - Stat type name
    ///
    /// # Returns
    ///
    /// StatId instance.
    pub fn get_entity_stat_id(entity_id: &str, stat_type: &str) -> StatId {
        StatId::from_str(&Self::entity_stat_id(entity_id, stat_type))
    }

    /// Loads entity parameters from database and applies stats.
    ///
    /// # Arguments
    ///
    /// * `resolver` - StatResolver to register stats in
    /// * `entity_configs` - Vector of entity stat configurations
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if template is not found or parameter resolution fails.
    pub fn load_entity_stats(
        &mut self,
        resolver: &mut StatResolver,
        entity_configs: Vec<EntityStatConfig>,
    ) -> Result<(), YamlStatError> {
        for config in &entity_configs {
            let stat_id = Self::entity_stat_id(&config.entity_id, &config.stat_type);
            self.apply_template(resolver, &config.template_name, &stat_id, &config.params)?;
        }

        // Save to cache
        for config in entity_configs {
            self.entity_configs
                .entry(config.entity_id.clone())
                .or_default()
                .push(config);
        }

        Ok(())
    }

    /// Loads stats for a single entity.
    ///
    /// # Arguments
    ///
    /// * `resolver` - StatResolver to register stats in
    /// * `entity_id` - Entity identifier
    /// * `stat_configs` - Vector of stat configurations for this entity
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if template is not found or parameter resolution fails.
    pub fn load_entity(
        &self,
        resolver: &mut StatResolver,
        entity_id: &str,
        stat_configs: Vec<EntityStatConfig>,
    ) -> Result<(), YamlStatError> {
        for config in stat_configs {
            let stat_id = Self::entity_stat_id(entity_id, &config.stat_type);
            self.apply_template(resolver, &config.template_name, &stat_id, &config.params)?;
        }
        Ok(())
    }

    /// Resolves an entity stat.
    ///
    /// # Arguments
    ///
    /// * `resolver` - StatResolver containing the stat
    /// * `entity_id` - Entity identifier
    /// * `stat_type` - Stat type name
    /// * `context` - StatContext for resolution
    ///
    /// # Returns
    ///
    /// Resolved stat value.
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if stat resolution fails.
    pub fn resolve_entity_stat(
        &self,
        resolver: &mut StatResolver,
        entity_id: &str,
        stat_type: &str,
        context: &zzstat::StatContext,
    ) -> Result<zzstat::ResolvedStat, YamlStatError> {
        let stat_id = StatId::from_str(&Self::entity_stat_id(entity_id, stat_type));
        Ok(resolver.resolve(&stat_id, context)?)
    }

    /// Adds a source directly to the resolver (can be used for equipment, buffs, etc.).
    ///
    /// # Arguments
    ///
    /// * `resolver` - StatResolver to add source to
    /// * `entity_id` - Entity identifier
    /// * `stat_type` - Stat type name
    /// * `source` - Source to add
    pub fn add_source_to_entity(
        &self,
        resolver: &mut StatResolver,
        entity_id: &str,
        stat_type: &str,
        source: Box<dyn StatSource>,
    ) {
        let stat_id = Self::get_entity_stat_id(entity_id, stat_type);
        resolver.register_source(stat_id, source);
    }

    /// Adds a transform directly to the resolver (can be used for buffs, debuffs, etc.).
    ///
    /// # Arguments
    ///
    /// * `resolver` - StatResolver to add transform to
    /// * `entity_id` - Entity identifier
    /// * `stat_type` - Stat type name
    /// * `transform` - Transform to add
    pub fn add_transform_to_entity(
        &self,
        resolver: &mut StatResolver,
        entity_id: &str,
        stat_type: &str,
        transform: Box<dyn StatTransform>,
    ) {
        let stat_id = Self::get_entity_stat_id(entity_id, stat_type);
        resolver.register_transform(stat_id, transform);
    }

    /// Converts entity parameters to database format.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity identifier
    /// * `stat_mappings` - Vector of (stat_type, template_name, params) tuples
    ///
    /// # Returns
    ///
    /// Vector of EntityStatConfig instances.
    pub fn entity_params_to_configs(
        entity_id: &str,
        stat_mappings: &[(String, String, HashMap<String, f64>)],
    ) -> Vec<EntityStatConfig> {
        stat_mappings
            .iter()
            .map(|(stat_type, template_name, params)| EntityStatConfig {
                entity_id: entity_id.to_string(),
                stat_type: stat_type.clone(),
                template_name: template_name.clone(),
                params: params.clone(),
            })
            .collect()
    }

    /// Applies a template with parameters to a StatResolver.
    ///
    /// # Arguments
    ///
    /// * `resolver` - StatResolver to register stats in
    /// * `template_name` - Name of the template to apply
    /// * `stat_name` - Name for the stat (can be entity_id:stat_type format)
    /// * `params` - Parameters to substitute in the template
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if template is not found or parameter resolution fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zzstat_json::StatTemplateManager;
    /// use std::collections::HashMap;
    ///
    /// let json = r#"
    /// {
    ///   "templates": {
    ///     "BaseHP": {
    ///       "sources": [
    ///         {"type": "constant", "value": "{{base_hp}}"}
    ///       ]
    ///     }
    ///   }
    /// }
    /// "#;
    ///
    /// let manager = StatTemplateManager::from_json(json)?;
    /// let mut resolver = zzstat::StatResolver::new();
    /// let mut params = HashMap::new();
    /// params.insert("base_hp".to_string(), 100.0);
    ///
    /// manager.apply_template(&mut resolver, "BaseHP", "player1_HP", &params)?;
    /// # Ok::<(), zzstat_json::YamlStatError>(())
    /// ```
    pub fn apply_template(
        &self,
        resolver: &mut StatResolver,
        template_name: &str,
        stat_name: &str,
        params: &HashMap<String, f64>,
    ) -> Result<(), YamlStatError> {
        use zzstat::StatContext;

        let template = self.templates.get(template_name).ok_or_else(|| {
            YamlStatError::InvalidConfig(format!("Template not found: {}", template_name))
        })?;

        let stat_id = StatId::from_str(stat_name);

        // Extract entity ID from entity_id:stat_type format
        // If format is entity_id:stat_type, extract entity_id, otherwise empty string
        let entity_id = if let Some(colon_pos) = stat_name.find(':') {
            &stat_name[..colon_pos]
        } else {
            ""
        };

        let context = StatContext::new();

        // Add sources
        for source_config in &template.sources {
            let resolved_source =
                Self::resolve_source(source_config, params, resolver, entity_id, &context)?;
            resolver.register_source(stat_id.clone(), resolved_source);
        }

        // Add transformations
        for transform_config in &template.transforms {
            let resolved_transform =
                Self::resolve_transform_with_entity(transform_config, params, entity_id)?;
            resolver.register_transform(stat_id.clone(), resolved_transform);
        }

        Ok(())
    }

    /// Applies multiple stats at once.
    ///
    /// # Arguments
    ///
    /// * `resolver` - StatResolver to register stats in
    /// * `applications` - Vector of (template_name, stat_name, params) tuples
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if any template is not found or parameter resolution fails.
    pub fn apply_templates(
        &self,
        resolver: &mut StatResolver,
        applications: &[(String, String, HashMap<String, f64>)],
    ) -> Result<(), YamlStatError> {
        for (template_name, stat_name, params) in applications {
            self.apply_template(resolver, template_name, stat_name, params)?;
        }
        Ok(())
    }

    /// Resolves source configuration with parameters to create a StatSource.
    fn resolve_source(
        config: &SourceConfig,
        params: &HashMap<String, f64>,
        _resolver: &StatResolver,
        _entity_id: &str,
        _context: &zzstat::StatContext,
    ) -> Result<Box<dyn StatSource>, YamlStatError> {
        use zzstat::source::ConstantSource;

        match config {
            SourceConfig::Constant { value, name: _ } => {
                let resolved_value = value.resolve(params).map_err(|e| {
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
                let base_val = base.resolve(params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Base resolution error: {}", e))
                })?;
                let scale_val = scale.resolve(params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Scale resolution error: {}", e))
                })?;
                let level_val = level
                    .as_ref()
                    .map(|l| l.resolve(params))
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

    /// Resolves transform configuration with parameters to create a StatTransform (with entity_id).
    fn resolve_transform_with_entity(
        config: &TransformConfig,
        params: &HashMap<String, f64>,
        entity_id: &str,
    ) -> Result<Box<dyn StatTransform>, YamlStatError> {
        use crate::transform::AdditiveTransform;
        use zzstat::transform::{ClampTransform, MultiplicativeTransform};

        match config {
            TransformConfig::Multiplicative { value, name: _ } => {
                let resolved_value = value.resolve(params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Transform resolution error: {}", e))
                })?;
                Ok(Box::new(MultiplicativeTransform::new(resolved_value)))
            }

            TransformConfig::Additive { value, name: _ } => {
                let resolved_value = value.resolve(params).map_err(|e| {
                    YamlStatError::InvalidConfig(format!("Transform resolution error: {}", e))
                })?;
                Ok(Box::new(AdditiveTransform::new(resolved_value)))
            }

            TransformConfig::Clamp { min, max, name: _ } => {
                let min_val = min
                    .as_ref()
                    .map(|m| m.resolve(params))
                    .transpose()
                    .map_err(|e| {
                        YamlStatError::InvalidConfig(format!("Clamp min resolution error: {}", e))
                    })?
                    .unwrap_or(f64::NEG_INFINITY);
                let max_val = max
                    .as_ref()
                    .map(|m| m.resolve(params))
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
                ConditionalTransform::from_config(
                    condition_stat,
                    *condition_value,
                    operator,
                    then,
                    else_then,
                    params,
                    entity_id,
                )
                .map(|t| Box::new(t) as Box<dyn StatTransform>)
            }

            TransformConfig::Map {
                dependencies,
                multiplier,
                name: _,
            } => {
                use crate::transform_map::MapTransform;

                let mut dependency_ids = Vec::new();
                for dep_name in dependencies {
                    let dep_stat_id = if !entity_id.is_empty() {
                        // Entity-based: entity_id:stat_type format
                        StatId::from_str(&format!("{}:{}", entity_id, dep_name))
                    } else {
                        // Global stat
                        StatId::from_str(dep_name)
                    };
                    dependency_ids.push(dep_stat_id);
                }

                let multiplier_val = multiplier
                    .as_ref()
                    .map(|m| m.resolve(params))
                    .transpose()
                    .map_err(|e| {
                        YamlStatError::InvalidConfig(format!("Multiplier resolution error: {}", e))
                    })?
                    .unwrap_or(1.0);

                Ok(Box::new(MapTransform::new(dependency_ids, multiplier_val)))
            }
        }
    }

    /// Resolves transform configuration with parameters to create a StatTransform (public, without entity_id).
    pub(crate) fn resolve_transform(
        config: &TransformConfig,
        params: &HashMap<String, f64>,
    ) -> Result<Box<dyn StatTransform>, YamlStatError> {
        Self::resolve_transform_with_entity(config, params, "")
    }
}
