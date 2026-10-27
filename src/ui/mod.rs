//! Slint-Bootstrap: initialisiert das Hauptfenster und verbindet es mit App-State,
//! Szenendarstellung und UI-Handlern.

mod handlers;
mod scene;

use crate::app_state::AppState;
use crate::horizon::HorizonsDateRange;
use crate::model::EphemerisDayIndex;
use scene::SkinCache;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::Handle;

slint::include_modules!();

/// Initialisiert das Hauptfenster und startet die Slint-Ereignisschleife.
///
/// Der vorberechnete Tagesindex bildet den Startzustand. `date_range` wird im
/// Fenster angezeigt und dient als Vergleichswert für spätere Bereichswechsel.
/// HTTP-Client und Tokio-Handle werden von den registrierten Handlern für
/// asynchrone Horizons-Abfragen weiterverwendet.
///
/// # Errors
///
/// Gibt Plattformfehler zurück, wenn das Hauptfenster nicht erstellt oder die
/// Ereignisschleife nicht gestartet werden kann.
pub fn run_ui(
    day_index: EphemerisDayIndex,
    date_range: HorizonsDateRange,
    client: reqwest::Client,
    runtime: Handle,
) -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let app_state = Rc::new(RefCell::new(AppState::new(day_index)));
    let skin_cache = Rc::new(RefCell::new(SkinCache::new()));
    let active_range = Rc::new(RefCell::new(date_range));

    initialize_window(&ui, &app_state.borrow(), date_range);
    scene::refresh_scene(&ui, &app_state.borrow(), &skin_cache);
    handlers::register_all(
        &ui,
        &app_state,
        &skin_cache,
        &active_range,
        &client,
        &runtime,
    );

    ui.run()
}

fn initialize_window(ui: &MainWindow, state: &AppState, date_range: HorizonsDateRange) {
    ui.set_start_date(date_range.start_time().into());
    ui.set_stop_date(date_range.stop_time().into());

    ui.set_date_text(state.date_text().into());
    set_camera_properties(ui, state.camera());
    ui.set_selected_center_name(state.selected_center().name().into());
    ui.set_auto_step_days(state.auto_step_days() as f32);
    ui.set_show_trails(state.show_trails());
}

pub(super) fn set_camera_properties(ui: &MainWindow, camera: crate::app_state::CameraState) {
    ui.set_camera_azimuth_deg(camera.azimuth_deg);
    ui.set_camera_elevation_deg(camera.elevation_deg);
    ui.set_camera_distance_au(camera.distance_au);
    ui.set_camera_fov_deg(camera.fov_deg);
}
