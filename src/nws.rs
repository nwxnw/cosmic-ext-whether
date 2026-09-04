use crate::types::{ForecastPeriod, ForecastResponse, Geometry, GridInfo, PointsResponse};

const BASE_URL: &str = "https://api.weather.gov";
const USER_AGENT: &str = "cosmic-ext-whether/0.1.0 (https://github.com/nwxnw/cosmic-ext-whether)";

#[derive(Debug, Clone)]
pub enum NwsError {
    Network(String),
    Api(String),
    Parse(String),
}

impl std::fmt::Display for NwsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NwsError::Network(msg) => write!(f, "Network error: {msg}"),
            NwsError::Api(msg) => write!(f, "API error: {msg}"),
            NwsError::Parse(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

fn client() -> Result<reqwest::Client, NwsError> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| NwsError::Network(e.to_string()))
}

/// Look up the NWS grid for a lat/lon and extract the nearest city name.
pub async fn fetch_points(lat: &str, lon: &str) -> Result<(GridInfo, String), NwsError> {
    let url = format!("{BASE_URL}/points/{lat},{lon}");
    let resp = client()?
        .get(&url)
        .send()
        .await
        .map_err(|e| NwsError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(NwsError::Api(format!(
            "Points API returned {}",
            resp.status()
        )));
    }

    let points: PointsResponse = resp
        .json()
        .await
        .map_err(|e| NwsError::Parse(e.to_string()))?;

    let props = &points.properties;
    let grid = GridInfo {
        office: props.grid_id.clone(),
        grid_x: props.grid_x,
        grid_y: props.grid_y,
        nearest_station: None,
    };

    let location_name = props
        .relative_location
        .as_ref()
        .map(|rl| format!("{}, {}", rl.properties.city, rl.properties.state))
        .unwrap_or_default();

    Ok((grid, location_name))
}

/// Fetch the 7-day forecast for a given grid.
pub async fn fetch_forecast(
    grid: &GridInfo,
    use_fahrenheit: bool,
) -> Result<(Vec<ForecastPeriod>, Option<Geometry>), NwsError> {
    let units = if use_fahrenheit { "us" } else { "si" };
    let url = format!(
        "{BASE_URL}/gridpoints/{}/{},{}/forecast?units={units}",
        grid.office, grid.grid_x, grid.grid_y
    );

    let resp = client()?
        .get(&url)
        .send()
        .await
        .map_err(|e| NwsError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(NwsError::Api(format!(
            "Forecast API returned {}",
            resp.status()
        )));
    }

    let forecast: ForecastResponse = resp
        .json()
        .await
        .map_err(|e| NwsError::Parse(e.to_string()))?;

    Ok((forecast.properties.periods, forecast.geometry))
}

/// Mean of the cell's ring, as `lat,lon`
///
/// The ring repeats its first point as its last; averaging over the duplicate
/// shifts the centroid far less than the 20km gate, so it is not trimmed.
fn ring_centroid(g: &Geometry) -> Option<(f64, f64)> {
    let ring = g.coordinates.first()?;
    if ring.is_empty() {
        return None;
    }
    let n = ring.len() as f64;
    // GeoJson order: p[0] is lon, p[1] is lat.
    let lon: f64 = ring.iter().map(|p| p[0]).sum::<f64>() / n;
    let lat: f64 = ring.iter().map(|p| p[1]).sum::<f64>() / n;
    Some((lat, lon))
}

/// Equirectangular approximation - ample for a 20km gate at this scale.
fn km_between(a: (f64, f64), b: (f64, f64)) -> f64 {
    let mean_lat = ((a.0 + b.0) / 2.0).to_radians();
    let dx = (b.1 - a.1) * mean_lat.cos();
    let dy = b.0 - a.0;
    dx.hypot(dy) * 111.32
}

/// Re-derive a cached grid when its cell centroid is farther than this from
/// the location it is supposed to serve. Cells are ~2.8km across, so a
/// correct grid sits within ~2km; the defect class misplaces by hundreds.
const GRID_MATCH_KM: f64 = 20.0;

/// Does this forecast cell actually belong to these coordinates?
///
/// Fails open: absent or unusable geometry counts as a match, so a reshaped
/// upstream field cannot stop the applet fetching.
pub fn grid_matches(geometry: Option<&Geometry>, lat: f64, lon: f64) -> bool {
    let Some(centroid) = geometry.and_then(ring_centroid) else {
        return true;
    };
    km_between(centroid, (lat, lon)) <= GRID_MATCH_KM
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Closed square ring about 2.8km across, centred on `(lat, lon)`.
    fn cell(lat: f64, lon: f64) -> Geometry {
        let d = 0.0125;
        Geometry {
            coordinates: vec![vec![
                [lon - d, lat - d],
                [lon + d, lat - d],
                [lon + d, lat + d],
                [lon - d, lat + d],
                [lon - d, lat - d],
            ]],
        }
    }

    #[test]
    fn centroid_is_the_ring_mean_in_lat_lon_order() {
        let (lat, lon) = ring_centroid(&cell(45.0, -122.0)).unwrap();
        assert!((lat - 45.0).abs() < 0.01, "{lat}");
        assert!((lon + 122.0).abs() < 0.01, "{lon}");
    }

    #[test]
    fn centroid_is_none_without_a_ring() {
        assert!(ring_centroid(&Geometry {
            coordinates: vec![]
        })
        .is_none());
        assert!(ring_centroid(&Geometry {
            coordinates: vec![vec![]]
        })
        .is_none());
    }

    #[test]
    fn km_between_matches_a_degree_of_latitude() {
        let d = km_between((0.0, 0.0), (1.0, 0.0));
        assert!((d - 111.32).abs() < 0.01, "{d}");
        // A degree of longitude shrinks with the cosine of the latitude.
        let d = km_between((60.0, 0.0), (60.0, 1.0));
        assert!((d - 55.66).abs() < 0.1, "{d}");
    }

    #[test]
    fn grid_matches_its_own_cell() {
        let g = cell(45.0, -122.0);
        assert!(grid_matches(Some(&g), 45.0, -122.0));
        assert!(grid_matches(Some(&g), 45.012, -122.012));
    }

    #[test]
    fn grid_rejects_a_cell_far_away() {
        let g = cell(45.0, -122.0);
        assert!(!grid_matches(Some(&g), 47.6, -122.3)); // ~290km
        assert!(!grid_matches(Some(&g), 45.0, -121.0)); // ~79km
    }

    #[test]
    fn gate_sits_at_twenty_km() {
        // 20km is 0.1797 degrees of latitude on this approximation.
        let g = cell(45.0, -122.0);
        assert!(grid_matches(Some(&g), 45.17, -122.0));
        assert!(!grid_matches(Some(&g), 45.19, -122.0));
    }

    #[test]
    fn grid_fails_open_without_usable_geometry() {
        assert!(grid_matches(None, 45.0, -122.0));
        assert!(grid_matches(
            Some(&Geometry {
                coordinates: vec![]
            }),
            45.0,
            -122.0
        ));
        assert!(grid_matches(
            Some(&Geometry {
                coordinates: vec![vec![]]
            }),
            45.0,
            -122.0
        ));
    }
}
