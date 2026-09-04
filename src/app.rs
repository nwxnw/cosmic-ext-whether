use std::collections::HashSet;
use std::sync::LazyLock;

use cosmic::app::{Core, Task};
use cosmic::cctk::wayland_protocols::xdg::shell::client::xdg_positioner::{Anchor, Gravity};
use cosmic::iced::core::window;
use cosmic::iced::runtime::platform_specific::wayland::popup::{SctkPopupSettings, SctkPositioner};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Limits, Rectangle, Subscription};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget;
use cosmic::Application;
use cosmic::Element;

static AUTOSIZE_MAIN_ID: LazyLock<widget::Id> = LazyLock::new(|| widget::Id::new("autosize-main"));

pub(crate) const FLYOUT_MAX_WIDTH: f32 = 680.0;

use crate::backend;
use crate::config::{self, detect_military_time, WhetherConfig, APP_ID};
use crate::geocoding;
use crate::types::{
    group_alerts, short_location_name, AirQuality, Alerts, CurrentObservation, FetchState,
    Forecast, SavedLocation, SearchResult, WeatherResult,
};
use crate::views::weather_icon_handle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Main,
    Setup,
    Locations,
    About,
}

pub struct AppModel {
    pub(crate) core: Core,
    pub(crate) popup: Option<Id>,
    pub(crate) flyout: Option<Id>,
    pub(crate) flyout_for: Option<String>,
    pub(crate) air_quality: Option<AirQuality>,
    pub(crate) current_expanded: bool,
    pub(crate) config: WhetherConfig,
    pub(crate) config_handle: Option<cosmic::cosmic_config::Config>,
    pub(crate) forecast: Option<Forecast>,
    pub(crate) observation: Option<CurrentObservation>,
    pub(crate) fetch_state: FetchState,
    pub(crate) page: Page,
    pub(crate) search_input: String,
    pub(crate) search_results: Vec<SearchResult>,
    pub(crate) searching: bool,
    pub(crate) search_error: Option<String>,
    pub(crate) search_done: bool,
    pub(crate) hourly_offset: usize,
    pub(crate) expanded_day: Option<usize>,
    pub(crate) last_updated: Option<std::time::Instant>,
    pub(crate) location_names: Vec<String>,
    pub(crate) alerts: Alerts,
    /// Alert ids the user has expanded. Keyed on id, not index, so the set
    /// survives a refresh in which entries shift as alerts expire.
    pub(crate) expanded_alerts: HashSet<String>,
    pub(crate) fetch_generation: u64,
    pub(crate) military_time: bool,
}

/// Number of hourly columns visible at once between the arrow buttons.
pub(crate) const HOURLY_PAGE_SIZE: usize = 6;

impl Default for AppModel {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            flyout: None,
            flyout_for: None,
            air_quality: None,
            current_expanded: false,
            config: WhetherConfig::default(),
            config_handle: None,
            forecast: None,
            observation: None,
            fetch_state: FetchState::Idle,
            page: Page::Setup,
            search_input: String::new(),
            search_results: Vec::new(),
            searching: false,
            search_error: None,
            search_done: false,
            hourly_offset: 0,
            expanded_day: None,
            last_updated: None,
            location_names: Vec::new(),
            alerts: Alerts::default(),
            expanded_alerts: HashSet::new(),
            fetch_generation: 0,
            military_time: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    /// Panel button pressed while the popup is open: destroy child first.
    ClosePopup,
    /// The fly-out trigger row: (`layout.virtual_offset()`, row bounds).
    ToggleFlyout(String, cosmic::iced::Vector, Rectangle),
    Surface(cosmic::surface::Action),
    FetchWeather,
    WeatherFetched(u64, Box<Result<WeatherResult, String>>),
    Tick(()),
    SearchInput(String),
    SearchSubmit,
    SearchResults(Result<Vec<SearchResult>, String>),
    SelectLocation(usize),

    ActivateLocation(usize),

    AddLocation,
    RemoveLocation(usize),
    BackToMain,
    ToggleUnits,
    HourlyPrev,
    HourlyNext,
    ToggleDay(usize),
    ToggleCurrentMore,
    ToggleAlert(String),
    ConfigChanged(WhetherConfig),
    OpenAbout,
    OpenUrl(String),
    Ignore,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let (config, config_handle) = config::load_config();
        let page = if config.locations.is_empty() {
            Page::Setup
        } else {
            Page::Main
        };

        let location_names = config.locations.iter().map(|l| l.name.clone()).collect();
        let military_time = detect_military_time();

        let mut app = Self {
            core,
            config,
            config_handle,
            page,
            location_names,
            military_time,
            ..Default::default()
        };

        let task = if app.config.active_location().is_some() {
            app.start_fetch()
        } else {
            Task::none()
        };

        (app, task)
    }

    fn view(&self) -> Element<'_, Message> {
        let have_popup = self.popup;
        let icon_name = self.weather_icon_name();
        let suggested_size = self.core.applet.suggested_size(true);
        let sp = cosmic::theme::spacing();

        let icon: Element<'_, Message> = cosmic::widget::icon(weather_icon_handle(icon_name))
            .size(suggested_size.0)
            .class(cosmic::theme::Svg::Custom(std::rc::Rc::new(|theme| {
                cosmic::iced::widget::svg::Style {
                    color: Some(theme.cosmic().background(theme.transparent).on.into()),
                }
            })))
            .into();

        let content: Element<'_, Message> = if let Some(temp) = self.current_temp_text() {
            let temp_widget = widget::text::body(temp).width(Length::Shrink);
            if self.core.applet.is_horizontal() {
                cosmic::iced::widget::row![icon, temp_widget]
                    .spacing(sp.space_xxxs)
                    .align_y(Alignment::Center)
                    .into()
            } else {
                cosmic::iced::widget::column![icon, temp_widget]
                    .spacing(2)
                    .align_x(Alignment::Center)
                    .into()
            }
        } else {
            icon
        };

        let horizontal = self.core.applet.is_horizontal();
        let pad = self.core.applet.suggested_padding(true).0;

        let button = cosmic::widget::button::custom(content)
            .padding(if horizontal { [0, pad] } else { [pad, 0] })
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_with_rectangle(move |offset, bounds| {
                if have_popup.is_some() {
                    Message::ClosePopup
                } else {
                    Message::Surface(app_popup::<AppModel>(
                        |_| Default::default(),
                        move |state: &mut AppModel| {
                            let new_id = Id::unique();
                            state.military_time = detect_military_time();
                            state.popup = Some(new_id);
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );
                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            // The panel centers the applet window in a slot of icon size plus
                            // twice the minor-axis padding; push the popup out to the slot edge.
                            let (icon_w, icon_h) = state.core.applet.suggested_size(true);
                            let (_, minor) = state.core.applet.suggested_padding(true);
                            let (slot, extent) = if state.core.applet.is_horizontal() {
                                (f32::from(icon_h + 2 * minor), bounds.height)
                            } else {
                                (f32::from(icon_w + 2 * minor), bounds.width)
                            };
                            let shortfall = ((slot - extent) / 2.0).max(0.0).round() as i32;
                            let (ox, oy) = popup_settings.positioner.offset;
                            popup_settings.positioner.offset =
                                (ox + ox.signum() * shortfall, oy + oy.signum() * shortfall);
                            popup_settings
                        },
                        None,
                    ))
                }
            });

        widget::autosize::autosize(button, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, id: Id) -> Element<'_, Message> {
        if self.flyout == Some(id) {
            return self.view_flyout();
        }
        let content = match &self.page {
            Page::Setup => self.view_setup(),
            Page::Main => self.view_main(),
            Page::Locations => self.view_locations(),
            Page::About => self.view_about(),
        };
        self.core.applet.popup_container(content).into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let task = self.handle(message);
        self.reconcile_flyout(task)
    }

    fn subscription(&self) -> Subscription<Message> {
        let config_sub = self
            .core()
            .watch_config::<WhetherConfig>(APP_ID)
            .map(|update| Message::ConfigChanged(update.config));

        let timer_sub = cosmic::iced::time::every(std::time::Duration::from_secs(
            self.config.refresh_interval_minutes as u64 * 60,
        ))
        .map(|_| Message::Tick(()));

        Subscription::batch(vec![config_sub, timer_sub])
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    /// Destroy the fly-out when the row it hangs off is no longer on screen:
    /// collapsed, pruned by a fetch, cleared by a location change, or hidden
    /// by a page change. One invariant instead of a destroy at every site.
    fn reconcile_flyout(&mut self, task: Task<Message>) -> Task<Message> {
        if self.flyout.is_none() && self.flyout_for.is_none() {
            return task;
        }
        if self
            .flyout_for
            .as_deref()
            .is_some_and(|key| self.flyout_row_visible(key))
        {
            return task;
        }
        self.flyout_for = None;
        match self.flyout.take() {
            // Batched, not chained: after a location change `task` is the
            // fetch, and the destroy must not wait for it.
            Some(id) => Task::batch([task, self.update(Message::Surface(destroy_popup(id)))]),
            None => task,
        }
    }

    fn flyout_row_visible(&self, key: &str) -> bool {
        self.page == Page::Main
            && self.expanded_alerts.contains(key)
            && self.alerts.list().iter().any(|a| a.key() == key)
    }

    fn handle(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::PopupClosed(id) => {
                if self.flyout == Some(id) {
                    self.flyout = None;
                    self.flyout_for = None;
                }
                if self.popup == Some(id) {
                    self.popup = None;
                    self.flyout = None;
                    self.flyout_for = None;
                }
            }
            Message::ClosePopup => {
                let Some(popup) = self.popup else {
                    return Task::none();
                };
                let close_parent = self.update(Message::Surface(destroy_popup(popup)));
                return match self.flyout.take() {
                    Some(child) => self
                        .update(Message::Surface(destroy_popup(child)))
                        .chain(close_parent),
                    None => close_parent,
                };
            }
            Message::ToggleFlyout(key, offset, bounds) => {
                let close = self
                    .flyout
                    .take()
                    .map(|id| self.update(Message::Surface(destroy_popup(id))));
                let same_row = self.flyout_for.take().is_some_and(|k| k == key);
                if same_row {
                    return close.unwrap_or_else(Task::none);
                }
                let Some(parent) = self.popup else {
                    return Task::none();
                };
                self.flyout_for = Some(key);
                let anchor_rect = Rectangle {
                    x: (bounds.x - offset.x) as i32,
                    y: (bounds.y - offset.y) as i32,
                    width: bounds.width as i32,
                    height: bounds.height as i32,
                };
                let open = self.update(Message::Surface(app_popup::<AppModel>(
                    |_| Default::default(),
                    move |state: &mut AppModel| {
                        let id = Id::unique();
                        state.flyout = Some(id);
                        SctkPopupSettings {
                            parent,
                            id,
                            positioner: SctkPositioner {
                                size: None,
                                size_limits: Limits::NONE
                                    .min_width(1.0)
                                    .max_width(FLYOUT_MAX_WIDTH)
                                    .min_height(1.0)
                                    .max_height(1000.0),
                                anchor_rect,
                                anchor: Anchor::TopRight,
                                gravity: Gravity::BottomRight,
                                // slide_y, flip_x, flip_y; no slide_x, so it flips
                                // to the other side at a screen edge, as the menu does.
                                constraint_adjustment: 15 & !1,
                                offset: (0, 0),
                                reactive: true,
                            },
                            parent_size: None,
                            // The parent already holds the seat grab, so the child
                            // inside its tree stays interactive either way; true keeps
                            // dismissal with the compositor. Spike, 2026-09-04.
                            grab: true,
                            close_with_children: false,
                            input_zone: None,
                        }
                    },
                    None,
                )));
                // Destroy the old child before the new one opens; the same
                // ordered chain the panel-button close uses.
                return match close {
                    Some(close) => close.chain(open),
                    None => open,
                };
            }
            Message::FetchWeather => {
                return self.start_fetch();
            }
            Message::WeatherFetched(generation, result) => {
                if generation != self.fetch_generation {
                    return Task::none();
                }
                match *result {
                    Ok(result) => {
                        self.fetch_state = FetchState::Loaded;
                        self.last_updated = Some(std::time::Instant::now());

                        let old_config = self.config.clone();
                        let idx = self.config.active_location_index;
                        if let Some(loc) = self.config.locations.get_mut(idx) {
                            if let Some(grid) = result.cached_grid {
                                loc.cached_grid = Some(grid);
                            }
                            if loc.name.is_empty() && !result.forecast.location_name.is_empty() {
                                loc.name = result.forecast.location_name.clone();
                            }
                        }
                        self.location_names = self
                            .config
                            .locations
                            .iter()
                            .map(|l| l.name.clone())
                            .collect();

                        self.alerts = result.alerts;
                        // Keep expansion for alerts still present; drop ids that expired.
                        let live = self.alerts.list();
                        let groups = group_alerts(live);
                        self.expanded_alerts.retain(|k| {
                            live.iter().any(|a| a.key() == *k)
                                || groups.iter().any(|g| g.key() == *k)
                        });
                        self.observation = result.observation;
                        self.air_quality = result.air_quality;
                        self.hourly_offset = 0;
                        self.expanded_day = None;
                        self.forecast = Some(result.forecast);
                        if self.config != old_config {
                            config::save_config(&self.config_handle, &self.config);
                        }
                    }
                    Err(e) => {
                        self.fetch_state = FetchState::Error(e);
                    }
                }
            }
            Message::Tick(_) => {
                if self.config.active_location().is_some()
                    && !matches!(self.fetch_state, FetchState::Loading)
                {
                    return self.start_fetch();
                }
            }
            Message::SearchInput(input) => {
                if input.is_empty() {
                    self.search_results.clear();
                }
                self.search_input = input;
                self.search_done = false;
            }
            Message::SearchSubmit => {
                let query = self.search_input.clone();
                if !query.is_empty() {
                    self.searching = true;
                    self.search_error = None;
                    return Task::perform(geocoding::search_location(query), |result| {
                        cosmic::Action::App(Message::SearchResults(
                            result.map_err(|e| e.to_string()),
                        ))
                    });
                }
            }
            Message::SearchResults(Ok(results)) => {
                self.searching = false;
                self.search_error = None;
                self.search_done = true;
                self.search_results = results;
            }
            Message::SearchResults(Err(e)) => {
                self.searching = false;
                self.search_done = true;
                self.search_results.clear();
                self.search_error = Some(e);
            }
            Message::SelectLocation(i) => {
                if let Some(result) = self.search_results.get(i) {
                    let country_code = result.address.as_ref().and_then(|a| a.country_code.clone());
                    let location = SavedLocation {
                        name: short_location_name(&result.display_name),
                        lat: result.lat.clone(),
                        lon: result.lon.clone(),
                        cached_grid: None,
                        country_code,
                    };
                    self.config.locations.push(location);
                    self.config.active_location_index = self.config.locations.len() - 1;
                    self.location_names = self
                        .config
                        .locations
                        .iter()
                        .map(|l| l.name.clone())
                        .collect();
                    config::save_config(&self.config_handle, &self.config);
                    self.page = Page::Main;
                    self.search_results.clear();
                    self.search_input.clear();
                    self.search_done = false;
                    self.current_expanded = false;
                    return self.start_fetch();
                }
            }

            Message::ActivateLocation(idx) => {
                if idx < self.config.locations.len() && idx != self.config.active_location_index {
                    self.config.active_location_index = idx;
                    self.location_names = self
                        .config
                        .locations
                        .iter()
                        .map(|l| l.name.clone())
                        .collect();
                    config::save_config(&self.config_handle, &self.config);
                    self.page = Page::Main;
                    self.forecast = None;
                    self.observation = None;
                    self.air_quality = None;
                    self.current_expanded = false;
                    self.alerts = Alerts::default();
                    return self.start_fetch();
                }
            }
            Message::AddLocation => {
                self.page = Page::Locations;
                self.search_results.clear();
                self.search_input.clear();
                self.current_expanded = false;
                self.search_done = false;
                self.search_error = None;
            }
            Message::RemoveLocation(idx) => {
                if idx < self.config.locations.len() {
                    self.config.locations.remove(idx);

                    if self.config.locations.is_empty() {
                        self.config.active_location_index = 0;
                        self.forecast = None;
                        self.observation = None;
                        self.alerts = Alerts::default();
                        self.fetch_state = FetchState::Idle;
                        self.page = Page::Setup;
                    } else if idx == self.config.active_location_index {
                        self.config.active_location_index = 0;
                        self.forecast = None;
                        self.observation = None;
                        self.air_quality = None;
                        self.current_expanded = false;
                        self.alerts = Alerts::default();
                        self.location_names = self
                            .config
                            .locations
                            .iter()
                            .map(|l| l.name.clone())
                            .collect();
                        config::save_config(&self.config_handle, &self.config);
                        return self.start_fetch();
                    } else if idx < self.config.active_location_index {
                        self.config.active_location_index -= 1;
                    }

                    self.location_names = self
                        .config
                        .locations
                        .iter()
                        .map(|l| l.name.clone())
                        .collect();
                    config::save_config(&self.config_handle, &self.config);
                }
            }
            Message::HourlyPrev => {
                self.hourly_offset = self.hourly_offset.saturating_sub(HOURLY_PAGE_SIZE);
            }
            Message::HourlyNext => {
                let total = self
                    .forecast
                    .as_ref()
                    .map(|f| f.hourly_periods.len())
                    .unwrap_or(0);
                let max_offset = total.saturating_sub(HOURLY_PAGE_SIZE);
                self.hourly_offset = (self.hourly_offset + HOURLY_PAGE_SIZE).min(max_offset);
            }
            Message::ToggleCurrentMore => {
                self.current_expanded = !self.current_expanded;
            }
            Message::ToggleDay(idx) => {
                if self.expanded_day == Some(idx) {
                    self.expanded_day = None;
                } else {
                    self.expanded_day = Some(idx);
                }
            }
            Message::ToggleAlert(id) => {
                if !self.expanded_alerts.remove(&id) {
                    self.expanded_alerts.insert(id);
                }
            }
            Message::ToggleUnits => {
                self.config.use_fahrenheit = !self.config.use_fahrenheit;
                config::save_config(&self.config_handle, &self.config);
                if self.config.active_location().is_some() {
                    return self.start_fetch();
                }
            }
            Message::BackToMain => {
                self.page = if self.config.locations.is_empty() {
                    Page::Setup
                } else {
                    Page::Main
                };
                self.search_results.clear();
                self.search_input.clear();
                self.search_done = false;
                self.search_error = None;
            }
            Message::ConfigChanged(new_config) => {
                let location_changed =
                    active_identity(&self.config) != active_identity(&new_config);
                let units_changed = self.config.use_fahrenheit != new_config.use_fahrenheit;
                self.config = new_config;
                if (location_changed || units_changed) && self.config.active_location().is_some() {
                    return self.start_fetch();
                }
            }
            Message::OpenAbout => {
                self.page = Page::About;
            }
            Message::OpenUrl(url) => {
                return Task::perform(
                    async move {
                        let _ = tokio::process::Command::new("xdg-open").arg(url).spawn();
                    },
                    |_| cosmic::Action::App(Message::Ignore),
                );
            }
            Message::Ignore => {}
        }
        Task::none()
    }

    /// Bump the fetch generation, mark loading, and dispatch a fetch for the active
    /// location. Central choke point so every dispatch site stamps the generation
    fn start_fetch(&mut self) -> Task<Message> {
        self.fetch_generation = self.fetch_generation.wrapping_add(1);
        self.fetch_state = FetchState::Loading;
        fetch_weather_task(&self.config, self.fetch_generation)
    }
}

/// The fields that actually determine "which" weather to fetch. Excludes cached_grid/name
/// so back-filling the grid caches doesn't read as a location change (breaks the self-watch loop).
fn active_identity(cfg: &WhetherConfig) -> Option<(&str, &str)> {
    cfg.active_location()
        .map(|l| (l.lat.as_str(), l.lon.as_str()))
}

fn fetch_weather_task(config: &WhetherConfig, generation: u64) -> Task<Message> {
    if let Some(loc) = config.active_location() {
        let lat = loc.lat.clone();
        let lon = loc.lon.clone();
        let use_fahrenheit = config.use_fahrenheit;
        let country_code = loc.country_code.clone();
        let cached_grid = loc.cached_grid.clone();
        let name = loc.name.clone();

        // Single adapter entry point: weathervane + US-only NWS shim, all mapped
        // into WeatherResult inside backend::fetch_weather.
        Task::perform(
            backend::fetch_weather(lat, lon, use_fahrenheit, country_code, cached_grid, name),
            move |result| {
                cosmic::Action::App(Message::WeatherFetched(generation, Box::new(result)))
            },
        )
    } else {
        Task::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AlertSeverity, WeatherAlert};

    fn alert(id: &str) -> WeatherAlert {
        WeatherAlert {
            id: id.into(),
            description: "text".into(),
            expires: chrono::Utc::now(),
            area_desc: "A".into(),
            event: "Wind".into(),
            headline: String::new(),
            severity: AlertSeverity::Moderate,
        }
    }

    #[test]
    fn flyout_row_is_visible_only_when_expanded_present_and_on_main() {
        let mut m = AppModel::default();
        let a = alert("x");
        let key = a.key();
        m.page = Page::Main;
        m.alerts = Alerts::Local(vec![a]);
        m.expanded_alerts.insert(key.clone());
        assert!(m.flyout_row_visible(&key));

        m.expanded_alerts.clear();
        assert!(!m.flyout_row_visible(&key), "collapsed");
        m.expanded_alerts.insert(key.clone());

        m.page = Page::Locations;
        assert!(!m.flyout_row_visible(&key), "off the main page");
        m.page = Page::Main;

        m.alerts = Alerts::default();
        assert!(!m.flyout_row_visible(&key), "pruned or location changed");
    }
}
