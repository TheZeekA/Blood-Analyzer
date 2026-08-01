/// Format a value for read-only display: up to 2 decimal places, trailing
/// zeros trimmed.
pub fn format_display_value(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Panel {
    Cbc,
    Cmp,
    Lipid,
}

impl Panel {
    pub fn label(&self) -> &'static str {
        match self {
            Panel::Cbc => "Complete Blood Count (CBC)",
            Panel::Cmp => "Comprehensive Metabolic Panel (CMP)",
            Panel::Lipid => "Lipid Panel",
        }
    }

    pub const ALL: [Panel; 3] = [Panel::Cbc, Panel::Cmp, Panel::Lipid];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Sex {
    Male,
    Female,
}

/// Which unit convention values are displayed/entered in. Reference data and
/// analysis always operate in SI internally; this only affects the UI layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSystem {
    Si,
    UsConventional,
}

impl UnitSystem {
    pub fn label(&self) -> &'static str {
        match self {
            UnitSystem::Si => "SI",
            UnitSystem::UsConventional => "US",
        }
    }
}

/// A reference range, either the same for everyone or split by sex.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RangeSpec {
    Fixed { low: f64, high: f64 },
    BySex { male: (f64, f64), female: (f64, f64) },
}

impl RangeSpec {
    pub fn resolve(&self, sex: Sex) -> (f64, f64) {
        match self {
            RangeSpec::Fixed { low, high } => (*low, *high),
            RangeSpec::BySex { male, female } => match sex {
                Sex::Male => *male,
                Sex::Female => *female,
            },
        }
    }
}

/// Optional panic/critical-value thresholds, distinct from the normal reference range.
#[derive(Debug, Clone, Copy, Default)]
pub struct CriticalThresholds {
    pub low: Option<f64>,
    pub high: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub id: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
    pub panel: Panel,
    pub range: RangeSpec,
    pub critical: Option<CriticalThresholds>,
    pub description: &'static str,
    pub high_meaning: &'static str,
    pub low_meaning: &'static str,
    /// Unit label shown when displaying in US-conventional mode.
    pub us_unit: &'static str,
    /// Multiply an SI value by this to get the US-conventional value.
    pub si_to_us_factor: f64,
    /// MedlinePlus (or equivalent authoritative) page for this marker.
    pub source_url: &'static str,
    /// Short, non-prescriptive lifestyle note for an above-range result.
    pub suggestion_high: Option<&'static str>,
    /// Short, non-prescriptive lifestyle note for a below-range result.
    pub suggestion_low: Option<&'static str>,
}

impl Parameter {
    pub fn unit_for(&self, system: UnitSystem) -> &'static str {
        match system {
            UnitSystem::Si => self.unit,
            UnitSystem::UsConventional => self.us_unit,
        }
    }

    pub fn si_to_display(&self, si_value: f64, system: UnitSystem) -> f64 {
        match system {
            UnitSystem::Si => si_value,
            UnitSystem::UsConventional => si_value * self.si_to_us_factor,
        }
    }

    pub fn display_to_si(&self, display_value: f64, system: UnitSystem) -> f64 {
        match system {
            UnitSystem::Si => display_value,
            UnitSystem::UsConventional => display_value / self.si_to_us_factor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    CriticalLow,
    Low,
    Normal,
    High,
    CriticalHigh,
}

impl Status {
    pub fn label(&self) -> &'static str {
        match self {
            Status::CriticalLow => "Critically Low",
            Status::Low => "Low",
            Status::Normal => "Normal",
            Status::High => "High",
            Status::CriticalHigh => "Critically High",
        }
    }

    pub fn is_abnormal(&self) -> bool {
        !matches!(self, Status::Normal)
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub value: f64,
    pub status: Status,
    pub range_low: f64,
    pub range_high: f64,
}
