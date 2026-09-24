## General / status
default-heading = Wetter
loading = Wetterdaten werden geladen...
fetch-error = Wetterdaten konnten nicht abgerufen werden: {$error}
stale-data = Zwischengespeicherte Daten werden angezeigt (Aktualisierung fehlgeschlagen)
no-location = Kein Standort festgelegt

## Setup & search
setup-title = Standort festlegen
search-placeholder = Stadt suchen...
search-button = Suchen
searching = Suche...
no-results = Keine Ergebnisse gefunden
search-error = Suche fehlgeschlagen: {$error}

## Manage locations
manage-locations = Standorte verwalten
no-saved-locations = Keine gespeicherten Standorte

## Current card - hero
label-wind = Wind
label-precipitation = Niederschlag
label-humidity = Luftfeuchtigkeit
label-aqi = AQI
label-uv = UV
feels-like = Gefühlt {$temp}
gusting-to = Böen bis {$gust}

## Current card - "More" expander
label-more = Mehr
label-less = Weniger
label-dew-point = Taupunkt
label-pressure = Luftdruck
label-air-quality = Luftqualität
label-ozone = Ozon

## Forecast (hourly / daily)
label-sunrise = Sonnenaufgang
label-sunset = Sonnenuntergang

## Alerts
alerts-unavailable = Warnungen nicht verfügbar
alerts-national = { $count ->
    [one] 1 landesweite Warnung, aktuelles Gebiet konnte nicht bestimmt werden
    *[other] { $count } landesweite Warnungen, aktuelles Gebiet konnte nicht bestimmt werden
}
alert-until = Bis { $time }
alert-full-description = Vollständige Beschreibung

## Footer
# `$minutes` is a number, so a plural selector works here if your language needs one.
# Categories are per-language: English has only one/other, Polish adds few/many.
# If a plural selector is used, one variant must be the default, marked `*` - normally *[other].
updated-ago = Vor {$minutes} Min. aktualisiert
updated-now = Gerade aktualisiert

## Reference - UV levels
uv-level-low = Niedrig
uv-level-moderate = Mittel
uv-level-high = Hoch
uv-level-very-high = Sehr hoch
uv-level-extreme = Extrem

## Reference - AQI categories
aqi-cat-good = Gut
aqi-cat-moderate = Mäßig
aqi-cat-unhealthy-sensitive = Gesundheitsschädlich für empfindliche Personen
aqi-cat-unhealthy = Gesundheitsschädlich
aqi-cat-very-unhealthy = Sehr gesundheitsschädlich
aqi-cat-hazardous = Gefährlich
aqi-cat-fair = Ausreichend
aqi-cat-poor = Schlecht
aqi-cat-very-poor = Sehr schlecht
aqi-cat-extremely-poor = Extrem schlecht

## Reference - weather conditions
condition-clear-sky = Wolkenlos
condition-mainly-clear = Überwiegend wolkenlos
condition-partly-cloudy = Teilweise bewölkt
condition-overcast = Bedeckt
condition-fog = Nebel
condition-drizzle = Nieselregen
condition-freezing-drizzle = Gefrierender Nieselregen
condition-rain = Regen
condition-freezing-rain = Eisregen
condition-snow = Schnee
condition-snow-grains = Schneegriesel
condition-rain-showers = Regenschauer
condition-snow-showers = Schneeschauer
condition-thunderstorm = Gewitter
condition-thunderstorm-hail = Gewitter mit Hagel
condition-unknown = Unbekannt

## Reference - compass directions
## ABBREVIATIONS, not words. Use your locale's conventional short form
## (Swedish NE = "NO", not "Nordost")
## These render inline in the wind line of a 360px popup so should be short
compass-n = N
compass-ne = NO
compass-e = O
compass-se = SO
compass-s = S
compass-sw = SW
compass-w = W
compass-nw = NW

## Reference - weekdays
weekday-monday = Montag
weekday-tuesday = Dienstag
weekday-wednesday = Mittwoch
weekday-thursday = Donnerstag
weekday-friday = Freitag
weekday-saturday = Samstag
weekday-sunday = Sonntag

## Reference - relative day labels
day-today = Heute
day-this-afternoon = Heute Nachmittag
day-tonight = Heute Abend
day-overnight = Nacht

# Hourly strip; "Now" heads the current hour's column
hourly-now = Jetzt
# 12-hour clock markers. Keep short — these render in a six-column strip.
time-am = AM
time-pm = PM

##About
app-title = Whether
about = Info
about-summary = Daten von Open-Meteo, NWS & JMA
about-summary-2 = via weathervane
about-homepage = Webseite
about-issues = Problem melden
