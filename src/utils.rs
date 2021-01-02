use crate::subprocess_control::SettingsMap;
use config::Config;

pub fn read_config() -> SettingsMap {
    let mut settings = Config::default();

    settings
        .merge(config::File::with_name("./bins/Info"))
        .expect("File exists");

    settings
        .try_into::<SettingsMap>()
        .expect("File format matches the try_into type parameter")
}
