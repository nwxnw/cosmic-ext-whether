use crate::fl;
use serde::{Deserialize, Serialize};

// --- Weather alert types ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Extreme,
    Severe,
    Moderate,
    Minor,
    Unknown,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WeatherAlert {
    pub event: String,
    pub headline: String,
    pub severity: AlertSeverity,
}

/// Unified weather result from either NWS or Open-Meteo.
#[derive(Debug, Clone)]
pub struct WeatherResult {
    pub forecast: Forecast,
    pub cached_grid: Option<GridInfo>,
    pub alerts: Vec<WeatherAlert>,
    pub observation: Option<CurrentObservation>,
    pub air_quality: Option<AirQuality>,
}

/// Real-time observation data from the nearest station (NWS) or current block (Open-Meteo).
#[derive(Debug, Clone)]
pub struct CurrentObservation {
    pub temperature: Option<i32>,
    pub temperature_unit: String,
    pub condition: Option<String>,
    pub wind_speed: Option<String>,
    pub wind_direction: Option<String>,
    pub humidity: Option<i32>,
    pub is_daytime: bool,
    pub feels_like: Option<i32>,
    pub dew_point: Option<i32>,
    pub uv_index: Option<f32>,
    pub pressure: Option<f32>, //hPa; PressureUnit conversion is T5
    pub cloud_cover: Option<i32>,
    pub wind_gusts: Option<String>, // formatted like wind_speed
    pub visibility: Option<f32>,    // meters (raw); convert in T4
}

/// AirQuality struct mapped from weathervane's AirQualityData
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AirQuality {
    pub aqi: i32,
    pub category: weathervane::AqiCategory,
    pub pm2_5: f32,
    pub pm10: f32,
    pub ozone: f32,
    pub no2: f32,
    pub co: f32,
    pub severity: u8, // 0..=5, best-to-worst; drives the chip tint
}

// --- NWS API response types ---

#[derive(Debug, Clone, Deserialize)]
pub struct PointsResponse {
    pub properties: PointsProperties,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointsProperties {
    pub grid_id: String,
    pub grid_x: u32,
    pub grid_y: u32,
    pub relative_location: Option<RelativeLocation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelativeLocation {
    pub properties: RelativeLocationProperties,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelativeLocationProperties {
    pub city: String,
    pub state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastResponse {
    pub properties: ForecastProperties,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastProperties {
    pub periods: Vec<ForecastPeriod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ForecastPeriod {
    pub name: String,
    pub temperature: i32,
    pub temperature_unit: String,
    pub wind_speed: String,
    pub wind_direction: String,
    pub short_forecast: String,
    pub detailed_forecast: String,
    pub is_daytime: bool,
    pub probability_of_precipitation: Option<PrecipValue>,
    #[serde(default)]
    pub start_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrecipValue {
    pub value: Option<f64>,
}

// --- App domain types ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedLocation {
    pub name: String,
    pub lat: String,
    pub lon: String,
    #[serde(default)]
    pub cached_grid: Option<GridInfo>,
    #[serde(default)]
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridInfo {
    pub office: String,
    pub grid_x: u32,
    pub grid_y: u32,
    #[serde(default)]
    pub nearest_station: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DailySun {
    pub date: String,    // "2026-07-04"
    pub sunrise: String, // local naive ISO
    pub sunset: String,
}
#[derive(Debug, Clone)]
pub struct Forecast {
    pub location_name: String,
    pub periods: Vec<ForecastPeriod>,
    pub hourly_periods: Vec<ForecastPeriod>,
    pub sun_times: Vec<DailySun>,
}

#[derive(Debug, Clone)]
pub enum FetchState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

// --- View helper types ---

#[derive(Debug, Clone)]
pub struct DaySummary {
    pub name: String,
    pub date: Option<String>,
    pub high: Option<i32>,
    pub low: Option<i32>,
    pub short_forecast: String,
    pub is_daytime: bool,
    pub wind_speed: String,
    pub wind_direction: String,
    pub precip_chance: Option<i32>,
    pub detailed_forecast: String,
}

pub fn pair_daily_periods(periods: &[ForecastPeriod]) -> Vec<DaySummary> {
    let mut summaries = Vec::new();
    let mut i = 0;

    while i < periods.len() {
        let period = &periods[i];

        if period.is_daytime {
            // Day period — look ahead for matching night
            let low = periods.get(i + 1).and_then(|night| {
                if !night.is_daytime {
                    Some(night.temperature)
                } else {
                    None
                }
            });
            summaries.push(DaySummary {
                name: period.name.clone(),
                date: period
                    .start_time
                    .as_ref()
                    .and_then(|s| s.get(..10).map(str::to_string)),
                high: Some(period.temperature),
                low,
                short_forecast: period.short_forecast.clone(),
                is_daytime: true,
                wind_speed: period.wind_speed.clone(),
                wind_direction: period.wind_direction.clone(),
                precip_chance: period
                    .probability_of_precipitation
                    .as_ref()
                    .and_then(|p| p.value)
                    .map(|v| v as i32),
                detailed_forecast: period.detailed_forecast.clone(),
            });
            i += if low.is_some() { 2 } else { 1 };
        } else {
            // Night-only period (e.g., first period is tonight)
            summaries.push(DaySummary {
                name: period.name.clone(),
                date: period
                    .start_time
                    .as_ref()
                    .and_then(|s| s.get(..10).map(str::to_string)),
                high: None,
                low: Some(period.temperature),
                short_forecast: period.short_forecast.clone(),
                is_daytime: false,
                wind_speed: period.wind_speed.clone(),
                wind_direction: period.wind_direction.clone(),
                precip_chance: period
                    .probability_of_precipitation
                    .as_ref()
                    .and_then(|p| p.value)
                    .map(|v| v as i32),
                detailed_forecast: period.detailed_forecast.clone(),
            });
            i += 1;
        }
    }

    summaries
}

/// Extract a display hour like "3 PM" from an ISO 8601 string.
///
/// Input format: "2026-02-28T14:00:00-08:00"
/// The 'T' separator is at index 10, hour digits are at 11..13.
pub fn format_hour(start_time: &str) -> String {
    if let Some(t_pos) = start_time.find('T') {
        if let Ok(hour) = start_time[t_pos + 1..t_pos + 3].parse::<u32>() {
            // Locales set `clock-24h = true` to use a 24-hour clock ("14h");
            // the default (en) keeps the 12-hour AM/PM clock.
            if fl!("clock-24h") == "true" {
                return format!("{hour}h");
            }
            return match hour {
                0 => "12 AM".to_string(),
                1..=11 => format!("{hour} AM"),
                12 => "12 PM".to_string(),
                _ => format!("{} PM", hour - 12),
            };
        }
    }
    start_time.to_string()
}

// --- Geocoding types ---

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub display_name: String,
    pub lat: String,
    pub lon: String,
    #[serde(default)]
    pub address: Option<NominatimAddress>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NominatimAddress {
    #[serde(default)]
    pub country_code: Option<String>,
}

/// Map a weather condition string and day/night flag to a symbolic icon name.
pub fn condition_icon(condition: &str, is_daytime: bool) -> &'static str {
    let s = condition.to_lowercase();
    let night = !is_daytime;

    if s.contains("thunder") || s.contains("storm") {
        return "weather-storm-symbolic";
    }
    if s.contains("snow") || s.contains("flurr") || s.contains("blizzard") {
        return "weather-snow-symbolic";
    }
    if s.contains("rain") || s.contains("shower") || s.contains("drizzle") {
        return if s.contains("scattered") {
            "weather-showers-scattered-symbolic"
        } else {
            "weather-showers-symbolic"
        };
    }
    if s.contains("fog") || s.contains("mist") || s.contains("haz") {
        return "weather-fog-symbolic";
    }
    if s.contains("mostly cloudy") || s.contains("overcast") {
        return "weather-overcast-symbolic";
    }
    if s.contains("partly") && s.contains("cloud") {
        return if night {
            "weather-few-clouds-night-symbolic"
        } else {
            "weather-few-clouds-symbolic"
        };
    }
    if s.contains("mostly sunny") || s.contains("mostly clear") {
        return if night {
            "weather-few-clouds-night-symbolic"
        } else {
            "weather-few-clouds-symbolic"
        };
    }
    if s.contains("sunny") || s.contains("clear") {
        return if night {
            "weather-clear-night-symbolic"
        } else {
            "weather-clear-symbolic"
        };
    }
    if s.contains("cloud") {
        return "weather-overcast-symbolic";
    }

    if night {
        "weather-clear-night-symbolic"
    } else {
        "weather-clear-symbolic"
    }
}

/// Extract a short "City, State" name from a Nominatim display_name.
///
/// Nominatim US results look like:
///   "Denver, City and County of Denver, Colorado, United States"
///   "Chicago, Cook County, Illinois, United States"
///
/// We take the first part (city) and second-to-last part (state),
/// falling back to the full string if the format is unexpected.
pub fn short_location_name(display_name: &str) -> String {
    let parts: Vec<&str> = display_name.split(", ").collect();
    if parts.len() >= 3 {
        let city = parts[0];
        let state = parts[parts.len() - 2];
        format!("{city}, {state}")
    } else {
        display_name.to_string()
    }
}
