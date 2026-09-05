## General / status
default-heading = Pogoda
loading = Ładowanie danych pogodowych...
fetch-error = Nie udało się pobrać pogody: {$error}
stale-data = Pokazuje wcześniej pobrane dane (aktualizacja nieudana)
no-location = Brak ustawionego położenia

## Setup & search
setup-title = Ustal swoje położenie
search-placeholder = Wyszukaj miasto...
search-button = Szukaj
searching = Szukanie...
no-results = Nie znaleziono wyników
search-error = Wyszukiwanie nieudane: {$error}

## Manage locations
manage-locations = Zarządzaj Położeniami
no-saved-locations = Brak zapisanych położeń

## Current card - hero
label-wind = Wiatr
label-precipitation = Opady
label-humidity = Wilgotność
label-aqi = AQI
label-uv = UV
feels-like = Odczuwalna {$temp}
gusting-to = w porywach do {$gust}

## Current card - "More" expander
label-more = Więcej
label-less = Mniej
label-dew-point = Punkt rosy
label-pressure = Ciśnienie
label-air-quality = Jakość powietrza
label-ozone = Ozon

## Forecast (hourly / daily)
label-sunrise = Wschód słońca
label-sunset = Zachód słońca

## Alerts
alerts-unavailable = Ostrzeżenia niedostępne
alerts-national = { $count ->
    [one] 1 ostrzeżenie ogólnokrajowe, nie można określić twojej jednostki terytorialnej
    [few] { $count } ostrzeżenia ogólnokrajowe, nie można określić twojej jednostki terytorialnej
    *[other] { $count } ostrzeżeń ogólnokrajowych, nie można określić twojej jednostki terytorialnej
}
alert-until = Przez { $time }
alert-full-description = Pełny opis

## Footer
# `$minutes` is a number, so a plural selector works here if your language needs one.
# Categories are per-language: English has only one/other, Polish adds few/many.
# If a plural selector is used, one variant must be the default, marked `*` - normally *[other].
updated-ago = Zaktualizowano {$minutes} { $minutes ->
        [one] minutę
        [few] minuty
       *[other] minut
    } temu
updated-now = Zaktualizowano przed chwilą

## Reference - UV levels
uv-level-low = Niski
uv-level-moderate = Średni
uv-level-high = Wysoki
uv-level-very-high = Bardzo wysoki
uv-level-extreme = Ekstremalny

## Reference - AQI categories
aqi-cat-good = Dobra
aqi-cat-moderate = Średnia
aqi-cat-unhealthy-sensitive = Niezdrowa dla wrażliwych grup
aqi-cat-unhealthy = Niezdrowa
aqi-cat-very-unhealthy = Bardzo niezdrowa
aqi-cat-hazardous = Niebezpieczna
aqi-cat-fair = Umiarkowana
aqi-cat-poor = Słaba
aqi-cat-very-poor = Bardzo słaba
aqi-cat-extremely-poor = Ekstremalnie słaba

## Reference - weather conditions
condition-clear-sky = Bezchmurne niebo
condition-mainly-clear = W większości bezchmurnie
condition-partly-cloudy = Częściowe zachmurzenie
condition-overcast = Zachmurzenie
condition-fog = Mgła
condition-drizzle = Mżawka
condition-freezing-drizzle = Marznąca mżawka
condition-rain = Deszcz
condition-freezing-rain = Marznący deszcz
condition-snow = Śnieg
condition-snow-grains = Ziarenka śniegu
condition-rain-showers = Przelotny deszcz
condition-snow-showers = Przelotny śnieg
condition-thunderstorm = Burza
condition-thunderstorm-hail = Burza z gradem
condition-unknown = Nieznane

## Reference - compass directions
## ABBREVIATIONS, not words. Use your locale's conventional short form
## (Swedish NE = "NO", not "Nordost")
## These render inline in the wind line of a 360px popup so should be short
compass-n = pn.
compass-ne = pn.-wsch.
compass-e = wsch.
compass-se = pd.-wsch.
compass-s = pd.
compass-sw = pd.-zach.
compass-w = zach.
compass-nw = pn.-zach.

## Reference - weekdays
weekday-monday = Poniedziałek
weekday-tuesday = Wtorek
weekday-wednesday = Środa
weekday-thursday = Czwartek
weekday-friday = Piątek
weekday-saturday = Sobota
weekday-sunday = Niedziela

## Reference - relative day labels
day-today = Dzisiaj
day-this-afternoon = Popołudniu
day-tonight = Wieczorem
day-overnight = W nocy

# Hourly strip; "Now" heads the current hour's column
hourly-now = Teraz
# 12-hour clock markers. Keep short — these render in a six-column strip.
time-am = AM
time-pm = PM

##About
app-title = Whether
about = O aplecie
about-summary = Dane z Open-Meteo, NWS & JMA
about-summary-2 = poprzez weathervane
about-homepage = Strona domowa
about-issues = Zgłoś problem
