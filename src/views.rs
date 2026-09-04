use crate::app::{AppModel, Message};
use crate::config::APP_ID;
use crate::fl;
use crate::types::{
    condition_icon_for, condition_icon_from_text, group_alerts, pair_daily_periods, AlertSeverity,
    Alerts, FetchState, Forecast, ForecastPeriod,
};
use cosmic::iced::{Alignment, Color, Length};
use cosmic::{widget, Element};

static AUTOSIZE_FLYOUT_ID: std::sync::LazyLock<widget::Id> =
    std::sync::LazyLock::new(|| widget::Id::new("autosize-flyout"));

fn weather_icon_for_period(period: &ForecastPeriod) -> &'static str {
    match period.condition {
        Some(c) => condition_icon_for(c, period.is_daytime),
        None => condition_icon_from_text(&period.short_forecast, period.is_daytime),
    }
}

pub(crate) fn weather_icon_handle(name: &str) -> cosmic::widget::icon::Handle {
    macro_rules! bundled {
        ($file:literal) => {
            cosmic::widget::icon::from_svg_bytes(
                &include_bytes!(concat!("../icons/", $file, ".svg"))[..],
            )
            .symbolic(true)
        };
    }
    match name {
        "weather-clear-symbolic" => bundled!("weather-clear-symbolic"),
        "weather-clear-night-symbolic" => bundled!("weather-clear-night-symbolic"),
        "weather-few-clouds-symbolic" => bundled!("weather-few-clouds-symbolic"),
        "weather-few-clouds-night-symbolic" => bundled!("weather-few-clouds-night-symbolic"),
        "weather-overcast-symbolic" => bundled!("weather-overcast-symbolic"),
        "weather-showers-symbolic" => bundled!("weather-showers-symbolic"),
        "weather-showers-scattered-symbolic" => bundled!("weather-showers-scattered-symbolic"),
        "weather-snow-symbolic" => bundled!("weather-snow-symbolic"),
        "weather-storm-symbolic" => bundled!("weather-storm-symbolic"),
        "weather-fog-symbolic" => bundled!("weather-fog-symbolic"),
        "weather-severe-alert-symbolic" => bundled!("weather-severe-alert-symbolic"),
        // fallback for any unmapped name
        _ => cosmic::widget::icon::from_name(name).symbolic(true).into(),
    }
}

impl AppModel {
    pub(crate) fn current_temp_text(&self) -> Option<String> {
        // Prefer observation temperature, fall back to first forecast period
        if let Some(obs) = &self.observation {
            if let Some(temp) = obs.temperature {
                return Some(format!("{}°{}", temp, obs.temperature_unit));
            }
        }
        self.forecast.as_ref().and_then(|f| {
            f.periods.first().map(|p| {
                let unit = if p.temperature_unit == "F" { "F" } else { "C" };
                format!("{}°{unit}", p.temperature)
            })
        })
    }

    pub(crate) fn weather_icon_name(&self) -> &str {
        if matches!(&self.alerts, Alerts::Local(a) if !a.is_empty()) {
            return "weather-severe-alert-symbolic";
        }
        // Prefer observation condition for panel icon
        if let Some(obs) = &self.observation {
            if let Some(cond) = obs.condition {
                return condition_icon_for(cond, obs.is_daytime);
            }
        }
        self.forecast
            .as_ref()
            .and_then(|f| f.periods.first())
            .map(weather_icon_for_period)
            .unwrap_or("weather-clear-symbolic")
    }

    pub(crate) fn view_setup(&self) -> Element<'_, Message> {
        let sp = cosmic::theme::spacing();
        let title = fl!("setup-title");
        let placeholder = fl!("search-placeholder");

        let search = widget::text_input(placeholder, &self.search_input)
            .on_input(Message::SearchInput)
            .on_submit(|_| Message::SearchSubmit);

        let search_label = fl!("search-button");
        let search_btn = widget::button::suggested(search_label).on_press_maybe(
            if self.search_input.is_empty() {
                None
            } else {
                Some(Message::SearchSubmit)
            },
        );

        let search_row = cosmic::iced::widget::row![search, search_btn]
            .spacing(sp.space_xxs)
            .align_y(Alignment::Center);

        let mut col = cosmic::iced::widget::column![widget::text::title4(title), search_row,]
            .spacing(sp.space_xs)
            .padding(sp.space_s)
            .width(Length::Fixed(360.0));

        if self.searching {
            let text = fl!("searching");
            col = col.push(widget::text::body(text));
        } else if let Some(e) = &self.search_error {
            let text = fl!("search-error", error = e.as_str());
            col = col.push(widget::text::body(text));
        } else if !self.search_results.is_empty() {
            for (i, result) in self.search_results.iter().enumerate() {
                let btn = widget::button::custom(
                    widget::text::body(&result.display_name).width(Length::Fill),
                )
                .class(cosmic::theme::Button::Text)
                .width(Length::Fill)
                .on_press(Message::SelectLocation(i));
                col = col.push(btn);
            }
        } else if self.search_done {
            let text = fl!("no-results");
            col = col.push(widget::text::body(text));
        }

        col.into()
    }

    pub(crate) fn view_locations(&self) -> Element<'_, Message> {
        let sp = cosmic::theme::spacing();
        let title = fl!("manage-locations");
        let back_btn =
            widget::button::icon(widget::icon::from_name("go-previous-symbolic").symbolic(true))
                .on_press(Message::BackToMain);

        let title_row =
            cosmic::iced::widget::row![back_btn, widget::text::title4(title).width(Length::Fill),]
                .align_y(Alignment::Center)
                .spacing(sp.space_xxs);

        let mut col = cosmic::iced::widget::column![title_row]
            .spacing(sp.space_xs)
            .padding(sp.space_s)
            .width(Length::Fixed(360.0));

        // Saved locations list
        if self.config.locations.is_empty() {
            let text = fl!("no-saved-locations");
            col = col.push(widget::text::body(text));
        } else {
            let mut list = cosmic::iced::widget::column![].spacing(0);
            for (i, loc) in self.config.locations.iter().enumerate() {
                if i > 0 {
                    list = list.push(widget::divider::horizontal::light());
                }

                let is_active = i == self.config.active_location_index;

                let label_col =
                    cosmic::iced::widget::column![widget::text::body(loc.name.clone()),].spacing(2);

                let selected = if is_active { Some(i) } else { None };
                let location_radio =
                    widget::radio(label_col, i, selected, Message::ActivateLocation)
                        .width(Length::Fill);

                let delete_btn = widget::button::icon(
                    widget::icon::from_name("edit-delete-symbolic").symbolic(true),
                )
                .on_press(Message::RemoveLocation(i));

                let row = cosmic::iced::widget::row![location_radio, delete_btn,]
                    .spacing(sp.space_xxs)
                    .align_y(Alignment::Center)
                    .padding([6, 4]);

                list = list.push(row);
            }
            col = col.push(list);
        }

        col = col.push(widget::divider::horizontal::default());

        // Search section (reuses setup pattern)
        let placeholder = fl!("search-placeholder");
        let search = widget::text_input(placeholder, &self.search_input)
            .on_input(Message::SearchInput)
            .on_submit(|_| Message::SearchSubmit);

        let search_label = fl!("search-button");
        let search_btn = widget::button::suggested(search_label).on_press_maybe(
            if self.search_input.is_empty() {
                None
            } else {
                Some(Message::SearchSubmit)
            },
        );

        let search_row = cosmic::iced::widget::row![search, search_btn]
            .spacing(sp.space_xxs)
            .align_y(Alignment::Center);
        col = col.push(search_row);

        if self.searching {
            let text = fl!("searching");
            col = col.push(widget::text::body(text));
        } else if let Some(e) = &self.search_error {
            let text = fl!("search-error", error = e.as_str());
            col = col.push(widget::text::body(text));
        } else if !self.search_results.is_empty() {
            for (i, result) in self.search_results.iter().enumerate() {
                let btn = widget::button::custom(
                    widget::text::body(&result.display_name).width(Length::Fill),
                )
                .class(cosmic::theme::Button::Text)
                .width(Length::Fill)
                .on_press(Message::SelectLocation(i));
                col = col.push(btn);
            }
        } else if self.search_done {
            let text = fl!("no-results");
            col = col.push(widget::text::body(text));
        }

        col.into()
    }

    pub(crate) fn view_main(&self) -> Element<'_, Message> {
        // Pinned outside the scroll area so the location name, the locations
        // chevron and refresh stay reachable at any scroll offset.
        let sp = cosmic::theme::spacing();
        let header = widget::container(self.view_header())
            .width(Length::Fixed(360.0))
            .padding([sp.space_s, sp.space_s, 0, sp.space_s]);

        let body = widget::scrollable(self.view_main_body())
            .class(cosmic::theme::iced::Scrollable::Minimal);

        cosmic::iced::widget::column![header, body]
            .width(Length::Fixed(360.0))
            .into()
    }

    fn view_main_body(&self) -> Element<'_, Message> {
        let sp = cosmic::theme::spacing();
        let mut col = cosmic::iced::widget::column![]
            .spacing(sp.space_xs)
            .padding([sp.space_xs, sp.space_s, sp.space_s, sp.space_s])
            .width(Length::Fixed(360.0));

        if let Some(banner) = self.view_alert_banner() {
            col = col.push(banner);
        }

        // Error / loading states
        match &self.fetch_state {
            FetchState::Loading if self.forecast.is_none() => {
                let text = fl!("loading");
                col = col.push(widget::text::body(text));
                return col.into();
            }
            FetchState::Error(e) => {
                let text = fl!("fetch-error", error = e.as_str());
                col = col.push(widget::text::body(text));
                if self.forecast.is_some() {
                    let stale = fl!("stale-data");
                    col = col.push(widget::text::caption(stale));
                }
            }
            _ => {}
        }

        if let Some(forecast) = &self.forecast {
            if let Some(card) = self.view_current_card(forecast) {
                col = col.push(card);
            }
            if let Some(hourly) = self.view_hourly(forecast) {
                col = col.push(hourly);
            }
            col = col.push(widget::divider::horizontal::default());
            col = col.push(self.view_daily(forecast));
        } else if matches!(self.fetch_state, FetchState::Idle) {
            let text = fl!("no-location");
            col = col.push(widget::text::body(text));
        }

        col = col.push(self.view_footer());
        col.into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        // --- Header: location name heading + chevron + refresh ---
        let sp = cosmic::theme::spacing();
        let location_name = self
            .config
            .locations
            .get(self.config.active_location_index)
            .map(|loc| loc.name.clone())
            .unwrap_or_else(|| fl!("default-heading"));
        let heading = widget::text::title4(location_name).width(Length::Fill);

        let chevron_btn =
            widget::button::icon(widget::icon::from_name("go-next-symbolic").symbolic(true))
                .on_press(Message::AddLocation);

        let refresh_btn =
            widget::button::icon(widget::icon::from_name("view-refresh-symbolic").symbolic(true))
                .on_press(Message::FetchWeather);

        let header_row = cosmic::iced::widget::row![heading, chevron_btn, refresh_btn]
            .align_y(Alignment::Center)
            .spacing(sp.space_xxs);
        header_row.into()
    }

    fn view_alert_banner(&self) -> Option<Element<'_, Message>> {
        // Alert banner
        let sp = cosmic::theme::spacing();
        if let Alerts::Unavailable(_) = &self.alerts {
            let text = fl!("alerts-unavailable");
            let caption =
                widget::text::caption(text).class(cosmic::theme::Text::Color(muted_color()));
            return Some(caption.into());
        }
        let alerts = self.alerts.list();
        if !alerts.is_empty() {
            let mut alert_col = cosmic::iced::widget::column![].spacing(sp.space_xxxs);

            match &self.alerts {
                Alerts::National(list) => {
                    let caption = fl!("alerts-national", count = list.len());
                    alert_col = alert_col.push(
                        widget::text::caption(caption)
                            .class(cosmic::theme::Text::Color(muted_color())),
                    );
                    for group in group_alerts(list) {
                        let expanded = self.expanded_alerts.contains(&group.key());
                        let title = cosmic::iced::widget::row![
                            widget::text::body(capitalize_first(&group.first().event))
                                .width(Length::Fill),
                            widget::text::caption(group.count().to_string())
                                .class(cosmic::theme::Text::Color(muted_color())),
                        ]
                        .spacing(sp.space_xs)
                        .align_y(Alignment::Center)
                        .width(Length::Fill);
                        let mut text_col = cosmic::iced::widget::column![title]
                            .spacing(sp.space_xxxs)
                            .width(Length::Fill);
                        if expanded {
                            text_col = text_col.push(
                                widget::text::caption(group.areas().join(", "))
                                    .class(cosmic::theme::Text::Color(muted_color())),
                            );
                            text_col = text_col.push(
                                widget::text::caption(self.expiry_label(group.latest_expires()))
                                    .class(cosmic::theme::Text::Color(muted_color())),
                            );
                        }
                        let row = cosmic::iced::widget::row![
                            alert_glyph(&group.first().severity),
                            text_col
                        ]
                        .spacing(sp.space_xs)
                        .align_y(Alignment::Start);
                        let row_btn = widget::button::custom(row)
                            .on_press(Message::ToggleAlert(group.key()))
                            .width(Length::Fill)
                            .class(flat_toggle_button_style());
                        alert_col = alert_col.push(row_btn);
                    }
                }
                _ => {
                    for alert in alerts {
                        let expanded = self.expanded_alerts.contains(&alert.key());
                        let mut event = widget::text::body(capitalize_first(&alert.event));
                        if alert.severity == AlertSeverity::Extreme {
                            event = event.font(cosmic::font::bold());
                        }
                        let mut text_col =
                            cosmic::iced::widget::column![event].spacing(sp.space_xxxs);
                        if expanded {
                            text_col = text_col.push(widget::text::body(alert.headline.clone()));
                            text_col = text_col.push(
                                widget::text::caption(self.expiry_label(alert.expires))
                                    .class(cosmic::theme::Text::Color(muted_color())),
                            )
                        }
                        let row =
                            cosmic::iced::widget::row![alert_glyph(&alert.severity), text_col]
                                .spacing(sp.space_xs)
                                .align_y(Alignment::Start);
                        let row_btn = widget::button::custom(row)
                            .on_press(Message::ToggleAlert(alert.key()))
                            .width(Length::Fill)
                            .class(flat_toggle_button_style());
                        alert_col = alert_col.push(row_btn);
                        if expanded && !alert.description.is_empty() {
                            let key = alert.key();
                            let label = cosmic::iced::widget::row![
                                widget::text::body(fl!("alert-full-description")),
                                widget::icon::from_name("go-next-symbolic")
                                    .size(16)
                                    .symbolic(true),
                            ]
                            .spacing(sp.space_xxxs)
                            .align_y(Alignment::Center);
                            alert_col = alert_col.push(
                                widget::container(
                                    widget::button::custom(label)
                                        .on_press_with_rectangle(move |offset, bounds| {
                                            Message::ToggleFlyout(key.clone(), offset, bounds)
                                        })
                                        .class(cosmic::theme::Button::Link),
                                )
                                .padding([
                                    0,
                                    0,
                                    0,
                                    16 + sp.space_xs,
                                ]),
                            );
                        }
                    }
                }
            }

            let alert_banner = widget::layer_container(alert_col)
                .layer(cosmic::cosmic_theme::Layer::Secondary)
                .padding(sp.space_xs)
                .width(Length::Fill);
            Some(alert_banner.into())
        } else {
            None
        }
    }

    fn view_current_card(&self, forecast: &Forecast) -> Option<Element<'_, Message>> {
        // Shared by the hero card
        let muted = muted_color();
        let sp = cosmic::theme::spacing();

        // --- Hero section ---
        if let Some(current) = forecast.periods.first() {
            // Prefer observation data when available, fall back to forecast period
            let (
                hero_temp,
                hero_unit,
                hero_condition,
                hero_icon_name,
                hero_wind,
                hero_humidity,
                hero_feels_like,
            ) = if let Some(obs) = &self.observation {
                let temp = obs.temperature.unwrap_or(current.temperature);
                let unit = &obs.temperature_unit;
                let cond = obs
                    .condition
                    .map(condition_label)
                    .unwrap_or_else(|| current.short_forecast.clone());
                let icon = obs
                    .condition
                    .map(|c| condition_icon_for(c, obs.is_daytime))
                    .unwrap_or_else(|| weather_icon_for_period(current));
                let wind = match (&obs.wind_speed, obs.compass) {
                    (Some(speed), Some(dir)) => Some(format!("{speed} {}", compass_label(dir))),
                    _ => None,
                };
                (
                    temp,
                    unit.clone(),
                    cond,
                    icon,
                    wind,
                    obs.humidity,
                    obs.feels_like,
                )
            } else {
                let dir = current
                    .compass
                    .map(compass_label)
                    .unwrap_or_else(|| current.wind_direction.clone());
                let wind = format!("{} {}", current.wind_speed, dir);
                (
                    current.temperature,
                    current.temperature_unit.clone(),
                    current.short_forecast.clone(),
                    weather_icon_for_period(current),
                    Some(wind),
                    None, // humidity
                    None, // feels_like
                )
            };

            let icon: Element<'_, Message> =
                cosmic::widget::icon(weather_icon_handle(hero_icon_name))
                    .size(28)
                    .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(|theme| {
                        cosmic::iced::widget::svg::Style {
                            color: Some(theme.cosmic().background(theme.transparent).on.into()),
                        }
                    })))
                    .into();

            let temp_label = format!("{}°{hero_unit}", hero_temp);
            let temp_btn = widget::button::custom(widget::text::title3(temp_label))
                .class(cosmic::widget::button::ButtonClass::Link)
                .on_press(Message::ToggleUnits);

            let icon_temp_row = cosmic::iced::widget::row![icon, temp_btn]
                .spacing(sp.space_xs)
                .align_y(Alignment::Center);

            let hero_uv_index = self.observation.as_ref().and_then(|o| o.uv_index);
            let hero_wind_gusts = self.observation.as_ref().and_then(|o| o.wind_gusts.clone());
            let hero_wind_speed = self.observation.as_ref().and_then(|o| o.wind_speed.clone());
            let hero_wind_dir = self
                .observation
                .as_ref()
                .and_then(|o| o.compass)
                .map(compass_label);

            let mut hero_content = cosmic::iced::widget::column![icon_temp_row]
                .spacing(2)
                .padding(sp.space_xs)
                .width(Length::Fill);

            // Condition · Feels like
            let mut cond = vec![(String::new(), hero_condition)];
            if let Some(f) = hero_feels_like {
                cond.push((
                    String::new(),
                    fl!("feels-like", temp = format!("{f}°{hero_unit}")),
                ))
            }
            hero_content = hero_content.push(stat_line(muted, cond));

            // Wind (+ optional gusts)
            if let (Some(speed), Some(dir)) = (&hero_wind_speed, &hero_wind_dir) {
                let mut value = format!("{speed} {dir}");
                if let Some(g) = &hero_wind_gusts {
                    value.push_str(&format!(", {}", fl!("gusting-to", gust = g.as_str())));
                }
                hero_content =
                    hero_content.push(stat_line(muted, vec![(fl!("label-wind"), value)]));
            } else if let Some(wind) = hero_wind {
                // Fallback (no obs wind components): label style, matching the primary path
                hero_content = hero_content.push(stat_line(muted, vec![(fl!("label-wind"), wind)]));
            }

            // Precipitation · Humidity
            let mut ph: Vec<(String, String)> = Vec::new();
            if let Some(p) = &current.probability_of_precipitation {
                let chance = (p.value.unwrap_or(0.0) as i32).to_string();
                ph.push((fl!("label-precipitation"), format!("{chance}%")));
            }
            if let Some(h) = hero_humidity {
                ph.push((fl!("label-humidity"), format!("{h}%")));
            }
            if !ph.is_empty() {
                hero_content = hero_content.push(stat_line(muted, ph));
            }

            // AQI + UV (health line). Muted labels + values like the other lines;
            // escalations override: colored pill at Unhealthy+ (forces a row), bold
            // value at Extreme UV (a bold span — no row needed).
            let uv: Option<(String, bool)> = hero_uv_index
                .filter(|u| *u >= 3.0)
                .map(|u| (format!("{} {}", u.round() as i32, uv_level(u)), u >= 11.0));
            let aqi = self.air_quality.as_ref();

            if let Some(a) = aqi.filter(|a| a.severity >= 3) {
                let sev = a.severity;
                let label = format!(
                    "{}: {} {}",
                    fl!("label-aqi"),
                    a.aqi,
                    aqi_category_label(a.category)
                );
                let pill: Element<'_, Message> = widget::container(widget::text::body(label))
                    .padding([2, 8])
                    .class(cosmic::theme::Container::custom(move |theme| {
                        let (bg, fg) = aqi_style(sev, theme);
                        cosmic::widget::container::Style {
                            icon_color: None,
                            text_color: Some(fg),
                            background: Some(cosmic::iced::Background::Color(bg)),
                            border: cosmic::iced::Border {
                                radius: theme.cosmic().radius_s().into(),
                                ..Default::default()
                            },
                            shadow: cosmic::iced::Shadow::default(),
                            snap: true,
                        }
                    }))
                    .into();
                let mut health = cosmic::iced::widget::row![pill]
                    .spacing(sp.space_xxs)
                    .align_y(Alignment::Center);
                if let Some((uvs, extreme)) = uv {
                    let mut uv_span = cosmic::iced::widget::span::<(), _>(uvs);
                    if extreme {
                        uv_span = uv_span.font(cosmic::font::bold());
                    }
                    let uv_el: Element<'_, Message> = cosmic::iced::widget::rich_text([
                        cosmic::iced::widget::span::<(), _>("·  ").color(muted),
                        cosmic::iced::widget::span::<(), _>(format!("{}  ", fl!("label-uv")))
                            .color(muted),
                        uv_span,
                    ])
                    .into();
                    health = health.push(uv_el);
                }
                let health: Element<'_, Message> = health.into();
                hero_content = hero_content.push(health);
            } else {
                let mut spans = Vec::new();
                if let Some(a) = aqi {
                    spans.push(
                        cosmic::iced::widget::span::<(), _>(format!("{}  ", fl!("label-aqi")))
                            .color(muted),
                    );
                    spans.push(cosmic::iced::widget::span::<(), _>(format!(
                        "{} {}",
                        a.aqi,
                        aqi_category_label(a.category)
                    )));
                }
                if let Some((uvs, extreme)) = uv {
                    if !spans.is_empty() {
                        spans.push(cosmic::iced::widget::span::<(), _>("  ·  ").color(muted));
                    }
                    spans.push(
                        cosmic::iced::widget::span::<(), _>(format!("{}  ", fl!("label-uv")))
                            .color(muted),
                    );
                    let mut uv_span = cosmic::iced::widget::span::<(), _>(uvs);
                    if extreme {
                        uv_span = uv_span.font(cosmic::font::bold());
                    }
                    spans.push(uv_span);
                }
                if !spans.is_empty() {
                    let line: Element<'_, Message> = cosmic::iced::widget::rich_text(spans).into();
                    hero_content = hero_content.push(line);
                }
            }
            hero_content = hero_content.push(self.view_current_more(&hero_unit, muted));

            let hero = widget::layer_container(hero_content)
                .layer(cosmic::cosmic_theme::Layer::Secondary)
                .width(Length::Fill);

            Some(hero.into())
        } else {
            None
        }
    }

    fn view_current_more(&self, hero_unit: &str, muted: Color) -> Element<'_, Message> {
        // --- "More" expander: secondary obs (dew point, pressure) + AQI
        // pollutants. Mirrors the daily accordion (ToggleDay). Body stays inside
        // the hero card — no nested Secondary layer — so it reads as one surface.
        // PM2.5 / PM10 stay literal (universal abbreviations); Ozone via label-ozone.
        let sp = cosmic::theme::spacing();
        let (more_icon, more_word) = if self.current_expanded {
            ("pan-up-symbolic", fl!("label-less"))
        } else {
            ("pan-down-symbolic", fl!("label-more"))
        };
        let more_row = cosmic::iced::widget::row![
            widget::icon::from_name(more_icon).symbolic(true).size(16),
            widget::text::body(more_word),
        ]
        .spacing(sp.space_xxs)
        .align_y(Alignment::Center)
        .padding(0);
        let more_btn = widget::button::custom(more_row)
            .on_press(Message::ToggleCurrentMore)
            .width(Length::Fill)
            .padding([2, 4])
            .class(flat_toggle_button_style());
        let mut more = cosmic::iced::widget::column![more_btn]
            .spacing(2)
            .width(Length::Fill);

        if self.current_expanded {
            let obs = self.observation.as_ref();
            let aqi = self.air_quality.as_ref();
            let mut more_col = cosmic::iced::widget::column![]
                .spacing(2)
                .padding([6, 0, 0, 0]);

            let mut secondary: Vec<(String, String)> = Vec::new();
            if let Some(dew) = obs.and_then(|o| o.dew_point) {
                secondary.push((fl!("label-dew-point"), format!("{dew}°{hero_unit}")));
            }
            if let Some(p) = obs.and_then(|o| o.pressure) {
                secondary.push((
                    fl!("label-pressure"),
                    format_pressure(p, self.config.use_fahrenheit),
                ));
            }
            if !secondary.is_empty() {
                more_col = more_col.push(stat_line(muted, secondary));
            }

            // AQI pollutant sub-block (only when air quality is present).
            if let Some(a) = aqi {
                let heading: Element<'_, Message> = cosmic::iced::widget::rich_text([
                    cosmic::iced::widget::span::<(), _>(fl!("label-air-quality")),
                    cosmic::iced::widget::span::<(), _>("  (µg/m³)").color(muted),
                ])
                .into();
                more_col = more_col.push(heading);
                more_col = more_col.push(stat_line(
                    muted,
                    vec![
                        ("PM2.5".to_string(), format!("{:.0}", a.pm2_5)),
                        ("PM10".to_string(), format!("{:.0}", a.pm10)),
                        (fl!("label-ozone"), format!("{:.0}", a.ozone)),
                    ],
                ));
            }

            more = more.push(more_col);
        }

        more.into()
    }

    fn view_hourly(&self, forecast: &Forecast) -> Option<Element<'_, Message>> {
        let sp = cosmic::theme::spacing();
        // --- Hourly forecast (paged with arrow buttons) ---
        if !forecast.hourly_periods.is_empty() {
            let total = forecast.hourly_periods.len();
            let offset = self
                .hourly_offset
                .min(total.saturating_sub(crate::app::HOURLY_PAGE_SIZE));
            let end = (offset + crate::app::HOURLY_PAGE_SIZE).min(total);
            let can_prev = offset > 0;
            let can_next = end < total;

            let prev_arrow: Element<'_, Message> = if can_prev {
                widget::button::icon(
                    widget::icon::from_name("go-previous-symbolic")
                        .symbolic(true)
                        .size(16),
                )
                .on_press(Message::HourlyPrev)
                .into()
            } else {
                widget::Space::new()
                    .width(Length::Fixed(16.0 + 2.0 * f32::from(sp.space_xxs)))
                    .into()
            };

            let mut hourly_row = cosmic::iced::widget::row![].spacing(0);
            for i in offset..end {
                let period = &forecast.hourly_periods[i];
                let hour_label = if i == 0 {
                    fl!("hourly-now")
                } else {
                    period
                        .start_time
                        .as_deref()
                        .map(|s| format_hour(s, self.military_time))
                        .unwrap_or_default()
                };

                let icon_name = weather_icon_for_period(period);
                let icon: Element<'_, Message> =
                    cosmic::widget::icon(weather_icon_handle(icon_name))
                        .size(24)
                        .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(|theme| {
                            cosmic::iced::widget::svg::Style {
                                color: Some(theme.cosmic().background(theme.transparent).on.into()),
                            }
                        })))
                        .into();

                let temp = widget::text::body(format!("{}°", period.temperature));

                let mut hour_col =
                    cosmic::iced::widget::column![widget::text::caption(hour_label), icon, temp,]
                        .spacing(sp.space_xxxs)
                        .align_x(Alignment::Center)
                        .width(Length::Fill);

                let has_precip_icon = icon_name.contains("showers")
                    || icon_name.contains("storm")
                    || icon_name.contains("snow");
                if let Some(precip) = period
                    .probability_of_precipitation
                    .as_ref()
                    .and_then(|p| p.value)
                {
                    let pct = precip as u32;
                    if has_precip_icon || pct >= 20 {
                        hour_col = hour_col.push(widget::text::caption(format!("{}%", pct)));
                    }
                }

                hourly_row = hourly_row.push(hour_col);
            }

            let next_arrow: Element<'_, Message> = if can_next {
                widget::button::icon(
                    widget::icon::from_name("go-next-symbolic")
                        .symbolic(true)
                        .size(16),
                )
                .on_press(Message::HourlyNext)
                .into()
            } else {
                widget::Space::new()
                    .width(Length::Fixed(16.0 + 2.0 * f32::from(sp.space_xxs)))
                    .into()
            };

            let paged_row = cosmic::iced::widget::row![prev_arrow, hourly_row, next_arrow]
                .spacing(sp.space_xxxs)
                .align_y(Alignment::Center)
                .width(Length::Fill);
            return Some(paged_row.into());
        }

        None
    }

    fn view_daily(&self, forecast: &Forecast) -> Element<'_, Message> {
        // --- Daily forecast (clickable rows with inline expansion) ---
        {
            let muted = muted_color();
            let sp = cosmic::theme::spacing();
            let summaries = pair_daily_periods(&forecast.periods);
            let mut rows = cosmic::iced::widget::column![].spacing(0);

            for (i, day) in summaries.iter().enumerate() {
                if i > 0 {
                    rows = rows.push(widget::divider::horizontal::light());
                }

                let is_expanded = self.expanded_day == Some(i);

                let icon_name = forecast_icon_for_summary(day);
                let icon: Element<'_, Message> =
                    cosmic::widget::icon(weather_icon_handle(icon_name))
                        .size(24)
                        .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(|theme| {
                            cosmic::iced::widget::svg::Style {
                                color: Some(theme.cosmic().background(theme.transparent).on.into()),
                            }
                        })))
                        .into();

                let name_text = widget::text::body(day_label(day, i)).width(Length::Fill);

                let temp_str = match (day.high, day.low) {
                    (Some(h), Some(l)) => {
                        format!("{}° / {}°", h, l)
                    }
                    (Some(h), None) => format!("{}°", h),
                    (None, Some(l)) => format!("— / {}°", l),
                    (None, None) => "—".to_string(),
                };
                let temp_text = widget::text::body(temp_str);

                let row_content = cosmic::iced::widget::row![icon, name_text, temp_text]
                    .spacing(sp.space_xxs)
                    .align_y(Alignment::Center)
                    .padding([6, 4]);

                let row_btn = widget::button::custom(row_content)
                    .on_press(Message::ToggleDay(i))
                    .width(Length::Fill)
                    .class(flat_toggle_button_style());

                rows = rows.push(row_btn);

                if is_expanded {
                    let mut detail_col = cosmic::iced::widget::column![].spacing(sp.space_xxxs);

                    // NWS supplies distinct short/detailed strings while OM offers only a condition label.
                    // Whether pulls the prose forecast from NWS via a shim. If prose forecast is
                    // available, wind+precip details are omitted from whether daily forecast.

                    let has_prose = !day.detailed_forecast.is_empty()
                        && day.detailed_forecast != day.short_forecast;

                    // 1. Summary / prose - leads (bare, full-strength body text)
                    let summary = if has_prose {
                        day.detailed_forecast.clone()
                    } else {
                        day.condition
                            .map(condition_label)
                            .unwrap_or_else(|| day.short_forecast.clone())
                    };
                    if !summary.is_empty() {
                        detail_col = detail_col.push(widget::text::body(summary));
                    }

                    // 2. Wind + Precip - structured stats. OM only: NWS prose already
                    // narrates wind/precip
                    if !has_prose {
                        let dir = day
                            .compass
                            .map(compass_label)
                            .unwrap_or_else(|| day.wind_direction.clone());
                        let wind_val = format!("{} {}", day.wind_speed, dir);
                        detail_col =
                            detail_col.push(stat_line(muted, vec![(fl!("label-wind"), wind_val)]));
                        if let Some(chance) = day.precip_chance {
                            detail_col = detail_col.push(stat_line(
                                muted,
                                vec![(fl!("label-precipitation"), format!("{chance}%"))],
                            ));
                        }
                    }

                    // 4. Sunrise / sunset
                    if let Some(sun) = day
                        .date
                        .as_ref()
                        .and_then(|d| forecast.sun_times.iter().find(|s| &s.date == d))
                    {
                        let sunrise = weathervane::format_time(&sun.sunrise, self.military_time);
                        let sunset = weathervane::format_time(&sun.sunset, self.military_time);
                        detail_col = detail_col.push(stat_line(
                            muted,
                            vec![
                                (fl!("label-sunrise"), sunrise),
                                (fl!("label-sunset"), sunset),
                            ],
                        ));
                    }

                    let detail = widget::layer_container(detail_col.padding([
                        sp.space_xxxs,
                        sp.space_s,
                        sp.space_xxs,
                        4 + 24 + sp.space_xxs,
                    ]))
                    .layer(cosmic::cosmic_theme::Layer::Secondary)
                    .width(Length::Fill);
                    rows = rows.push(detail);
                }
            }

            rows.into()
        }
    }

    fn view_footer(&self) -> Element<'_, Message> {
        // --- Footer: "Updated X min ago"(left) + "Whether · vX" About link (right) ---
        let updated_text = self.last_updated.map(|updated| {
            let elapsed = updated.elapsed().as_secs() / 60;
            if elapsed == 0 {
                fl!("updated-now")
            } else {
                fl!("updated-ago", minutes = elapsed)
            }
        });
        let about_link = widget::button::custom(widget::text::caption(format!(
            "{} · v{}",
            fl!("app-title"),
            env!("CARGO_PKG_VERSION")
        )))
        .class(cosmic::theme::Button::Link)
        .on_press(Message::OpenAbout);
        cosmic::iced::widget::row![
            widget::text::caption(updated_text.unwrap_or_default()).width(Length::Fill),
            about_link,
        ]
        .align_y(Alignment::Center)
        .into()
    }
    pub(crate) fn view_about(&self) -> Element<'_, Message> {
        let sp = cosmic::theme::spacing();
        let back_btn =
            widget::button::icon(widget::icon::from_name("go-previous-symbolic").symbolic(true))
                .on_press(Message::BackToMain);
        let title_row = cosmic::iced::widget::row![
            back_btn,
            widget::text::title4(fl!("about")).width(Length::Fill),
        ]
        .align_y(Alignment::Center)
        .spacing(sp.space_xxs);

        let icon = widget::icon::from_name(format!("{APP_ID}-symbolic"))
            .symbolic(true)
            .size(64);
        let name = widget::text::title4(fl!("app-title"));
        let summary = widget::text::caption(fl!("about-summary"));
        let summary2 = widget::text::caption(fl!("about-summary-2"));
        let version = widget::text::body(format!("v{}", env!("CARGO_PKG_VERSION")));
        let identity = cosmic::iced::widget::column![icon, name, summary, summary2, version]
            .align_x(Alignment::Center)
            .spacing(sp.space_xxxs);

        let homepage = widget::button::link(fl!("about-homepage")).on_press(Message::OpenUrl(
            "https://github.com/nwxnw/cosmic-ext-whether".to_string(),
        ));
        let issues = widget::button::link(fl!("about-issues")).on_press(Message::OpenUrl(
            "https://github.com/nwxnw/cosmic-ext-whether/issues".to_string(),
        ));
        let links = cosmic::iced::widget::column![homepage, issues]
            .align_x(Alignment::Center)
            .spacing(sp.space_xxxs);

        let license = widget::text::caption("GPL-3.0");

        let body = cosmic::iced::widget::column![identity, links, license]
            .align_x(Alignment::Center)
            .spacing(sp.space_s)
            .width(Length::Fill)
            .padding(sp.space_s);

        let content = cosmic::iced::widget::column![title_row, body]
            .spacing(sp.space_xs)
            .padding(sp.space_s);
        widget::container(widget::scrollable(content))
            .width(Length::Fixed(360.0))
            .into()
    }

    pub(crate) fn view_flyout(&self) -> Element<'_, Message> {
        let sp = cosmic::theme::spacing();
        let alert = self
            .flyout_for
            .as_deref()
            .and_then(|key| self.alerts.list().iter().find(|a| a.key() == key));
        let content: Element<'_, Message> = match alert {
            Some(alert) => {
                let title = widget::text::title4(capitalize_first(&alert.event));
                // Monospace and never reflowed: NWS text is column-aligned
                // teletype with hard breaks, and the width is sized to it.
                let body = widget::scrollable(widget::text::monotext(alert.description.clone()))
                    .class(cosmic::theme::iced::Scrollable::Minimal);
                cosmic::iced::widget::column![title, body]
                    .spacing(sp.space_xs)
                    .into()
            }
            // Owner gone; the invariant destroys this surface on the next message.
            None => widget::text::body("").into(),
        };
        let card =
            widget::container(content)
                .padding(sp.space_s)
                .class(cosmic::theme::Container::custom(|theme| {
                    let cosmic = theme.cosmic();
                    let bg = cosmic.background(theme.transparent);
                    cosmic::iced::widget::container::Style {
                        text_color: Some(bg.on.into()),
                        background: Some(cosmic::iced::Color::from(bg.base).into()),
                        border: cosmic::iced::Border {
                            radius: cosmic.corner_radii.radius_m.into(),
                            width: 1.0,
                            color: bg.divider.into(),
                        },
                        ..Default::default()
                    }
                }));
        widget::autosize::autosize(card, AUTOSIZE_FLYOUT_ID.clone())
            .limits(
                cosmic::iced::Limits::NONE
                    .min_width(1.0)
                    .max_width(crate::app::FLYOUT_MAX_WIDTH)
                    .min_height(1.0)
                    .max_height(1000.0),
            )
            .into()
    }

    fn expiry_label(&self, expires: chrono::DateTime<chrono::Utc>) -> String {
        use chrono::{Datelike, FixedOffset};
        // The location's clock, not the viewer's, so the line agrees with the
        // hourly strip beside it. Falls back to the viewer's clock only if no
        // forecast is loaded, which cannot happen while alerts render.
        let offset = self
            .forecast
            .as_ref()
            .and_then(|f| FixedOffset::east_opt(f.utc_offset_seconds))
            .unwrap_or_else(|| *chrono::Local::now().offset());
        let at = expires.with_timezone(&offset);
        let today = chrono::Utc::now().with_timezone(&offset).date_naive();
        let time = weathervane::format_time(&at.to_rfc3339(), self.military_time);
        let when = if at.date_naive() == today {
            time
        } else {
            format!("{} {}", weekday_label(at.weekday()), time)
        };
        fl!("alert-until", time = when)
    }
}

fn forecast_icon_for_summary(day: &crate::types::DaySummary) -> &'static str {
    match day.condition {
        Some(c) => condition_icon_for(c, day.is_daytime),
        None => condition_icon_from_text(&day.short_forecast, day.is_daytime),
    }
}

// AQI loud tier (bands 3-5) - (background, text). The quiet tier (0-2) blends into
// the card's Secondary surface via the theme (see aqi_style), so it reads as plain
// text and only "grows in" a colored pill as severity rises.
const AQI_FILL_LIGHT: [(Color, Color); 3] = [
    (
        Color::from_rgb8(0xd2, 0x44, 0x44),
        Color::from_rgb8(0xff, 0xff, 0xff),
    ), // 3 Unhealthy/Poor
    (
        Color::from_rgb8(0x8f, 0x3f, 0x97),
        Color::from_rgb8(0xff, 0xff, 0xff),
    ), // 4 Very Unhealthy
    (
        Color::from_rgb8(0x72, 0x2a, 0x35),
        Color::from_rgb8(0xff, 0xff, 0xff),
    ), // 5 Hazardous
];

const AQI_FILL_DARK: [(Color, Color); 3] = [
    (
        Color::from_rgb8(0xb8, 0x3f, 0x3f),
        Color::from_rgb8(0xff, 0xff, 0xff),
    ),
    (
        Color::from_rgb8(0x94, 0x4a, 0x9c),
        Color::from_rgb8(0xff, 0xff, 0xff),
    ),
    (
        Color::from_rgb8(0x8a, 0x3a, 0x48),
        Color::from_rgb8(0xff, 0xff, 0xff),
    ),
];

fn aqi_style(severity: u8, theme: &cosmic::Theme) -> (Color, Color) {
    let cosmic = theme.cosmic();
    if severity >= 3 {
        let table = if cosmic.is_dark {
            &AQI_FILL_DARK
        } else {
            &AQI_FILL_LIGHT
        };
        table[(severity as usize - 3).min(2)]
    } else {
        // Quiet tier: blend into the Secondary surface + normal text -> plain-text look
        (
            cosmic.secondary(false).base.into(),
            cosmic.secondary(false).on.into(),
        )
    }
}

/// One hero detail line from (label, value) pairs: muted labels + full-strength
/// values, with a muted "  ·  " separator between pairs. An empty label emits the
/// value alone (e.g. the weather condition). Spans own their strings → 'static.
fn stat_line(muted: Color, pairs: Vec<(String, String)>) -> Element<'static, Message> {
    use cosmic::iced::widget::{rich_text, span};
    let mut spans = Vec::new();
    for (i, (label, value)) in pairs.into_iter().enumerate() {
        if i > 0 {
            spans.push(span::<(), _>("  ·  ").color(muted));
        }
        if !label.is_empty() {
            spans.push(span::<(), _>(format!("{label}  ")).color(muted));
        }
        spans.push(span::<(), _>(value));
    }
    rich_text(spans).into()
}

/// Pressure formatted for the current unit preference: inHg (imperial) or hPa.
/// `hpa` is the raw value from the observation. (T5 replaces the `bool` with the
/// 3-way pressure-unit cycle.)
fn format_pressure(hpa: f32, imperial: bool) -> String {
    if imperial {
        format!("{:.2} inHg", hpa * 0.02953)
    } else {
        format!("{:.0} hPa", hpa)
    }
}

/// Flat, borderless toggle-button styling shared by the daily forecast rows and the
/// current-card "More" control: transparent until interaction, then a component
/// hover/pressed fill with a small radius.
fn flat_toggle_button_style() -> cosmic::theme::Button {
    fn flat() -> cosmic::widget::button::Style {
        cosmic::widget::button::Style {
            background: None,
            border_width: 0.0,
            border_color: cosmic::iced::Color::TRANSPARENT,
            outline_width: 0.0,
            outline_color: cosmic::iced::Color::TRANSPARENT,
            icon_color: None,
            text_color: None,
            overlay: None,
            shadow_offset: Default::default(),
            border_radius: Default::default(),
        }
    }
    cosmic::theme::Button::Custom {
        active: Box::new(|_focused, _theme| flat()),
        disabled: Box::new(|_theme| flat()),
        hovered: Box::new(|_focused, theme| {
            let cosmic = theme.cosmic();
            cosmic::widget::button::Style {
                background: Some(cosmic::iced::Background::Color(
                    cosmic.background(theme.transparent).component.hover.into(),
                )),
                border_radius: cosmic.radius_s().into(),
                ..flat()
            }
        }),
        pressed: Box::new(|_focused, theme| {
            let cosmic = theme.cosmic();
            cosmic::widget::button::Style {
                background: Some(cosmic::iced::Background::Color(
                    cosmic
                        .background(theme.transparent)
                        .component
                        .pressed
                        .into(),
                )),
                border_radius: cosmic.radius_s().into(),
                ..flat()
            }
        }),
    }
}

fn muted_color() -> Color {
    let mut c: Color = cosmic::theme::active().cosmic().background(false).on.into();
    c.a = 0.7;
    c
}

/// Glyph tint by CAP severity. Minor and Unknown stay at the surface's text
/// color so only the rows that matter draw the eye.
fn severity_tint(severity: &AlertSeverity, theme: &cosmic::Theme) -> Color {
    let cosmic = theme.cosmic();
    match severity {
        AlertSeverity::Extreme | AlertSeverity::Severe => cosmic.destructive_color().into(),
        AlertSeverity::Moderate => cosmic.warning_color().into(),
        AlertSeverity::Minor | AlertSeverity::Unknown => {
            cosmic.background(theme.transparent).on.into()
        }
    }
}

/// Per-row alert glyph, tinted by severity.
fn alert_glyph<'a>(severity: &AlertSeverity) -> Element<'a, Message> {
    let sev = severity.clone();
    cosmic::widget::icon(weather_icon_handle("weather-severe-alert-symbolic"))
        .size(16)
        .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(move |theme| {
            cosmic::iced::widget::svg::Style {
                color: Some(severity_tint(&sev, theme)),
            }
        })))
        .into()
}

/// MeteoAlarm events arrive as the feed wrote them ("wind gusts"); NWS
/// events are title-cased. Lift the first letter so both read alike.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn uv_level(uv: f32) -> String {
    if uv < 3.0 {
        fl!("uv-level-low") // not reached (caller gates at >=3.0), but keeps the fn total
    } else if uv < 6.0 {
        fl!("uv-level-moderate")
    } else if uv < 8.0 {
        fl!("uv-level-high")
    } else if uv < 11.0 {
        fl!("uv-level-very-high")
    } else {
        fl!("uv-level-extreme")
    }
}

fn aqi_category_label(c: weathervane::AqiCategory) -> String {
    use weathervane::{AqiCategory, EuAqiCategory as Eu, UsAqiCategory as Us};
    match c {
        AqiCategory::Us(Us::Good) | AqiCategory::Eu(Eu::Good) => fl!("aqi-cat-good"),
        AqiCategory::Us(Us::Moderate) | AqiCategory::Eu(Eu::Moderate) => fl!("aqi-cat-moderate"),
        AqiCategory::Us(Us::UnhealthySensitive) => fl!("aqi-cat-unhealthy-sensitive"),
        AqiCategory::Us(Us::Unhealthy) => fl!("aqi-cat-unhealthy"),
        AqiCategory::Us(Us::VeryUnhealthy) => fl!("aqi-cat-very-unhealthy"),
        AqiCategory::Us(Us::Hazardous) => fl!("aqi-cat-hazardous"),
        AqiCategory::Eu(Eu::Fair) => fl!("aqi-cat-fair"),
        AqiCategory::Eu(Eu::Poor) => fl!("aqi-cat-poor"),
        AqiCategory::Eu(Eu::VeryPoor) => fl!("aqi-cat-very-poor"),
        AqiCategory::Eu(Eu::ExtremelyPoor) => fl!("aqi-cat-extremely-poor"),
    }
}

fn compass_label(d: weathervane::CompassDirection) -> String {
    use weathervane::CompassDirection as D;
    match d {
        D::N => fl!("compass-n"),
        D::NE => fl!("compass-ne"),
        D::E => fl!("compass-e"),
        D::SE => fl!("compass-se"),
        D::S => fl!("compass-s"),
        D::SW => fl!("compass-sw"),
        D::W => fl!("compass-w"),
        D::NW => fl!("compass-nw"),
    }
}

fn weekday_label(w: chrono::Weekday) -> String {
    use chrono::Weekday as W;
    match w {
        W::Sun => fl!("weekday-sunday"),
        W::Mon => fl!("weekday-monday"),
        W::Tue => fl!("weekday-tuesday"),
        W::Wed => fl!("weekday-wednesday"),
        W::Thu => fl!("weekday-thursday"),
        W::Fri => fl!("weekday-friday"),
        W::Sat => fl!("weekday-saturday"),
    }
}

/// Row label for a daily forecast row.
///
/// Rows 1+ resolve the weekday from the ISO date so the name is localized;
/// Row 0 uses the the four-band rule over `is_daytime` and the period's start hour.
/// Falls back to the raw ISO date string if it is missing or unparseable.
fn day_label(day: &crate::types::DaySummary, index: usize) -> String {
    if index == 0 {
        let hour = day.hour.unwrap_or(if day.is_daytime { 6 } else { 18 });
        return match (day.is_daytime, hour) {
            (true, h) if h < 12 => fl!("day-today"),
            (true, _) => fl!("day-this-afternoon"),
            (false, h) if h >= 6 => fl!("day-tonight"),
            (false, _) => fl!("day-overnight"),
        };
    }

    day.date
        .as_deref()
        .and_then(weathervane::ParsedDate::from_iso)
        .map(|d| weekday_label(d.weekday))
        .unwrap_or_else(|| day.date.clone().unwrap_or_default())
}

fn condition_label(c: weathervane::WeatherCondition) -> String {
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
/// Display hour for an hourly column head, e.g. "3 PM" or "15:00"
///
/// Input format: "2026-02-28T14:00:00-08:00" Minutes are always :00 by
/// construction, so 24h emits `{hour:02}:00` rather than a full clock time.
fn format_hour(start_time: &str, military_time: bool) -> String {
    let Some(hour) = crate::types::iso_hour(start_time) else {
        return start_time.to_string();
    };

    if military_time {
        return format!("{hour:02}:00");
    }
    let (display, marker) = match hour {
        0 => (12, fl!("time-am")),
        1..=11 => (hour, fl!("time-am")),
        12 => (12, fl!("time-pm")),
        _ => (hour - 12, fl!("time-pm")),
    };
    format!("{display} {marker}")
}
