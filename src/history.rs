use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Local};

use crate::model::Sex;

/// One saved, dated set of results. `values` are canonical SI floats keyed by
/// `Parameter::id`, independent of whatever unit system was active when saved.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedSession {
    pub timestamp: DateTime<Local>,
    pub sex: Sex,
    pub values: HashMap<String, f64>,
}

impl SavedSession {
    pub fn label(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M").to_string()
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct History(pub Vec<SavedSession>);

impl History {
    fn file_path() -> Option<PathBuf> {
        let appdata = std::env::var_os("APPDATA")?;
        Some(
            PathBuf::from(appdata)
                .join("BloodAnalyzer")
                .join("history.json"),
        )
    }

    pub fn load() -> Self {
        Self::file_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::file_path() else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Newest-first for display.
    pub fn sorted_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.0.len()).collect();
        indices.sort_by(|&a, &b| self.0[b].timestamp.cmp(&self.0[a].timestamp));
        indices
    }

    pub fn add(&mut self, session: SavedSession) {
        self.0.push(session);
        self.save();
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.0.len() {
            self.0.remove(index);
            self.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let mut values = HashMap::new();
        values.insert("wbc".to_string(), 7.5);
        values.insert("glucose".to_string(), 5.1);
        let session = SavedSession {
            timestamp: Local::now(),
            sex: Sex::Female,
            values,
        };
        let history = History(vec![session]);

        let json = serde_json::to_string(&history).unwrap();
        let restored: History = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.0.len(), 1);
        assert_eq!(restored.0[0].values.get("wbc"), Some(&7.5));
        assert_eq!(restored.0[0].sex, Sex::Female);
    }

    #[test]
    fn sorted_indices_newest_first() {
        let mk = |secs_ago: i64| SavedSession {
            timestamp: Local::now() - chrono::Duration::seconds(secs_ago),
            sex: Sex::Male,
            values: HashMap::new(),
        };
        let history = History(vec![mk(100), mk(0), mk(50)]);
        assert_eq!(history.sorted_indices(), vec![1, 2, 0]);
    }
}
