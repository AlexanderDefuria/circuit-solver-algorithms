use serde::{Deserialize, Serialize};

/// Possible Component Types
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) enum Component {
    Ground,
    Resistor,
    VoltageSrc,
    CurrentSrc,
    // DependentVoltage, DependentCurrent
    // Switch, Inductor, Capacitor,
}

impl Component {
    pub(crate) fn pretty_string(&self) -> String {
        match self {
            Component::Ground => "Ground".to_string(),
            Component::Resistor => "Resistor".to_string(),
            Component::VoltageSrc => "Voltage".to_string(),
            Component::CurrentSrc => "Current".to_string(),
        }
    }

    pub(crate) fn basic_string(&self) -> String {
        match self {
            Component::Ground => "GND".to_string(),
            Component::Resistor => "R".to_string(),
            Component::VoltageSrc => "V_src".to_string(),
            Component::CurrentSrc => "C_src".to_string(),
        }
    }

    pub(crate) fn unit_string(&self) -> String {
        match self {
            Component::Ground => "V".to_string(),
            Component::Resistor => "Ohm".to_string(),
            Component::VoltageSrc => "V".to_string(),
            Component::CurrentSrc => "A".to_string(),
        }
    }

    pub(crate) fn from_string(string: &str) -> Component {
        match string {
            "Ground" => Component::Ground,
            "Resistor" => Component::Resistor,
            "Voltage" => Component::VoltageSrc,
            "Current" => Component::CurrentSrc,
            _ => panic!("Invalid Component Type"),
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
