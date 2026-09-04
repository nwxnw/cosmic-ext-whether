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
    pub id: String,
    pub description: String,
    pub expires: chrono::DateTime<chrono::Utc>,
    pub area_desc: String,
    pub event: String,
    pub headline: String,
    pub severity: AlertSeverity,
}

impl WeatherAlert {
    /// UI state key. One CAP identifier spans every area of its alert, so
    /// `id` alone would open every area at once under a national feed
    pub fn key(&self) -> String {
        format!("{}|{}", self.id, self.area_desc)
    }
}

/// One row of a national feed: every alert sharing an (`event`, `severity`)
/// pair, in first-seen order. Never empty.
pub struct AlertGroup<'a> {
    pub members: Vec<&'a WeatherAlert>,
}

impl<'a> AlertGroup<'a> {
    pub fn first(&self) -> &'a WeatherAlert {
        self.members[0]
    }

    pub fn count(&self) -> usize {
        self.members.len()
    }

    /// UI state key for the group row. Prefixed so it can never collide with
    /// a `WeatherAlert::key()`.
    pub fn key(&self) -> String {
        format!("group|{}|{:?}", self.first().event, self.first().severity)
    }

    pub fn areas(&self) -> Vec<&'a str> {
        let mut seen = std::collections::HashSet::new();
        self.members
            .iter()
            .map(|a| a.area_desc.as_str())
            .filter(|a| !a.is_empty() && seen.insert(*a))
            .collect()
    }

    pub fn latest_expires(&self) -> chrono::DateTime<chrono::Utc> {
        self.members
            .iter()
            .map(|a| a.expires)
            .max()
            .unwrap_or_else(|| self.first().expires)
    }
}

/// Group a national feed by (`event`, `severity`) in first-seen order. A
/// national feed is largely one event repeated per region, so alert
/// rows read as duplicates.
pub fn group_alerts(alerts: &[WeatherAlert]) -> Vec<AlertGroup<'_>> {
    let mut groups: Vec<AlertGroup<'_>> = Vec::new();
    for a in alerts {
        match groups
            .iter_mut()
            .find(|g| g.first().event == a.event && g.first().severity == a.severity)
        {
            Some(g) => g.members.push(a),
            None => groups.push(AlertGroup { members: vec![a] }),
        }
    }
    groups
}

#[derive(Debug, Clone)]
pub enum Alerts {
    /// The alert fetch failed while the weather fetch succeeded. Never renders as "no alerts".
    Unavailable(String),
    /// Narrowed to the location by the provider. An empty list is a genuine quiet day.
    Local(Vec<WeatherAlert>),
    /// A MeteoAlarm national feed that could not be narrowed to the location. Entries are national, not local.
    National(Vec<WeatherAlert>),
}

impl Default for Alerts {
    fn default() -> Self {
        Alerts::Local(Vec::new())
    }
}

impl Alerts {
    /// Every alert regardless of state, for views that list them.
    pub fn list(&self) -> &[WeatherAlert] {
        match self {
            Alerts::Local(a) | Alerts::National(a) => a,
            Alerts::Unavailable(_) => &[],
        }
    }
}

/// Unified weather result from either NWS or Open-Meteo.
#[derive(Debug, Clone)]
pub struct WeatherResult {
    pub forecast: Forecast,
    pub cached_grid: Option<GridInfo>,
    pub alerts: Alerts,
    pub observation: Option<CurrentObservation>,
    pub air_quality: Option<AirQuality>,
}

/// Real-time observation data from the nearest station (NWS) or current block (Open-Meteo).
#[derive(Debug, Clone)]
pub struct CurrentObservation {
    pub temperature: Option<i32>,
    pub temperature_unit: String,
    pub condition: Option<weathervane::WeatherCondition>,
    pub wind_speed: Option<String>,
    pub compass: Option<weathervane::CompassDirection>,
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
pub struct Geometry {
    pub coordinates: Vec<Vec<[f64; 2]>>, // Polygon: one ring of [lon, lat]
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastResponse {
    pub properties: ForecastProperties,
    #[serde(default, deserialize_with = "lenient")]
    pub geometry: Option<Geometry>,
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
    #[serde(skip)]
    pub compass: Option<weathervane::CompassDirection>,
    pub short_forecast: String,
    pub detailed_forecast: String,
    pub is_daytime: bool,
    pub probability_of_precipitation: Option<PrecipValue>,
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(skip)]
    pub condition: Option<weathervane::WeatherCondition>,
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
    /// The location's offset east of UTC, from Open-Meteo `timezone=auto`.
    /// Alert expiries are UTC instants and render in this rame so they
    /// agree with the hourly strip and headline's own "until".
    pub utc_offset_seconds: i32,
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
    pub date: Option<String>,
    pub hour: Option<u32>,
    pub high: Option<i32>,
    pub low: Option<i32>,
    pub short_forecast: String,
    pub is_daytime: bool,
    pub wind_speed: String,
    pub wind_direction: String,
    pub compass: Option<weathervane::CompassDirection>,
    pub precip_chance: Option<i32>,
    pub condition: Option<weathervane::WeatherCondition>,
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
                condition: period.condition,
                hour: period.start_time.as_deref().and_then(iso_hour),
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
                compass: period.compass,
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
                condition: period.condition,
                hour: period.start_time.as_deref().and_then(iso_hour),
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
                compass: period.compass,
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

/// Hour-of-day from an ISO 8601 string, read positionally.
///
/// Deliberately avoids chrono: the synthesized strings carry no offset, and
/// `with_timezone(&Local)` would convert to the reader's clock rather than
/// the forecast point's.
pub(crate) fn iso_hour(s: &str) -> Option<u32> {
    let t = s.find('T')?;
    s.get(t + 1..t + 3)?.parse().ok()
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

/// Return the condition.icon_name from weathervane, mapping to weather clear for unknown condition
pub fn condition_icon_for(c: weathervane::WeatherCondition, is_daytime: bool) -> &'static str {
    let night = !is_daytime;

    match c {
        weathervane::WeatherCondition::Unknown => {
            if night {
                "weather-clear-night-symbolic"
            } else {
                "weather-clear-symbolic"
            }
        }
        _ => c.icon_name(night),
    }
}

/// US-daily fallback to map a weather condition string and day/night flag to a symbolic icon name.
pub fn condition_icon_from_text(condition: &str, is_daytime: bool) -> &'static str {
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

/// Tolerate a reshaped `geometry` rather than failing the whole response.
fn lenient<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(T::deserialize(v).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alert(event: &str, severity: AlertSeverity) -> WeatherAlert {
        WeatherAlert {
            id: "x".into(),
            description: String::new(),
            expires: chrono::DateTime::<chrono::Utc>::MIN_UTC,
            area_desc: String::new(),
            event: event.into(),
            headline: String::new(),
            severity,
        }
    }

    #[test]
    fn groups_by_event_and_severity_in_first_seen_order() {
        let list = vec![
            alert("Heat", AlertSeverity::Moderate),
            alert("Heat", AlertSeverity::Severe),
            alert("Heat", AlertSeverity::Moderate),
            alert("Wind", AlertSeverity::Moderate),
        ];
        let groups = group_alerts(&list);
        let rows: Vec<(&str, usize)> = groups
            .iter()
            .map(|g| (g.first().event.as_str(), g.count()))
            .collect();
        assert_eq!(rows, vec![("Heat", 2), ("Heat", 1), ("Wind", 1)]);
        assert_eq!(groups[1].first().severity, AlertSeverity::Severe);
    }
    fn period(is_daytime: bool, temperature: i32, start_time: Option<&str>) -> ForecastPeriod {
        ForecastPeriod {
            name: String::new(),
            temperature,
            temperature_unit: "F".into(),
            wind_speed: String::new(),
            wind_direction: String::new(),
            compass: None,
            short_forecast: String::new(),
            detailed_forecast: String::new(),
            is_daytime,
            probability_of_precipitation: None,
            start_time: start_time.map(str::to_string),
            condition: None,
        }
    }

    fn highs_and_lows(days: &[DaySummary]) -> Vec<(Option<i32>, Option<i32>)> {
        days.iter().map(|d| (d.high, d.low)).collect()
    }

    #[test]
    fn day_and_night_pair_into_one_summary() {
        let periods = [
            period(true, 75, None),
            period(false, 52, None),
            period(true, 71, None),
            period(false, 50, None),
        ];
        let days = pair_daily_periods(&periods);
        assert_eq!(
            highs_and_lows(&days),
            vec![(Some(75), Some(52)), (Some(71), Some(50))]
        );
        assert!(days.iter().all(|d| d.is_daytime));
    }

    #[test]
    fn leading_night_becomes_a_low_only_row() {
        // NWS opens with "Tonight" when the forecast is fetched after dark.
        let periods = [
            period(false, 52, None),
            period(true, 75, None),
            period(false, 50, None),
        ];
        let days = pair_daily_periods(&periods);
        assert_eq!(
            highs_and_lows(&days),
            vec![(None, Some(52)), (Some(75), Some(50))]
        );
        assert!(!days[0].is_daytime);
    }

    #[test]
    fn day_without_a_following_night_keeps_no_low() {
        let periods = [
            period(true, 75, None),
            period(true, 71, None),
            period(false, 50, None),
        ];
        let days = pair_daily_periods(&periods);
        assert_eq!(
            highs_and_lows(&days),
            vec![(Some(75), None), (Some(71), Some(50))]
        );
    }

    #[test]
    fn trailing_day_and_empty_input() {
        assert_eq!(
            highs_and_lows(&pair_daily_periods(&[period(true, 75, None)])),
            vec![(Some(75), None)]
        );
        assert!(pair_daily_periods(&[]).is_empty());
    }

    #[test]
    fn date_and_hour_come_from_start_time() {
        let periods = [
            period(true, 75, Some("2026-09-04T06:00:00-07:00")),
            period(false, 52, None),
        ];
        let days = pair_daily_periods(&periods);
        assert_eq!(days[0].date.as_deref(), Some("2026-09-04"));
        assert_eq!(days[0].hour, Some(6));

        let days = pair_daily_periods(&[period(false, 52, None)]);
        assert_eq!((days[0].date.as_deref(), days[0].hour), (None, None));
    }

    #[test]
    fn precip_chance_truncates_to_whole_percent() {
        let mut p = period(true, 75, None);
        p.probability_of_precipitation = Some(PrecipValue { value: Some(40.0) });
        assert_eq!(pair_daily_periods(&[p])[0].precip_chance, Some(40));

        let mut p = period(true, 75, None);
        p.probability_of_precipitation = Some(PrecipValue { value: None });
        assert_eq!(pair_daily_periods(&[p])[0].precip_chance, None);
    }
}
