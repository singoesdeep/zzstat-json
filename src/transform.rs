use std::collections::HashMap;
use zzstat::{StatContext, StatError, StatId, StatTransform};

/// Additive transform - adds a constant value to the stat.
pub struct AdditiveTransform {
    value: f64,
}

impl AdditiveTransform {
    /// Creates a new AdditiveTransform.
    ///
    /// # Arguments
    ///
    /// * `value` - Value to add to the stat
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl StatTransform for AdditiveTransform {
    fn depends_on(&self) -> Vec<StatId> {
        Vec::new() // Additive transform doesn't depend on other stats
    }

    fn apply(
        &self,
        value: f64,
        _dependencies: &HashMap<StatId, f64>,
        _context: &StatContext,
    ) -> Result<f64, StatError> {
        Ok(value + self.value)
    }

    fn description(&self) -> String {
        format!("AdditiveTransform(+{})", self.value)
    }
}
