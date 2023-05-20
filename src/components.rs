use serde::{Deserialize, Serialize};
use serde_json::Result;

/// Possible Component Types
#[derive(Serialize, Deserialize)]
pub(crate) enum Component {
    Ground,
    Resistor,
    Voltage,
    Current,
    // DependentVoltage, DependentCurrent
    // Switch, Inductor, Capacitor,
}