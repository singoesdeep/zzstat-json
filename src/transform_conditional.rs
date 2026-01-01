use crate::config::TransformConfig;
use crate::error::YamlStatError;
use std::collections::HashMap;
use zzstat::{StatContext, StatError, StatId, StatTransform};

/// Conditional transform - applies different transforms based on a stat's value.
pub struct ConditionalTransform {
    condition_stat_id: StatId,
    condition_value: f64,
    operator: ConditionalOperator,
    then_transform: Box<dyn StatTransform>,
    else_transform: Option<Box<dyn StatTransform>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConditionalOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
}

impl ConditionalOperator {
    fn from_str(op: &str) -> Result<Self, String> {
        match op {
            ">" => Ok(Self::GreaterThan),
            "<" => Ok(Self::LessThan),
            ">=" => Ok(Self::GreaterThanOrEqual),
            "<=" => Ok(Self::LessThanOrEqual),
            "==" => Ok(Self::Equal),
            _ => Err(format!("Invalid operator: {}", op)),
        }
    }

    fn evaluate(&self, stat_value: f64, condition_value: f64) -> bool {
        match self {
            Self::GreaterThan => stat_value > condition_value,
            Self::LessThan => stat_value < condition_value,
            Self::GreaterThanOrEqual => stat_value >= condition_value,
            Self::LessThanOrEqual => stat_value <= condition_value,
            Self::Equal => (stat_value - condition_value).abs() < f64::EPSILON,
        }
    }
}

impl ConditionalTransform {
    /// Creates a new ConditionalTransform.
    ///
    /// # Arguments
    ///
    /// * `condition_stat_id` - Stat ID to check
    /// * `condition_value` - Value to compare against
    /// * `operator` - Comparison operator
    /// * `then_transform` - Transform to apply when condition is met
    /// * `else_transform` - Transform to apply when condition is not met (optional)
    pub(crate) fn new(
        condition_stat_id: StatId,
        condition_value: f64,
        operator: ConditionalOperator,
        then_transform: Box<dyn StatTransform>,
        else_transform: Option<Box<dyn StatTransform>>,
    ) -> Self {
        Self {
            condition_stat_id,
            condition_value,
            operator,
            then_transform,
            else_transform,
        }
    }

    /// Creates a ConditionalTransform from TransformConfig.
    ///
    /// # Arguments
    ///
    /// * `condition_stat` - Stat name to check
    /// * `condition_value` - Value to compare against
    /// * `operator` - Comparison operator string (">", "<", ">=", "<=", "==")
    /// * `then` - Transform config to apply when condition is met
    /// * `else_then` - Transform config to apply when condition is not met (optional)
    /// * `params` - Parameters for resolving transform configs
    /// * `entity_id` - Entity ID (empty string for global stats)
    ///
    /// # Returns
    ///
    /// A `ConditionalTransform` instance.
    ///
    /// # Errors
    ///
    /// Returns `YamlStatError` if operator is invalid or transform resolution fails.
    pub fn from_config(
        condition_stat: &str,
        condition_value: f64,
        operator: &str,
        then: &TransformConfig,
        else_then: &Option<Box<TransformConfig>>,
        params: &HashMap<String, f64>,
        entity_id: &str,
    ) -> Result<Self, YamlStatError> {
        use zzstat::StatId;

        // Create condition stat ID
        let condition_stat_id = if !entity_id.is_empty() {
            StatId::from_str(&format!("{}:{}", entity_id, condition_stat))
        } else {
            StatId::from_str(condition_stat)
        };

        // Parse operator
        let op = ConditionalOperator::from_str(operator)
            .map_err(|e| YamlStatError::InvalidConfig(format!("Operator error: {}", e)))?;

        // Create then transform
        let then_transform = crate::template::StatTemplateManager::resolve_transform(then, params)?;

        // Create else transform (if exists)
        let else_transform = else_then
            .as_ref()
            .map(|e| crate::template::StatTemplateManager::resolve_transform(e, params))
            .transpose()?;

        Ok(Self::new(
            condition_stat_id,
            condition_value,
            op,
            then_transform,
            else_transform,
        ))
    }
}

impl StatTransform for ConditionalTransform {
    fn depends_on(&self) -> Vec<StatId> {
        let mut deps = vec![self.condition_stat_id.clone()];

        // Add then transform's dependencies
        deps.extend(self.then_transform.depends_on());

        // Add else transform's dependencies (if exists)
        if let Some(ref else_transform) = self.else_transform {
            deps.extend(else_transform.depends_on());
        }

        deps
    }

    fn apply(
        &self,
        value: f64,
        dependencies: &HashMap<StatId, f64>,
        context: &StatContext,
    ) -> Result<f64, StatError> {
        // Get condition stat's value
        let condition_stat_value = dependencies
            .get(&self.condition_stat_id)
            .copied()
            .unwrap_or(0.0);

        // Evaluate condition
        let condition_met = self
            .operator
            .evaluate(condition_stat_value, self.condition_value);

        // Apply transform based on condition
        if condition_met {
            self.then_transform.apply(value, dependencies, context)
        } else if let Some(ref else_transform) = self.else_transform {
            else_transform.apply(value, dependencies, context)
        } else {
            // If no else transform, return value as-is
            Ok(value)
        }
    }

    fn description(&self) -> String {
        format!(
            "ConditionalTransform(if {} {} {} then apply else {:?})",
            self.condition_stat_id,
            match self.operator {
                ConditionalOperator::GreaterThan => ">",
                ConditionalOperator::LessThan => "<",
                ConditionalOperator::GreaterThanOrEqual => ">=",
                ConditionalOperator::LessThanOrEqual => "<=",
                ConditionalOperator::Equal => "==",
            },
            self.condition_value,
            self.else_transform.is_some()
        )
    }
}
