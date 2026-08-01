use crate::model::{AnalysisResult, CriticalThresholds, Status};

/// Classify a single entered value against an already-resolved reference
/// range and optional critical/panic thresholds. Range resolution (sex,
/// user overrides) happens upstream of this function so the classification
/// logic itself stays a pure, easily-tested function.
pub fn analyze(value: f64, range: (f64, f64), critical: Option<CriticalThresholds>) -> AnalysisResult {
    let (low, high) = range;
    let critical = critical.unwrap_or_default();

    let status = if critical.low.is_some_and(|c| value <= c) {
        Status::CriticalLow
    } else if value < low {
        Status::Low
    } else if critical.high.is_some_and(|c| value >= c) {
        Status::CriticalHigh
    } else if value > high {
        Status::High
    } else {
        Status::Normal
    };

    AnalysisResult { value, status, range_low: low, range_high: high }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_at_bounds_is_normal() {
        assert_eq!(analyze(3.5, (3.5, 5.0), None).status, Status::Normal);
        assert_eq!(analyze(5.0, (3.5, 5.0), None).status, Status::Normal);
    }

    #[test]
    fn just_outside_bounds_is_low_or_high() {
        assert_eq!(analyze(3.49, (3.5, 5.0), None).status, Status::Low);
        assert_eq!(analyze(5.01, (3.5, 5.0), None).status, Status::High);
    }

    #[test]
    fn within_range_is_normal() {
        assert_eq!(analyze(4.2, (3.5, 5.0), None).status, Status::Normal);
    }

    #[test]
    fn critical_thresholds_take_precedence() {
        let critical = Some(CriticalThresholds { low: Some(2.5), high: Some(6.5) });
        assert_eq!(analyze(2.5, (3.5, 5.0), critical).status, Status::CriticalLow);
        assert_eq!(analyze(2.4, (3.5, 5.0), critical).status, Status::CriticalLow);
        assert_eq!(analyze(3.0, (3.5, 5.0), critical).status, Status::Low);
        assert_eq!(analyze(6.5, (3.5, 5.0), critical).status, Status::CriticalHigh);
        assert_eq!(analyze(6.6, (3.5, 5.0), critical).status, Status::CriticalHigh);
        assert_eq!(analyze(6.0, (3.5, 5.0), critical).status, Status::High);
    }

    #[test]
    fn no_lower_critical_never_flags_critical_low() {
        let critical = Some(CriticalThresholds { low: None, high: Some(11.3) });
        assert_eq!(analyze(-100.0, (0.0, 1.7), critical).status, Status::Low);
    }
}
