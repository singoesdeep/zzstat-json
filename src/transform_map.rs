use std::collections::HashMap;
use zzstat::{StatContext, StatError, StatId, StatTransform};

/// Map transform - adds values from dependent stats multiplied by a multiplier.
///
/// This transform depends on other stats. The values are retrieved from the resolver's cache
/// (via dependencies parameter) and summed, then multiplied by the multiplier, and added
/// to the current stat value.
pub struct MapTransform {
    dependencies: Vec<StatId>,
    multiplier: f64,
}

impl MapTransform {
    /// Creates a new MapTransform.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - Vector of stat IDs this transform depends on
    /// * `multiplier` - Multiplier to apply to the sum of dependent stat values
    pub fn new(dependencies: Vec<StatId>, multiplier: f64) -> Self {
        Self {
            dependencies,
            multiplier,
        }
    }
}

impl StatTransform for MapTransform {
    fn depends_on(&self) -> Vec<StatId> {
        // Return dependencies so zzstat's dependency graph can resolve them first
        self.dependencies.clone()
    }

    fn apply(
        &self,
        value: f64,
        dependencies: &HashMap<StatId, f64>,
        _context: &StatContext,
    ) -> Result<f64, StatError> {
        // Sum up values from dependent stats
        let mut sum = 0.0;
        for dep_id in &self.dependencies {
            let dep_value = dependencies
                .get(dep_id)
                .copied()
                .ok_or_else(|| StatError::MissingDependency(dep_id.clone()))?;
            sum += dep_value;
        }

        // Multiply by multiplier and add to current value
        Ok(value + (sum * self.multiplier))
    }

    fn description(&self) -> String {
        format!(
            "MapTransform(sum of {:?} × {})",
            self.dependencies, self.multiplier
        )
    }
}
