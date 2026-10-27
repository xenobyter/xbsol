use super::super::{MainWindow, scene};
use crate::app_state::AppState;
use crate::horizon::{HorizonsDateRange, get_ephemerides_for_bodies};
use crate::ui::scene::SkinCache;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::Handle;

pub(super) fn register(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
    active_range: &Rc<RefCell<HorizonsDateRange>>,
    client: &reqwest::Client,
    runtime: &Handle,
) {
    let weak_load = ui.as_weak();
    let state_load = Rc::clone(app_state);
    let cache_load = Rc::clone(skin_cache);
    let active_range_load = Rc::clone(active_range);
    let client = client.clone();
    let runtime = runtime.clone();
    ui.on_load_date_range(move |start, stop| {
        let requested_range = match HorizonsDateRange::from_iso_dates(start.as_str(), stop.as_str())
        {
            Ok(date_range) => date_range,
            Err(error) => {
                if let Some(ui) = weak_load.upgrade() {
                    ui.set_range_status(error.into());
                }
                return;
            }
        };

        if *active_range_load.borrow() == requested_range {
            if let Some(ui) = weak_load.upgrade() {
                ui.set_range_status("Der Zeitraum ist unverändert.".into());
            }
            return;
        }

        let Some(ui) = weak_load.upgrade() else {
            return;
        };
        ui.set_range_loading(true);
        ui.set_range_status("Horizons-Daten werden geladen...".into());

        let runtime = runtime.clone();
        let client = client.clone();
        let weak_ui = ui.as_weak();
        let state = Rc::clone(&state_load);
        let skin_cache = Rc::clone(&cache_load);
        let active_range = Rc::clone(&active_range_load);

        if let Err(error) = slint::spawn_local(async move {
            let result = runtime
                .spawn(async move { get_ephemerides_for_bodies(&client, requested_range).await })
                .await;
            let result = match result {
                Ok(Ok(table)) => Ok(table),
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };

            let Some(ui) = weak_ui.upgrade() else {
                return;
            };
            ui.set_range_loading(false);

            match result {
                Ok(table) => match crate::settings::save_date_range(requested_range) {
                    Ok(()) => {
                        *active_range.borrow_mut() = requested_range;
                        let mut app_state = state.borrow_mut();
                        app_state.replace_day_index(table.build_day_index());
                        ui.set_date_text(app_state.date_text().into());
                        ui.set_range_status("Zeitraum geladen und gespeichert.".into());
                        scene::refresh_scene(&ui, &app_state, &skin_cache);
                    }
                    Err(error) => ui.set_range_status(
                        format!("Einstellungen konnten nicht gespeichert werden: {error}").into(),
                    ),
                },
                Err(error) => ui.set_range_status(
                    format!("Horizons-Daten konnten nicht geladen werden: {error}").into(),
                ),
            }
        }) {
            ui.set_range_loading(false);
            ui.set_range_status(
                format!("Ladevorgang konnte nicht gestartet werden: {error}").into(),
            );
        }
    });
}
