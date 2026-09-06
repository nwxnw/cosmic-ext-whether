## General / status
default-heading = Clima
loading = Carregando dados do clima...
fetch-error = Não foi possível obter o clima: {$error}
stale-data = Exibindo dados em cache (falha na atualização)
no-location = Nenhuma localização definida

## Setup & search
setup-title = Defina sua localização
search-placeholder = Buscar uma cidade...
search-button = Buscar
searching = Buscando...
no-results = Nenhum resultado encontrado
search-error = Falha na busca: {$error}

## Manage locations
manage-locations = Gerenciar localizações
no-saved-locations = Nenhuma localização salva

## Current card - hero
label-wind = Vento
label-precipitation = Precipitação
label-humidity = Umidade
label-aqi = IQA
label-uv = UV
feels-like = Sensação de {$temp}
gusting-to = rajadas de até {$gust}

## Current card - "More" expander
label-more = Mais
label-less = Menos
label-dew-point = Ponto de orvalho
label-pressure = Pressão
label-air-quality = Qualidade do ar
label-ozone = Ozônio

## Forecast (hourly / daily)
label-sunrise = Nascer do sol
label-sunset = Pôr do sol

## TODO: Review translations of English stubs
## Alerts
alerts-unavailable = Alertas indisponíveis
alerts-national = { $count ->
    [one] 1 alerta em todo o país, não foi possível determinar sua região
    *[other] { $count } alertas em todo o país, não foi possível determinar sua região
}
alert-until = Até { $time }
alert-full-description = Descrição completa

## Footer
# `$minutes` is a number, so a plural selector works here if your language needs one.
# Categories are per-language: English has only one/other, Polish adds few/many.
# If a plural selector is used, one variant must be the default, marked `*` - normally *[other].
updated-ago = Atualizado há {$minutes} min
updated-now = Atualizado agora

## Reference - UV levels
uv-level-low = Baixo
uv-level-moderate = Moderado
uv-level-high = Alto
uv-level-very-high = Muito alto
uv-level-extreme = Extremo

## Reference - AQI categories
aqi-cat-good = Boa
aqi-cat-moderate = Moderada
aqi-cat-unhealthy-sensitive = Insalubre para grupos sensíveis
aqi-cat-unhealthy = Insalubre
aqi-cat-very-unhealthy = Muito insalubre
aqi-cat-hazardous = Perigosa
aqi-cat-fair = Razoável
aqi-cat-poor = Ruim
aqi-cat-very-poor = Muito ruim
aqi-cat-extremely-poor = Extremamente ruim

## Reference - weather conditions
condition-clear-sky = Céu limpo
condition-mainly-clear = Predominantemente limpo
condition-partly-cloudy = Parcialmente nublado
condition-overcast = Encoberto
condition-fog = Neblina
condition-drizzle = Garoa
condition-freezing-drizzle = Garoa congelante
condition-rain = Chuva
condition-freezing-rain = Chuva congelante
condition-snow = Neve
condition-snow-grains = Grãos de neve
condition-rain-showers = Pancadas de chuva
condition-snow-showers = Pancadas de neve
condition-thunderstorm = Tempestade
condition-thunderstorm-hail = Tempestade com granizo
condition-unknown = Desconhecido

## Reference - compass directions
## ABBREVIATIONS, not words. Use your locale's conventional short form
## (Swedish NE = "NO", not "Nordost")
## These render inline in the wind line of a 360px popup so should be short
compass-n = N
compass-ne = NE
compass-e = L
compass-se = SE
compass-s = S
compass-sw = SO
compass-w = O
compass-nw = NO

## Reference - weekdays
weekday-monday = Segunda-feira
weekday-tuesday = Terça-feira
weekday-wednesday = Quarta-feira
weekday-thursday = Quinta-feira
weekday-friday = Sexta-feira
weekday-saturday = Sábado
weekday-sunday = Domingo

## Reference - relative day labels
day-today = Hoje
day-this-afternoon = Hoje à tarde
day-tonight = Hoje à noite
day-overnight = Madrugada

# Hourly strip; "Now" heads the current hour's column
hourly-now = Agora
# 12-hour clock markers. Keep short — these render in a six-column strip.
time-am = AM
time-pm = PM

##About
app-title = Whether
about = Sobre
about-summary = Dados do Open-Meteo, NWS e JMA
about-summary-2 = via weathervane
about-homepage = Página inicial
about-issues = Relatar um problema
