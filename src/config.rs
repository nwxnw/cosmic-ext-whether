use cosmic::cosmic_config;
use cosmic::cosmic_config::{
    cosmic_config_derive::CosmicConfigEntry, Config, ConfigGet, CosmicConfigEntry,
};

use crate::types::SavedLocation;

pub const APP_ID: &str = "com.github.nwxnw.cosmic-ext-whether";

/// Detect whether to default to Fahrenheit based on the user's locale.
///
/// Checks `LC_MEASUREMENT` then `LANG` for a country code.
/// US, Liberia (LR), and Myanmar (MM) use Fahrenheit; everyone else uses Celsius.
/// Falls back to `true` (Fahrenheit) if no locale can be determined.
fn detect_fahrenheit_default() -> bool {
    let locale_str = std::env::var("LC_MEASUREMENT")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("LANG").ok().filter(|s| !s.is_empty()));

    let Some(locale) = locale_str else {
        return true;
    };

    // Extract country code from e.g. "en_US.UTF-8" or "en_US"
    // Find the '_' separator, then take the next 2 chars as country code
    let country = locale
        .find('_')
        .and_then(|pos| locale.get(pos + 1..pos + 3))
        .map(|c| c.to_uppercase());

    match country.as_deref() {
        Some("US") | Some("LR") | Some("MM") => true,
        Some(_) => false,
        None => true, // Can't parse → preserve existing default
    }
}

#[derive(Debug, Clone, PartialEq, Eq, CosmicConfigEntry)]
#[version = 5]
pub struct WhetherConfig {
    pub use_fahrenheit: bool,
    pub locations: Vec<SavedLocation>,
    pub active_location_index: usize,
    pub refresh_interval_minutes: u32,
}

impl Default for WhetherConfig {
    fn default() -> Self {
        Self {
            use_fahrenheit: detect_fahrenheit_default(),
            locations: vec![],
            active_location_index: 0,
            refresh_interval_minutes: 30,
        }
    }
}

impl WhetherConfig {
    pub fn active_location(&self) -> Option<&SavedLocation> {
        self.locations.get(self.active_location_index)
    }
}

pub fn load_config() -> (WhetherConfig, Option<Config>) {
    // Try loading v5 config
    if let Ok(config) = Config::new(APP_ID, WhetherConfig::VERSION) {
        match WhetherConfig::get_entry(&config) {
            Ok(cfg) => return (cfg, Some(config)),
            Err((_, cfg)) => {
                // Partial load succeeded — new fields get defaults
                let _ = cfg.write_entry(&config);
                return (cfg, Some(config));
            }
        }
    }

    // v5 config doesn't exist - try migrating from v3
    // The only schema change from v4 is removal of `SavedLocation.source`
    // (dropped in the v0.3.0 weathervane migration). It lived inside the RON
    // `locations` blob, so serde simply ignores the now-unknown field and each
    // v4 location deserializes cleanly - no field transform
    if let Ok(v4_handle) = Config::new(APP_ID, 4) {
        if let Ok(cfg) = WhetherConfig::get_entry(&v4_handle) {
            if let Ok(v5_handle) = Config::new(APP_ID, WhetherConfig::VERSION) {
                let _ = cfg.write_entry(&v5_handle);
                return (cfg, Some(v5_handle));
            }
            return (cfg, None);
        }
    }

    // v4 config doesn't exist — try migrating from v3
    // SavedLocation's new `country_code` field has #[serde(default)] so v3 data
    // deserializes correctly (all locations get country_code: None).
    if let Ok(v3_handle) = Config::new(APP_ID, 3) {
        if let Ok(cfg) = WhetherConfig::get_entry(&v3_handle) {
            if let Ok(v4_handle) = Config::new(APP_ID, WhetherConfig::VERSION) {
                let _ = cfg.write_entry(&v4_handle);
                return (cfg, Some(v4_handle));
            }
            return (cfg, None);
        }
    }

    // Try migrating from v2
    if let Ok(v2_handle) = Config::new(APP_ID, 2) {
        if let Ok(cfg) = WhetherConfig::get_entry(&v2_handle) {
            if let Ok(v4_handle) = Config::new(APP_ID, WhetherConfig::VERSION) {
                let _ = cfg.write_entry(&v4_handle);
                return (cfg, Some(v4_handle));
            }
            return (cfg, None);
        }
    }

    (WhetherConfig::default(), None)
}

pub fn save_config(config_handle: &Option<Config>, cfg: &WhetherConfig) {
    if let Some(handle) = config_handle {
        let _ = cfg.write_entry(handle);
    }
}

pub fn detect_military_time() -> bool {
    if let Ok(cfg) = cosmic_config::Config::new("com.system76.CosmicAppletTime", 1) {
        if let Ok(v) = cfg.get::<bool>("military_time") {
            return v;
        }
    }
    detect_military_time_from_locale()
}

fn detect_military_time_from_locale() -> bool {
    let locale = std::env::var("LC_TIME")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("LANG").ok().filter(|s| !s.is_empty()));
    military_time_for_locale(locale.as_deref())
}

/// Country code after `_` in a locale string decides the clock.
/// `None` and anything without a parseable country keep the 12-hour default.
fn military_time_for_locale(locale: Option<&str>) -> bool {
    let Some(locale) = locale else {
        return false;
    };

    let country = locale
        .find('_')
        .and_then(|pos| locale.get(pos + 1..pos + 3))
        .map(|c| c.to_uppercase());

    match country.as_deref() {
        Some(
            "US" | "CA" | "AU" | "NZ" | "PH" | "IN" | "PK" | "BD" | "MY" | "EG" | "SA" | "JO"
            | "MX" | "CO",
        ) => false,
        Some(_) => true,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::military_time_for_locale;

    #[test]
    fn twelve_hour_countries_stay_on_twelve_hour() {
        for l in [
            "en_US.UTF-8",
            "en_CA",
            "en_AU.UTF-8",
            "es_MX",
            "en_IN",
            "es_CO",
        ] {
            assert!(!military_time_for_locale(Some(l)), "{l}");
        }
    }

    #[test]
    fn other_countries_use_twenty_four_hour() {
        for l in [
            "sv_SE.UTF-8",
            "en_GB",
            "pt_BR.UTF-8",
            "pl_PL",
            "de_DE@euro",
            "ja_JP.UTF-8",
        ] {
            assert!(military_time_for_locale(Some(l)), "{l}");
        }
    }

    #[test]
    fn country_code_is_case_insensitive() {
        assert!(!military_time_for_locale(Some("en_us.utf8")));
        assert!(military_time_for_locale(Some("sv_se")));
    }

    #[test]
    fn missing_or_unparseable_locale_keeps_the_default() {
        assert!(!military_time_for_locale(None));
        assert!(!military_time_for_locale(Some("C")));
        assert!(!military_time_for_locale(Some("POSIX")));
        assert!(!military_time_for_locale(Some("en")));
        assert!(!military_time_for_locale(Some("en_")));
    }
}
