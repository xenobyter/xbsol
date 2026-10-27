use super::super::{MainWindow, scene};
use crate::app_state::AppState;
use crate::horizon::{HorizonsDateRange, get_ephemerides_for_bodies};
use crate::model::EphemerisTable;
use crate::ui::scene::SkinCache;
use slint::ComponentHandle;
use std::cell::{Cell, RefCell};
use std::error::Error;
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
    let request_generation_load = Rc::new(RequestGeneration::default());
    ui.on_load_date_range(move |start, stop| {
        let request_id = request_generation_load.start();
        let requested_range = match HorizonsDateRange::from_iso_dates(start.as_str(), stop.as_str())
        {
            Ok(date_range) => date_range,
            Err(error) => {
                if let Some(ui) = weak_load.upgrade() {
                    ui.set_range_loading(false);
                    ui.set_range_status(error.into());
                }
                return;
            }
        };

        if *active_range_load.borrow() == requested_range {
            if let Some(ui) = weak_load.upgrade() {
                ui.set_range_loading(false);
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
        let request_generation = Rc::clone(&request_generation_load);
        let state = Rc::clone(&state_load);
        let skin_cache = Rc::clone(&cache_load);
        let active_range = Rc::clone(&active_range_load);

        if let Err(error) = slint::spawn_local(async move {
            let result = fetch_date_range(runtime, client, requested_range).await;

            // Ein inzwischen überholter Request darf den aktuellen UI-Zustand nicht ändern.
            if !request_generation.is_current(request_id) {
                return;
            }
            let Some(ui) = weak_ui.upgrade() else {
                return;
            };
            ui.set_range_loading(false);
            finish_date_range_load(
                &ui,
                &state,
                &skin_cache,
                &active_range,
                requested_range,
                result,
            );
        }) {
            ui.set_range_loading(false);
            ui.set_range_status(
                format!("Ladevorgang konnte nicht gestartet werden: {error}").into(),
            );
        }
    });
}

#[derive(Default)]
struct RequestGeneration(Cell<u64>);

impl RequestGeneration {
    fn start(&self) -> u64 {
        let request_id = self.0.get().wrapping_add(1);
        self.0.set(request_id);
        request_id
    }

    fn is_current(&self, request_id: u64) -> bool {
        self.0.get() == request_id
    }
}

#[derive(Debug, thiserror::Error)]
enum DateRangeLoadError {
    #[error("Horizons-Daten konnten nicht geladen werden: {0}")]
    Horizons(#[source] Box<dyn Error + Send + Sync>),
    #[error("Horizons-Ladevorgang ist fehlgeschlagen: {0}")]
    Worker(#[source] tokio::task::JoinError),
    #[error("Einstellungen konnten nicht gespeichert werden: {0}")]
    Settings(#[source] Box<dyn Error>),
}

async fn fetch_date_range(
    runtime: Handle,
    client: reqwest::Client,
    date_range: HorizonsDateRange,
) -> Result<EphemerisTable, DateRangeLoadError> {
    runtime
        .spawn(async move { get_ephemerides_for_bodies(&client, date_range).await })
        .await
        .map_err(DateRangeLoadError::Worker)?
        .map_err(DateRangeLoadError::Horizons)
}

fn finish_date_range_load(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
    active_range: &Rc<RefCell<HorizonsDateRange>>,
    requested_range: HorizonsDateRange,
    result: Result<EphemerisTable, DateRangeLoadError>,
) {
    let table = match result {
        Ok(table) => table,
        Err(error) => {
            ui.set_range_status(error.to_string().into());
            return;
        }
    };

    if let Err(error) = crate::settings::save_date_range(requested_range) {
        ui.set_range_status(DateRangeLoadError::Settings(error).to_string().into());
        return;
    }

    *active_range.borrow_mut() = requested_range;
    let mut app_state = app_state.borrow_mut();
    app_state.replace_day_index(table.build_day_index());
    ui.set_date_text(app_state.date_text().into());
    ui.set_range_status("Zeitraum geladen und gespeichert.".into());
    scene::refresh_scene(ui, &app_state, skin_cache);
}

#[cfg(test)]
mod tests {
    use super::RequestGeneration;

    #[test]
    fn newer_request_invalidates_older_request() {
        let generation = RequestGeneration::default();

        let older_request = generation.start();
        let newer_request = generation.start();

        assert!(!generation.is_current(older_request));
        assert!(generation.is_current(newer_request));
    }
}
