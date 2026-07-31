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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

/// A reference range, either the same for everyone or split by sex.
#[derive(Debug, Clone, Copy)]
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
    pub parameter_id: &'static str,
    pub value: f64,
    pub status: Status,
    pub range_low: f64,
    pub range_high: f64,
}
