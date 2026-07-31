use crate::model::{AnalysisResult, Parameter, Sex, Status};

/// Classify a single entered value against a parameter's reference range
/// (resolved for the given sex) and any critical/panic thresholds.
pub fn analyze(parameter: &Parameter, value: f64, sex: Sex) -> AnalysisResult {
    let (low, high) = parameter.range.resolve(sex);
    let critical = parameter.critical.unwrap_or_default();

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

    AnalysisResult {
        parameter_id: parameter.id,
        value,
        status,
        range_low: low,
        range_high: high,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CriticalThresholds, Panel, RangeSpec};

    fn fixed_param(low: f64, high: f64, critical: Option<CriticalThresholds>) -> Parameter {
        Parameter {
            id: "test",
            name: "Test Param",
            unit: "unit",
            panel: Panel::Cmp,
            range: RangeSpec::Fixed { low, high },
            critical,
            description: "",
            high_meaning: "",
            low_meaning: "",
        }
    }

    #[test]
    fn exactly_at_bounds_is_normal() {
        let p = fixed_param(3.5, 5.0, None);
        assert_eq!(analyze(&p, 3.5, Sex::Male).status, Status::Normal);
        assert_eq!(analyze(&p, 5.0, Sex::Male).status, Status::Normal);
    }

    #[test]
    fn just_outside_bounds_is_low_or_high() {
        let p = fixed_param(3.5, 5.0, None);
        assert_eq!(analyze(&p, 3.49, Sex::Male).status, Status::Low);
        assert_eq!(analyze(&p, 5.01, Sex::Male).status, Status::High);
    }

    #[test]
    fn within_range_is_normal() {
        let p = fixed_param(3.5, 5.0, None);
        assert_eq!(analyze(&p, 4.2, Sex::Male).status, Status::Normal);
    }

    #[test]
    fn critical_thresholds_take_precedence() {
        let p = fixed_param(
            3.5,
            5.0,
            Some(CriticalThresholds { low: Some(2.5), high: Some(6.5) }),
        );
        assert_eq!(analyze(&p, 2.5, Sex::Male).status, Status::CriticalLow);
        assert_eq!(analyze(&p, 2.4, Sex::Male).status, Status::CriticalLow);
        assert_eq!(analyze(&p, 3.0, Sex::Male).status, Status::Low);
        assert_eq!(analyze(&p, 6.5, Sex::Male).status, Status::CriticalHigh);
        assert_eq!(analyze(&p, 6.6, Sex::Male).status, Status::CriticalHigh);
        assert_eq!(analyze(&p, 6.0, Sex::Male).status, Status::High);
    }

    #[test]
    fn by_sex_range_resolves_correctly() {
        let p = Parameter {
            id: "sex_test",
            name: "Sex Test",
            unit: "unit",
            panel: Panel::Cbc,
            range: RangeSpec::BySex { male: (135.0, 175.0), female: (120.0, 155.0) },
            critical: None,
            description: "",
            high_meaning: "",
            low_meaning: "",
        };
        assert_eq!(analyze(&p, 130.0, Sex::Male).status, Status::Low);
        assert_eq!(analyze(&p, 130.0, Sex::Female).status, Status::Normal);
    }

    #[test]
    fn no_lower_critical_never_flags_critical_low() {
        let p = fixed_param(0.0, 1.7, Some(CriticalThresholds { low: None, high: Some(11.3) }));
        assert_eq!(analyze(&p, -100.0, Sex::Male).status, Status::Low);
    }
}
