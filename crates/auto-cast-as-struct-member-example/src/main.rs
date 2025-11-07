use auto_cast_as_struct_member_example::{
    current_resistance::{AutoCastAsCurrentResistanceOhms, CurrentResistanceOhms},
    volume::{AutoCastAsVolumeCubicM, VolumeCubicM},
};

#[derive(Debug, Clone)]
struct Cylinder {
    radius_m: f64,
    length_m: f64,
}

impl VolumeCubicM for Cylinder {
    fn volume_cubic_m(&self) -> f64 {
        std::f64::consts::PI * self.radius_m * self.radius_m * self.length_m
    }
}

#[derive(Debug, Clone)]
struct Resistor {
    current_resistance_ohms: f64,
}

impl CurrentResistanceOhms for Resistor {
    fn current_resistance_ohms(&self) -> f64 {
        self.current_resistance_ohms
    }
}

#[derive(Debug, Clone)]
struct HeatingElement {
    geometry: Cylinder,
    resistance: Resistor,
}

impl AutoCastAsVolumeCubicM for HeatingElement {
    fn as_volume_cubic_m_ref(&self) -> &impl VolumeCubicM {
        &self.geometry
    }
}

impl AutoCastAsCurrentResistanceOhms for HeatingElement {
    fn as_current_resistance_ohms_ref(&self) -> &impl CurrentResistanceOhms {
        &self.resistance
    }
}

fn main() {
    let heating_element = HeatingElement {
        geometry: Cylinder {
            radius_m: 0.005,
            length_m: 0.02,
        },
        resistance: Resistor {
            current_resistance_ohms: 8.86,
        },
    };

    println!(
        "Heating element: volume: {} m³, resistance: {}Ω",
        heating_element.volume_cubic_m(),
        heating_element.current_resistance_ohms()
    );
}
