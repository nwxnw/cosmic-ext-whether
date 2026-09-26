use crate::types::SearchResult;

const NOMINATIM_URL: &str = "https://nominatim.openstreetmap.org/search";
const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

#[derive(Debug, Clone)]
pub struct GeoError(pub String);

impl std::fmt::Display for GeoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Search for locations by name using Nominatim (worldwide).
pub async fn search_location(query: String) -> Result<Vec<SearchResult>, GeoError> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| GeoError(e.to_string()))?;

    let resp = client
        .get(NOMINATIM_URL)
        .query(&[
            ("q", query.as_str()),
            ("format", "json"),
            ("limit", "5"),
            ("addressdetails", "1"),
        ])
        .send()
        .await
        .map_err(|e| GeoError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(GeoError(format!("Nominatim returned {}", resp.status())));
    }

    let results: Vec<SearchResult> = resp.json().await.map_err(|e| GeoError(e.to_string()))?;

    Ok(results)
}
