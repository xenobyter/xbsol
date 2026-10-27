use super::super::{MainWindow, scene, set_camera_properties};
use crate::app_state::AppState;
use crate::ui::scene::SkinCache;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

pub(super) fn register(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    register_day_navigation(ui, app_state, skin_cache);
    register_playback(ui, app_state, skin_cache);
    register_center_selection(ui, app_state, skin_cache);
}

fn register_day_navigation(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let weak_prev = ui.as_weak();
    let state_prev = Rc::clone(app_state);
    let cache_prev = Rc::clone(skin_cache);
    ui.on_prev_day(move || {
        state_prev.borrow_mut().prev_day();
        if let Some(ui) = weak_prev.upgrade() {
            ui.set_date_text(state_prev.borrow().date_text().into());
            scene::refresh_scene(&ui, &state_prev.borrow(), &cache_prev);
        }
    });

    let weak_next = ui.as_weak();
    let state_next = Rc::clone(app_state);
    let cache_next = Rc::clone(skin_cache);
    ui.on_next_day(move || {
        state_next.borrow_mut().next_day();
        if let Some(ui) = weak_next.upgrade() {
            ui.set_date_text(state_next.borrow().date_text().into());
            scene::refresh_scene(&ui, &state_next.borrow(), &cache_next);
        }
    });
}

fn register_playback(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let weak_auto = ui.as_weak();
    let state_auto = Rc::clone(app_state);
    ui.on_toggle_auto_play(move || {
        state_auto.borrow_mut().toggle_auto_play();
        if let Some(ui) = weak_auto.upgrade() {
            ui.set_auto_play(state_auto.borrow().is_auto_play());
        }
    });

    let weak_tick = ui.as_weak();
    let state_tick = Rc::clone(app_state);
    let cache_tick = Rc::clone(skin_cache);
    ui.on_auto_tick(move || {
        if state_tick.borrow().is_auto_play() {
            let mut state = state_tick.borrow_mut();
            let dt_days = state.auto_step_days();
            state.advance_auto_step(dt_days);
            let current = state.date_text();
            if let Some(ui) = weak_tick.upgrade() {
                ui.set_date_text(current.into());
                scene::refresh_scene(&ui, &state, &cache_tick);
            }
        }
    });

    let state_step = Rc::clone(app_state);
    ui.on_set_auto_step_days(move |days| {
        state_step.borrow_mut().set_auto_step_days(days as f64);
    });
}

fn register_center_selection(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let weak_select_center = ui.as_weak();
    let state_center = Rc::clone(app_state);
    let cache_select_center = Rc::clone(skin_cache);
    ui.on_select_center(move |name| {
        let mut state = state_center.borrow_mut();
        let body_name = name.trim();
        let target = match body_name {
            "Sun" => crate::model::BodyId::Sun,
            "Mercury" => crate::model::BodyId::Mercury,
            "Venus" => crate::model::BodyId::Venus,
            "Earth" => crate::model::BodyId::Earth,
            "Moon" => crate::model::BodyId::Moon,
            "Mars" => crate::model::BodyId::Mars,
            "Jupiter" => crate::model::BodyId::Jupiter,
            "Saturn" => crate::model::BodyId::Saturn,
            "Uranus" => crate::model::BodyId::Uranus,
            "Neptune" => crate::model::BodyId::Neptune,
            "Io" => crate::model::BodyId::Io,
            "Europa" => crate::model::BodyId::Europa,
            "Ganymede" => crate::model::BodyId::Ganymede,
            "Callisto" => crate::model::BodyId::Callisto,
            "Mimas" => crate::model::BodyId::Mimas,
            "Enceladus" => crate::model::BodyId::Enceladus,
            "Tethys" => crate::model::BodyId::Tethys,
            "Dione" => crate::model::BodyId::Dione,
            "Rhea" => crate::model::BodyId::Rhea,
            "Titan" => crate::model::BodyId::Titan,
            "Iapetus" => crate::model::BodyId::Iapetus,
            "Miranda" => crate::model::BodyId::Miranda,
            "Ariel" => crate::model::BodyId::Ariel,
            "Umbriel" => crate::model::BodyId::Umbriel,
            "Titania" => crate::model::BodyId::Titania,
            "Oberon" => crate::model::BodyId::Oberon,
            "Triton" => crate::model::BodyId::Triton,
            "Nereid" => crate::model::BodyId::Nereid,
            "Proteus" => crate::model::BodyId::Proteus,
            "Phobos" => crate::model::BodyId::Phobos,
            "Deimos" => crate::model::BodyId::Deimos,
            _ => crate::model::BodyId::Sun,
        };

        state.set_center(target);

        if let Some(ui) = weak_select_center.upgrade() {
            let camera = state.camera();
            ui.set_selected_center_name(target.name().to_string().into());
            set_camera_properties(&ui, camera);
            scene::refresh_scene(&ui, &state, &cache_select_center);
        }
    });
}
