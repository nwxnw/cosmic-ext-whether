use crate::types::SearchResult;
use i18n_embed::unic_langid::LanguageIdentifier;

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

/// `Accept-Language` value from the system locale list, most preferred
/// first, with English appended as the lowest-priority fallback so place
/// names without a translation in the user's language don't stay in the
/// local script. `None` when the system reports no usable locale.
fn accept_language(langs: &[LanguageIdentifier]) -> Option<String> {
    if langs.is_empty() {
        return None;
    }
    let mut tags: Vec<String> = Vec::new();
    for lang in langs {
        let tag = lang.to_string();
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    if !langs.iter().any(|l| l.language.as_str() == "en") {
        tags.push("en".to_string());
    }
    Some(
        tags.iter()
            .enumerate()
            .map(|(i, tag)| match i {
                0 => tag.clone(),
                _ => format!("{tag};q=0.{}", 10usize.saturating_sub(i).max(1)),
            })
            .collect::<Vec<_>>()
            .join(","),
    )
}

/// Search for locations by name using Nominatim (worldwide).
pub async fn search_location(query: String) -> Result<Vec<SearchResult>, GeoError> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| GeoError(e.to_string()))?;

    let mut req = client.get(NOMINATIM_URL).query(&[
        ("q", query.as_str()),
        ("format", "json"),
        ("limit", "5"),
        ("addressdetails", "1"),
    ]);
    let langs = i18n_embed::DesktopLanguageRequester::requested_languages();
    if let Some(value) = accept_language(&langs) {
        req = req.header(reqwest::header::ACCEPT_LANGUAGE, value);
    }

    let resp = req.send().await.map_err(|e| GeoError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(GeoError(format!("Nominatim returned {}", resp.status())));
    }

    let results: Vec<SearchResult> = resp.json().await.map_err(|e| GeoError(e.to_string()))?;

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(tags: &[&str]) -> Vec<LanguageIdentifier> {
        tags.iter().map(|t| t.parse().unwrap()).collect()
    }

    #[test]
    fn no_locale_sends_no_header() {
        assert_eq!(accept_language(&[]), None);
    }

    #[test]
    fn appends_english_fallback() {
        assert_eq!(
            accept_language(&ids(&["sv-SE"])).as_deref(),
            Some("sv-SE,en;q=0.9")
        );
    }

    #[test]
    fn english_locale_gets_no_extra_fallback() {
        assert_eq!(accept_language(&ids(&["en-US"])).as_deref(), Some("en-US"));
    }

    #[test]
    fn keeps_order_and_drops_duplicates() {
        assert_eq!(
            accept_language(&ids(&["de-DE", "de-DE", "de"])).as_deref(),
            Some("de-DE,de;q=0.9,en;q=0.8")
        );
    }
}
