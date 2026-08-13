use crate::fl;
use crate::nws;
use crate::types::{
    AirQuality, AlertSeverity, CurrentObservation, DailySun, Forecast, ForecastPeriod, GridInfo,
    PrecipValue, WeatherAlert, WeatherResult,
};
use weathervane::{MeasurementSystem, TemperatureUnit};

pub async fn fetch_weather(
    lat: String,
    lon: String,
    use_fahrenheit: bool,
    country_code: Option<String>, //"us" gates the shim
    cached_grid: Option<GridInfo>,
    location_name: String, // from the saved location (geocoding)
) -> Result<WeatherResult, String> {
    let temp_unit = if use_fahrenheit {
        TemperatureUnit::Fahrenheit
    } else {
        TemperatureUnit::Celsius
    };
    let measurement = if use_fahrenheit {
        MeasurementSystem::Imperial
    } else {
        MeasurementSystem::Metric
    };
    let latf: f64 = lat.parse().map_err(|_| "bad latitude".to_string())?;
    let lonf: f64 = lon.parse().map_err(|_| "bad longitude".to_string())?;

    let is_us = country_code.as_deref() == Some("us");

    // shim future: US-only, best-effort. Runs concurrently with weathervane calls
    let shim_fut = async {
        if !is_us {
            return None;
        }
        // reuse cached grid if present, else /points
        let grid = match &cached_grid {
            Some(g) => g.clone(),
            None => match nws::fetch_points(&lat, &lon).await {
                Ok((g, _name)) => g,
                Err(_) => return None,
            },
        };
        match nws::fetch_forecast(&grid, use_fahrenheit).await {
            Ok(periods) => Some((grid, periods)),
            Err(_) => None,
        }
    };

    let (weather_res, aq_res, alerts_res, shim) = tokio::join!(
        weathervane::fetch_weather(latf, lonf, temp_unit, measurement),
        weathervane::fetch_air_quality(latf, lonf, None),
        weathervane::fetch_alerts(latf, lonf),
        shim_fut,
    );

    let weather = weather_res.map_err(|e| e.to_string())?;
    let air_quality = aq_res.ok().map(map_air_quality);
    let alerts = alerts_res
        .unwrap_or_default()
        .into_iter()
        .map(map_alert)
        .collect();

    let unit_str = if use_fahrenheit { "F" } else { "C" };
    let speed_unit = measurement.wind_speed_unit();

    // Only daily periods + cached grid depend on the shim.
    let (daily_periods, cached_grid) = match shim {
        Some((grid, periods)) => (periods, Some(grid)),
        None => (
            build_daily_periods(&weather.forecast, unit_str, speed_unit),
            None,
        ),
    };

    // Everything else always comes from weathervane.
    let sun_times = weather
        .forecast
        .iter()
        .map(|d| DailySun {
            date: d.date.clone(),
            sunrise: d.sunrise.clone(),
            sunset: d.sunset.clone(),
        })
        .collect();

    let hourly_periods =
        build_hourly_periods(&weather.hourly, &weather.forecast, unit_str, speed_unit);

    let observation = Some(build_observation(&weather, unit_str, speed_unit));

    Ok(WeatherResult {
        forecast: Forecast {
            location_name,
            periods: daily_periods,
            hourly_periods,
            sun_times,
        },
        cached_grid,
        alerts,
        observation,
        air_quality,
    })
}

fn map_alert(a: weathervane::Alert) -> WeatherAlert {
    WeatherAlert {
        event: a.event,
        headline: a.headline,
        severity: map_severity(a.severity),
    }
}

fn build_daily_periods(
    daily: &[weathervane::DailyForecast],
    unit: &str,
    speed_unit: &str,
) -> Vec<ForecastPeriod> {
    let mut periods = Vec::with_capacity(daily.len() * 2);
    for (i, d) in daily.iter().enumerate() {
        let day_name = date_to_day_name(&d.date, i == 0);
        let description = condition_label(&d.condition).to_string();
        let wind_dir = d.compass_direction.as_str().to_string();
        let wind_speed = format!("{:.0} {speed_unit}", d.windspeed_max);
        let precip = d.precipitation_probability_max.map(|v| v as f64);

        // Day period (high)
        periods.push(ForecastPeriod {
            name: day_name.clone(),
            temperature: d.temp_max.round() as i32,
            temperature_unit: unit.to_string(),
            wind_speed: wind_speed.clone(),
            wind_direction: wind_dir.clone(),
            short_forecast: description.clone(),
            detailed_forecast: description.clone(),
            is_daytime: true,
            probability_of_precipitation: Some(PrecipValue { value: precip }),
            start_time: Some(format!("{}T12:00:00", d.date)),
        });

        // Night period (low)
        let night_name = if i == 0 {
            fl!("night-tonight")
        } else {
            fl!("night-day", day = day_name.clone())
        };
        periods.push(ForecastPeriod {
            name: night_name,
            temperature: d.temp_min.round() as i32,
            temperature_unit: unit.to_string(),
            wind_speed,
            wind_direction: wind_dir,
            short_forecast: description.clone(),
            detailed_forecast: description,
            is_daytime: false,
            probability_of_precipitation: Some(PrecipValue { value: precip }),
            start_time: Some(format!("{}T00:00:00", d.date)),
        });
    }
    periods
}

fn build_hourly_periods(
    hourly: &[weathervane::HourlyForecast],
    daily: &[weathervane::DailyForecast],
    unit: &str,
    speed_unit: &str,
) -> Vec<ForecastPeriod> {
    hourly
        .iter()
        .map(|h| {
            let description = condition_label(&h.condition).to_string();
            ForecastPeriod {
                name: String::new(),
                temperature: h.temperature.round() as i32,
                temperature_unit: unit.to_string(),
                wind_speed: format!("{:.0} {speed_unit}", h.windspeed),
                wind_direction: String::new(), //HourlyForecast carries no wind direction
                short_forecast: description.clone(),
                detailed_forecast: description,
                is_daytime: hour_is_daytime(&h.time, daily),
                probability_of_precipitation: Some(PrecipValue {
                    value: Some(h.precipitation_probability as f64),
                }),
                start_time: Some(h.time.clone()),
            }
        })
        .collect()
}

fn build_observation(
    w: &weathervane::WeatherData,
    unit: &str,
    speed_unit: &str,
) -> CurrentObservation {
    let cur = &w.current;
    let is_daytime = w
        .forecast
        .first()
        .map(|d| !weathervane::is_night_time(&d.sunrise, &d.sunset, w.utc_offset_seconds))
        .unwrap_or(true);

    CurrentObservation {
        temperature: Some(cur.temperature.round() as i32),
        temperature_unit: unit.to_string(),
        condition: Some(condition_label(&cur.condition).to_string()),
        wind_speed: Some(format!("{:.0} {speed_unit}", cur.windspeed)),
        wind_direction: Some(cur.compass_direction.as_str().to_string()),
        humidity: Some(cur.humidity),
        is_daytime,
        feels_like: Some(cur.feels_like.round() as i32),
        dew_point: Some(cur.dew_point.round() as i32),
        uv_index: Some(cur.uv_index),
        pressure: Some(cur.pressure), // hPa raw; PressureUnit conversion is T5
        cloud_cover: Some(cur.cloud_cover),
        wind_gusts: Some(format!("{:.0} {speed_unit}", cur.wind_gusts)),
        visibility: Some(cur.visibility), // meters raw; convert in T4
    }
}

/// Which day's sunrise/sunset brackets this hour. All times are local-frame ISO
/// string, so a lexigcographic compare is valid.
fn hour_is_daytime(time: &str, daily: &[weathervane::DailyForecast]) -> bool {
    let date = time.get(..10).unwrap_or(time);
    daily
        .iter()
        .find(|d| d.date == date)
        .map(|d| time >= d.sunrise.as_str() && time < d.sunset.as_str())
        .unwrap_or(true)
}

/// ISO date ("2026-03-01") to day name ("Sunday"); "Today" for the first day.
fn date_to_day_name(date_str: &str, is_first: bool) -> String {
    if is_first {
        return fl!("day-today");
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        use chrono::Datelike;
        return match date.weekday() {
            chrono::Weekday::Mon => fl!("day-monday"),
            chrono::Weekday::Tue => fl!("day-tuesday"),
            chrono::Weekday::Wed => fl!("day-wednesday"),
            chrono::Weekday::Thu => fl!("day-thursday"),
            chrono::Weekday::Fri => fl!("day-friday"),
            chrono::Weekday::Sat => fl!("day-saturday"),
            chrono::Weekday::Sun => fl!("day-sunday"),
        };
    }
    date_str.to_string()
}

fn map_severity(s: weathervane::AlertSeverity) -> AlertSeverity {
    use weathervane::AlertSeverity as S;
    match s {
        S::Extreme => AlertSeverity::Extreme,
        S::Severe => AlertSeverity::Severe,
        S::Moderate => AlertSeverity::Moderate,
        S::Minor => AlertSeverity::Minor,
        S::Unknown => AlertSeverity::Unknown,
    }
}

fn map_air_quality(a: weathervane::AirQualityData) -> AirQuality {
    AirQuality {
        aqi: a.aqi,
        category: a.category,
        pm2_5: a.pm2_5,
        pm10: a.pm10,
        ozone: a.ozone,
        no2: a.nitrogen_dioxide,
        co: a.carbon_monoxide,
        severity: aqi_severity_index(&a.category),
    }
}

fn aqi_severity_index(c: &weathervane::AqiCategory) -> u8 {
    use weathervane::{AqiCategory, EuAqiCategory as Eu, UsAqiCategory as Us};
    match c {
        AqiCategory::Us(Us::Good) | AqiCategory::Eu(Eu::Good) => 0,
        AqiCategory::Us(Us::Moderate) | AqiCategory::Eu(Eu::Moderate) => 1,
        AqiCategory::Us(Us::UnhealthySensitive) | AqiCategory::Eu(Eu::Fair) => 2,
        AqiCategory::Us(Us::Unhealthy) | AqiCategory::Eu(Eu::Poor) => 3,
        AqiCategory::Us(Us::VeryUnhealthy) | AqiCategory::Eu(Eu::VeryPoor) => 4,
        AqiCategory::Us(Us::Hazardous) | AqiCategory::Eu(Eu::ExtremelyPoor) => 5,
    }
}

/// Human-readable label for a weathervane condition. Kept keyword-compatible
/// with `types::condition_icon()` (contains "rain"/"snow"/"storm"/"fog"/"cloud"/
/// "clear" etc.) so the existing icon mapping still resolves. Day/night is applied
/// downstream via `is_daytime`, so there's no day/night text split here.
fn condition_label(c: &weathervane::WeatherCondition) -> String {
    use weathervane::WeatherCondition as C;
    match c {
        C::ClearSky => fl!("condition-clear-sky"),
        C::MainlyClear => fl!("condition-mainly-clear"),
        C::PartlyCloudy => fl!("condition-partly-cloudy"),
        C::Overcast => fl!("condition-overcast"),
        C::Foggy => fl!("condition-fog"),
        C::Drizzle => fl!("condition-drizzle"),
        C::FreezingDrizzle => fl!("condition-freezing-drizzle"),
        C::Rain => fl!("condition-rain"),
        C::FreezingRain => fl!("condition-freezing-rain"),
        C::Snow => fl!("condition-snow"),
        C::SnowGrains => fl!("condition-snow-grains"),
        C::RainShowers => fl!("condition-rain-showers"),
        C::SnowShowers => fl!("condition-snow-showers"),
        C::Thunderstorm => fl!("condition-thunderstorm"),
        C::ThunderstormHail => fl!("condition-thunderstorm-hail"),
        C::Unknown => fl!("condition-unknown"),
    }
}
