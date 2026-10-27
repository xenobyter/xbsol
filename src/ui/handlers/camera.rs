use super::super::{MainWindow, scene, set_camera_properties};
use crate::app_state::AppState;
use crate::ui::scene::SkinCache;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

const KEYBOARD_PAN_STEP_PX: f32 = 28.0;
const KEYBOARD_ORBIT_HORIZONTAL_DEG: f32 = 2.5;
const KEYBOARD_ORBIT_VERTICAL_DEG: f32 = 2.0;
const KEYBOARD_ZOOM_STEP: f32 = 1.0;

pub(super) fn register(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    register_scene_controls(ui, app_state, skin_cache);
    register_pointer_camera_controls(ui, app_state, skin_cache);
    register_keyboard_camera_control(ui, app_state, skin_cache);
}

fn register_scene_controls(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let weak_trails = ui.as_weak();
    let state_trails = Rc::clone(app_state);
    let cache_trails = Rc::clone(skin_cache);
    ui.on_toggle_trails(move || {
        let mut state = state_trails.borrow_mut();
        state.toggle_trails();

        if let Some(ui) = weak_trails.upgrade() {
            ui.set_show_trails(state.show_trails());
            scene::refresh_scene(&ui, &state, &cache_trails);
        }
    });

    let weak_resize = ui.as_weak();
    let state_resize = Rc::clone(app_state);
    let cache_resize = Rc::clone(skin_cache);
    ui.on_scene_resized(move |_viewport_width_px, _viewport_height_px| {
        if let Some(ui) = weak_resize.upgrade() {
            scene::refresh_scene(&ui, &state_resize.borrow(), &cache_resize);
        }
    });
}

fn register_pointer_camera_controls(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let weak_orbit = ui.as_weak();
    let state_orbit = Rc::clone(app_state);
    let cache_orbit = Rc::clone(skin_cache);
    ui.on_orbit_camera(move |delta_x_px, delta_y_px| {
        let mut state = state_orbit.borrow_mut();
        state.orbit_by_pixels(delta_x_px, delta_y_px);

        if let Some(ui) = weak_orbit.upgrade() {
            set_camera_properties(&ui, state.camera());
            scene::refresh_scene(&ui, &state, &cache_orbit);
        }
    });

    let weak_zoom = ui.as_weak();
    let state_zoom = Rc::clone(app_state);
    let cache_zoom = Rc::clone(skin_cache);
    ui.on_zoom_camera(move |delta_steps| {
        let mut state = state_zoom.borrow_mut();
        state.zoom_by_steps(delta_steps);

        if let Some(ui) = weak_zoom.upgrade() {
            set_camera_properties(&ui, state.camera());
            scene::refresh_scene(&ui, &state, &cache_zoom);
        }
    });

    let weak_pan = ui.as_weak();
    let state_pan = Rc::clone(app_state);
    let cache_pan = Rc::clone(skin_cache);
    ui.on_pan_camera(
        move |delta_x_px, delta_y_px, viewport_width_px, viewport_height_px| {
            let mut state = state_pan.borrow_mut();
            state.pan_by_pixels(
                delta_x_px,
                delta_y_px,
                viewport_width_px,
                viewport_height_px,
            );

            if let Some(ui) = weak_pan.upgrade() {
                scene::refresh_scene(&ui, &state, &cache_pan);
            }
        },
    );
}

fn register_keyboard_camera_control(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let weak_key = ui.as_weak();
    let state_key = Rc::clone(app_state);
    let cache_key = Rc::clone(skin_cache);
    ui.on_camera_key(move |action| {
        let mut state = state_key.borrow_mut();

        if let Some(ui) = weak_key.upgrade() {
            let width = ui.get_scene_width().max(1.0);
            let height = ui.get_scene_height().max(1.0);

            apply_camera_action(&mut state, action.as_str(), width, height);

            set_camera_properties(&ui, state.camera());
            scene::refresh_scene(&ui, &state, &cache_key);
        }
    });
}

fn apply_camera_action(state: &mut AppState, action: &str, width: f32, height: f32) {
    match action {
        "pan_left" => state.pan_by_pixels(-KEYBOARD_PAN_STEP_PX, 0.0, width, height),
        "pan_right" => state.pan_by_pixels(KEYBOARD_PAN_STEP_PX, 0.0, width, height),
        "pan_up" => state.pan_by_pixels(0.0, -KEYBOARD_PAN_STEP_PX, width, height),
        "pan_down" => state.pan_by_pixels(0.0, KEYBOARD_PAN_STEP_PX, width, height),
        "orbit_left" => state.orbit_by_degrees(-KEYBOARD_ORBIT_HORIZONTAL_DEG, 0.0),
        "orbit_right" => state.orbit_by_degrees(KEYBOARD_ORBIT_HORIZONTAL_DEG, 0.0),
        "orbit_up" => state.orbit_by_degrees(0.0, -KEYBOARD_ORBIT_VERTICAL_DEG),
        "orbit_down" => state.orbit_by_degrees(0.0, KEYBOARD_ORBIT_VERTICAL_DEG),
        "zoom_in" => state.zoom_by_steps(KEYBOARD_ZOOM_STEP),
        "zoom_out" => state.zoom_by_steps(-KEYBOARD_ZOOM_STEP),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, apply_camera_action};
    use crate::model::EphemerisDayIndex;

    #[test]
    fn applies_camera_keyboard_actions() {
        let mut state = AppState::new(EphemerisDayIndex::default());

        apply_camera_action(&mut state, "orbit_left", 800.0, 600.0);
        assert_eq!(state.camera().azimuth_deg, 132.5);

        apply_camera_action(&mut state, "zoom_in", 800.0, 600.0);
        assert_eq!(state.camera().distance_au, 4.5 / 1.12);
    }

    #[test]
    fn ignores_unknown_camera_keyboard_action() {
        let mut state = AppState::new(EphemerisDayIndex::default());
        let camera_before = state.camera();

        apply_camera_action(&mut state, "unknown", 800.0, 600.0);

        let camera_after = state.camera();
        assert_eq!(camera_after.azimuth_deg, camera_before.azimuth_deg);
        assert_eq!(camera_after.elevation_deg, camera_before.elevation_deg);
        assert_eq!(camera_after.distance_au, camera_before.distance_au);
        assert_eq!(camera_after.fov_deg, camera_before.fov_deg);
    }
}
