## Allmänt / Status
default-heading = Väder
loading = Laddar väderdata ...
fetch-error = Kunde inte hämta väder: {$error}
stale-data = Visar cachad data (uppdateringen misslyckades)
no-location = Ingen plats angiven

## Konfigurera & sök
setup-title = Ange din plats
search-placeholder = Sök efter en stad...
search-button = Sök
searching = Söker...
no-results = Inga resultat hittades
search-error = Sökningen misslyckades: {$error}

## Hantera platser
manage-locations = Hantera platser
no-saved-locations = Inga sparade platser

## Nuvarande kort - hjältetitel
label-wind = Vind
label-precipitation = Nederbörd
label-humidity = Fuktighet
label-aqi = AQI
label-uv = UV
feels-like = Känns som {$temp}
gusting-to = vindpustar till {$gust}

## Nuvarande kort - "Mer" expander
label-more = Mer
label-less = Mindre
label-dew-point = Daggpunkt
label-pressure = Tryck
label-air-quality = Luftkvalitet
label-ozone = Ozon

## Prognos (varje timme / dagligen)
label-sunrise = Soluppgång
label-sunset = Solnedgång

## TODO: Review translations of English stubs
##Aviseringar
alerts-unavailable = Varningar otillgängliga
alerts-national = { $count ->
    [one] 1 varning i hela landet, ditt område kunde inte fastställas
    *[other] { $count } varningar i hela landet, ditt område kunde inte fastställas
}
alert-until = Till { $time }
alert-full-description = Fullständig beskrivning

##Sidfot
# `$minutes` is a number, so a plural selector works here if your language needs one.
# Categories are per-language: English has only one/other, Polish adds few/many.
# If a plural selector is used, one variant must be the default, marked `*` - normally *[other].
updated-ago = Uppdaterad {$minutes} min sedan
updated-now = Uppdaterad just nu

## UV etiketter
## Reference - UV etiketter
uv-level-low = Låg
uv-level-moderate = Måttlig
uv-level-high = Hög
uv-level-very-high = Väldigt hög
uv-level-extreme = Extrem

## AQI-kategorier, både EU och USA-kategorier
## Reference - AQI-kategorier
aqi-cat-good = Bra
aqi-cat-moderate = Måttligt
aqi-cat-unhealthy-sensitive = Ohälsosamt för känsliga grupper
aqi-cat-unhealthy = Ohälsosamt
aqi-cat-very-unhealthy = Väldigt ohälsosamt
aqi-cat-hazardous = Farligt
aqi-cat-fair = Skäligt
aqi-cat-poor = Dåligt
aqi-cat-very-poor = Väldigt dåligt
aqi-cat-extremely-poor = Extremt dåligt

## Reference - weather conditions
condition-clear-sky = Klar himmel
condition-mainly-clear = Huvudsakligen klart
condition-partly-cloudy = Delvis molnigt
condition-overcast = Mulet
condition-fog = Dimma
condition-drizzle = Duggregn
condition-freezing-drizzle = Underkylande duggregn
condition-rain = Regn
condition-freezing-rain = Underkylande regn
condition-snow = Snö
condition-snow-grains = Snökorn
condition-rain-showers = Regnskurar
condition-snow-showers = Snöskurar
condition-thunderstorm = Åskväder
condition-thunderstorm-hail = Åskväder med hagel
condition-unknown = Okänt

## Reference - compass directions
## ABBREVIATIONS, not words. Use your locale's conventional short form
compass-n = N
compass-ne = NO
compass-e = O
compass-se = SO
compass-s = S
compass-sw = SV
compass-w = V
compass-nw = NV

## Reference - weekdays
weekday-monday = Måndag
weekday-tuesday = Tisdag
weekday-wednesday = Onsdag
weekday-thursday = Torsdag
weekday-friday = Fredag
weekday-saturday = Lördag
weekday-sunday = Söndag

## Reference - relative day labels
day-today = Idag
day-this-afternoon = Denna eftermiddag
day-tonight = Inatt
day-overnight = Över natten

# Hourly strip; "Now" heads the current hour's column
hourly-now = Nu
# 12-hour clock markers. Keep short — these render in a six-column strip.
time-am = FM
time-pm = EM

## Om sida
app-title = Whether
about = Om
about-summary = Data från Open-Meteo, NWS & JMA
about-summary-2 = via weathervane
about-homepage = Hemsida
about-issues = Rapportera ett problem

