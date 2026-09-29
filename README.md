# Whether

A weather applet for the [COSMIC](https://github.com/pop-os/cosmic-epoch) desktop panel.

![Whether in Swedish, Polish, Brazilian Portuguese, and English](screenshots/whether-languages.png)

## Features

- **Current conditions** - temperature, feels-like, humidity, wind & gusts, precipitation, AQI, and UV index
  - Click **More** for dew point, pressure, and the pollutant breakdown (PM2.5, PM10, ozone)
- **Weather alerts** for the US, Europe, Canada, and Australia, shown inline with a severity icon and tint, expandable to the headline, affected areas, and local expiry time
  - Detailed alert text (available in the US and Canada) opens in a side panel instead of filling the popup
- **Hourly and 7-day forecasts** - click any day to expand its detailed forecast, wind, precipitation, and sunrise/sunset
- **Automatic data sourcing by location** via [weathervane](https://github.com/crenshawdev/weathervane):
  - **Open-Meteo** - the worldwide base forecast, free ([CC BY 4.0](https://open-meteo.com/en/license))
  - **NWS** (National Weather Service) - overlays US current conditions and daily forecast detail, free
  - **JMA** - overlays the current temperature in Japan from AMeDAS observations, free
- **Location search** via [Nominatim](https://nominatim.openstreetmap.org/) (OpenStreetMap), with multiple saved locations
- **Imperial/Metric toggle** - click the hero temperature to switch between imperial (°F, mph, inHg) and metric (°C, km/h, hPa)
- **Panel display** - day/night-aware weather icon and current temperature

## Localization

Whether supports five languages today and can support more. Thanks to these contributors, Whether is available in English and:

- **Swedish** - [@bittin](https://github.com/bittin)
- **Polish** - [@skajmer](https://github.com/skajmer), [@VandaLHJ](https://github.com/VandaLHJ)
- **Portuguese (Brazil)** - [@wag-panfilli](https://github.com/wag-panfilli)
- **German** - [@GreatFlo](https://github.com/GreatFlo)

To add a language, copy `i18n/en/cosmic_ext_whether.ftl` into a new locale directory under `i18n/`, translate the value to the right of each `=` while leaving the keys and `{$placeholders}` unchanged, then open a pull request.

Some of what you see comes from the weather service rather than from Whether, and appears in whatever language that agency publishes:

- **Weather alert text** (event names, headlines, area names, and detailed reports) is written by the issuing agency (NWS, MeteoAlarm, ECCC, BOM) and keeps its wording.
- **US daily forecast detail** ("Sunny, with a high near 82. North northwest wind 2 to 9 mph.") is authored by the National Weather Service and is English only.
- **Condition descriptions** fall back to the provider's own wording when a report doesn't map to a known condition.

Everything Whether renders itself - conditions, wind directions, weekdays, times, and all interface text - is translated.

## Installation

### COSMIC Store

Whether is available in the COSMIC Store in the Applets category. Installing from the Store provides automatic updates.

To remove it, use the Store, or run `flatpak uninstall com.github.nwxnw.cosmic-ext-whether`.

### Manual download

Download the latest `.deb` or `.flatpak` release from the [Releases](https://github.com/nwxnw/cosmic-ext-whether/releases) page.

**Debian/Ubuntu (.deb)**
- Install: `sudo apt install ./cosmic-ext-whether_*_amd64.deb`
- Uninstall: `sudo apt remove cosmic-ext-whether`

**Flatpak bundle**
- Install: `flatpak install --user ./cosmic-ext-whether_*.flatpak`
- Uninstall: `flatpak uninstall --user com.github.nwxnw.cosmic-ext-whether`

### From source

Requires Rust, [just](https://github.com/casey/just), and the `pkg-config`, `libxkbcommon-dev`, `wayland-protocols`, and `libwayland-dev` system dependencies.

- Install: `just build-release && just install`
- Uninstall: `just uninstall`

If you previously installed via `sudo just install`, run `sudo just uninstall-system` once to clear the old `/usr` install.

## License

[GPL-3.0-only](LICENSE)
