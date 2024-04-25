pub trait Configurable {
    type ConfigSetting;
    /// Builds a communication out of
    fn configure(&mut self, configuration_setting: Self::ConfigSetting);
}
