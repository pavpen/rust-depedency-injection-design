pub trait CurrentResistanceOhms {
    fn current_resistance_ohms(&self) -> f64;
}

pub trait AutoCastAsCurrentResistanceOhms {
    fn as_current_resistance_ohms_ref(&self) -> &impl CurrentResistanceOhms;
}

impl<T: AutoCastAsCurrentResistanceOhms> CurrentResistanceOhms for T {
    fn current_resistance_ohms(&self) -> f64 {
        self.as_current_resistance_ohms_ref()
            .current_resistance_ohms()
    }
}
