use super::super::{MainWindow, scene, set_camera_properties};
use crate::app_state::AppState;
use crate::model::BodyId;
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
    register_day_change(ui, app_state, skin_cache, AppState::prev_day, true);
    register_day_change(ui, app_state, skin_cache, AppState::next_day, false);
}

fn register_day_change(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
    change_day: fn(&mut AppState),
    previous: bool,
) {
    let weak_ui = ui.as_weak();
    let state = Rc::clone(app_state);
    let cache = Rc::clone(skin_cache);
    let handler = move || {
        change_day(&mut state.borrow_mut());
        if let Some(ui) = weak_ui.upgrade() {
            let state = state.borrow();
            refresh_date_and_scene(&ui, &state, &cache);
        }
    };

    if previous {
        ui.on_prev_day(handler);
    } else {
        ui.on_next_day(handler);
    }
}

fn refresh_date_and_scene(ui: &MainWindow, state: &AppState, cache: &Rc<RefCell<SkinCache>>) {
    ui.set_date_text(state.date_text().into());
    scene::refresh_scene(ui, state, cache);
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
            if let Some(ui) = weak_tick.upgrade() {
                refresh_date_and_scene(&ui, &state, &cache_tick);
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
        let Some(target) = BodyId::from_name(name.trim()) else {
            return;
        };
        let mut state = state_center.borrow_mut();
        state.set_center(target);

        if let Some(ui) = weak_select_center.upgrade() {
            let camera = state.camera();
            ui.set_selected_center_name(target.name().into());
            set_camera_properties(&ui, camera);
            scene::refresh_scene(&ui, &state, &cache_select_center);
        }
    });
}
