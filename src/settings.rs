use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{Parameter, RangeSpec, Sex};

/// User-edited reference ranges, keyed by `Parameter::id`. Layered on top of
/// the built-in defaults in `reference_data::PARAMETERS` rather than mutating
/// them, so "reset to default" is just "remove the override".
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RangeOverrides(pub HashMap<String, RangeSpec>);

impl RangeOverrides {
    fn file_path() -> Option<PathBuf> {
        let appdata = std::env::var_os("APPDATA")?;
        Some(PathBuf::from(appdata).join("BloodAnalyzer").join("range_overrides.json"))
    }

    pub fn load() -> Self {
        Self::file_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::file_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn set(&mut self, param_id: &str, range: RangeSpec) {
        self.0.insert(param_id.to_string(), range);
    }

    pub fn reset(&mut self, param_id: &str) {
        self.0.remove(param_id);
    }

    pub fn reset_all(&mut self) {
        self.0.clear();
    }

    pub fn get(&self, param_id: &str) -> Option<RangeSpec> {
        self.0.get(param_id).copied()
    }
}

/// Resolve the range actually in effect for a parameter: an override if one
/// exists, otherwise the built-in default (sex-resolved).
pub fn effective_range(param: &Parameter, sex: Sex, overrides: &RangeOverrides) -> (f64, f64) {
    match overrides.get(param.id) {
        Some(range) => range.resolve(sex),
        None => param.range.resolve(sex),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Panel;

    fn test_param() -> Parameter {
        Parameter {
            id: "test",
            name: "Test",
            unit: "unit",
            panel: Panel::Cmp,
            range: RangeSpec::Fixed { low: 1.0, high: 2.0 },
            critical: None,
            description: "",
            high_meaning: "",
            low_meaning: "",
            us_unit: "unit",
            si_to_us_factor: 1.0,
            source_url: "",
            suggestion_high: None,
            suggestion_low: None,
        }
    }

    #[test]
    fn falls_back_to_default_when_no_override() {
        let param = test_param();
        let overrides = RangeOverrides::default();
        assert_eq!(effective_range(&param, Sex::Male, &overrides), (1.0, 2.0));
    }

    #[test]
    fn override_takes_precedence() {
        let param = test_param();
        let mut overrides = RangeOverrides::default();
        overrides.set("test", RangeSpec::Fixed { low: 5.0, high: 9.0 });
        assert_eq!(effective_range(&param, Sex::Male, &overrides), (5.0, 9.0));
    }

    #[test]
    fn reset_removes_override() {
        let param = test_param();
        let mut overrides = RangeOverrides::default();
        overrides.set("test", RangeSpec::Fixed { low: 5.0, high: 9.0 });
        overrides.reset("test");
        assert_eq!(effective_range(&param, Sex::Male, &overrides), (1.0, 2.0));
    }

    #[test]
    fn serde_roundtrip() {
        let mut overrides = RangeOverrides::default();
        overrides.set("wbc", RangeSpec::Fixed { low: 3.5, high: 10.5 });
        overrides.set(
            "hemoglobin",
            RangeSpec::BySex { male: (130.0, 170.0), female: (115.0, 150.0) },
        );
        let json = serde_json::to_string(&overrides).unwrap();
        let restored: RangeOverrides = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get("wbc"), overrides.get("wbc"));
    }
}
