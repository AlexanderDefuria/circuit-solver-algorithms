use crate::util::PrettyString;
use serde::{Deserialize, Serialize};

/// Possible Component Types
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum Component {
    Compound(Simplification),
    Ground,
    Resistor,
    VoltageSrc,
    CurrentSrc,
    DependentVoltage,
    DependentCurrent,
    Switch,
    Inductor,
    Capacitor,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Simplification {
    None,
    Series,
    Parallel,
    Norton,
    Thevinin,
}

impl Component {
    pub(crate) fn basic_string(&self) -> String {
        match self {
            Component::Ground => "GND".to_string(),
            Component::Resistor => "R".to_string(),
            Component::VoltageSrc => "SRC(V)".to_string(),
            Component::CurrentSrc => "SRC(C)".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub(crate) fn unit_string(&self) -> String {
        match self {
            Component::Ground => "V".to_string(),
            Component::Resistor => "Ω".to_string(),
            Component::VoltageSrc => "V".to_string(),
            Component::CurrentSrc => "A".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub(crate) fn is_source(&self) -> bool {
        match self {
            Component::VoltageSrc => true,
            Component::CurrentSrc => true,
            _ => false,
        }
    }
}

impl PrettyString for Component {
    fn pretty_string(&self) -> String {
        match self {
            Component::Ground => "Ground".to_string(),
            Component::Resistor => "Resistor".to_string(),
            Component::VoltageSrc => "Voltage".to_string(),
            Component::CurrentSrc => "Current".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty_string() {
        assert_eq!(Component::Ground.pretty_string(), "Ground".to_string());
        assert_eq!(Component::Resistor.pretty_string(), "Resistor".to_string());
        assert_eq!(Component::VoltageSrc.pretty_string(), "Voltage".to_string());
        assert_eq!(Component::CurrentSrc.pretty_string(), "Current".to_string());
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", Component::Ground), "Ground".to_string());
        assert_eq!(format!("{:?}", Component::Resistor), "Resistor".to_string());
        assert_eq!(
            format!("{:?}", Component::VoltageSrc),
            "VoltageSrc".to_string()
        );
        assert_eq!(
            format!("{:?}", Component::CurrentSrc),
            "CurrentSrc".to_string()
        );
    }
}
