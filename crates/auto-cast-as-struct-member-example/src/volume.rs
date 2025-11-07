pub trait VolumeCubicM {
    fn volume_cubic_m(&self) -> f64;
}

pub trait AutoCastAsVolumeCubicM {
    fn as_volume_cubic_m_ref(&self) -> &impl VolumeCubicM;
}

impl<T: AutoCastAsVolumeCubicM> VolumeCubicM for T {
    fn volume_cubic_m(&self) -> f64 {
        self.as_volume_cubic_m_ref().volume_cubic_m()
    }
}
