use crate::app_state::AppState;
use crate::model::EphemerisDayIndex;
use slint::{Color, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

const AUTO_STEP_DAYS: f64 = 0.25;

pub fn run_ui(day_index: EphemerisDayIndex) -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let app_state = Rc::new(RefCell::new(AppState::new(day_index)));

    {
        let state = app_state.borrow();
        ui.set_date_text(state.date_text().into());
        let camera = state.camera();
        ui.set_camera_azimuth_deg(camera.azimuth_deg);
        ui.set_camera_elevation_deg(camera.elevation_deg);
        ui.set_camera_distance_au(camera.distance_au);
        ui.set_camera_fov_deg(camera.fov_deg);
        ui.set_selected_center_name(state.selected_center().name().to_string().into());
    }

    refresh_scene(&ui, &app_state.borrow());

    let weak_prev = ui.as_weak();
    let state_prev = Rc::clone(&app_state);
    ui.on_prev_day(move || {
        state_prev.borrow_mut().prev_day();
        if let Some(ui) = weak_prev.upgrade() {
            ui.set_date_text(state_prev.borrow().date_text().into());
            refresh_scene(&ui, &state_prev.borrow());
        }
    });

    let weak_next = ui.as_weak();
    let state_next = Rc::clone(&app_state);
    ui.on_next_day(move || {
        state_next.borrow_mut().next_day();
        if let Some(ui) = weak_next.upgrade() {
            ui.set_date_text(state_next.borrow().date_text().into());
            refresh_scene(&ui, &state_next.borrow());
        }
    });

    let weak_auto = ui.as_weak();
    let state_auto = Rc::clone(&app_state);
    ui.on_toggle_auto_play(move || {
        state_auto.borrow_mut().toggle_auto_play();
        if let Some(ui) = weak_auto.upgrade() {
            ui.set_auto_play(state_auto.borrow().is_auto_play());
        }
    });

    let weak_select_center = ui.as_weak();
    let state_center = Rc::clone(&app_state);
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
            _ => crate::model::BodyId::Sun,
        };

        state.set_center(target);

        if let Some(ui) = weak_select_center.upgrade() {
            ui.set_selected_center_name(target.name().to_string().into());
            refresh_scene(&ui, &state);
        }
    });

    let weak_tick = ui.as_weak();
    let state_tick = Rc::clone(&app_state);
    ui.on_auto_tick(move || {
        if state_tick.borrow().is_auto_play() {
            let mut state = state_tick.borrow_mut();
            state.advance_auto_step(AUTO_STEP_DAYS);
            let current = state.date_text();
            if let Some(ui) = weak_tick.upgrade() {
                ui.set_date_text(current.into());
                refresh_scene(&ui, &state);
            }
        }
    });

    let weak_resize = ui.as_weak();
    let state_resize = Rc::clone(&app_state);
    ui.on_scene_resized(move |_viewport_width_px, _viewport_height_px| {
        if let Some(ui) = weak_resize.upgrade() {
            refresh_scene(&ui, &state_resize.borrow());
        }
    });

    let weak_orbit = ui.as_weak();
    let state_orbit = Rc::clone(&app_state);
    ui.on_orbit_camera(move |delta_x_px, delta_y_px| {
        let mut state = state_orbit.borrow_mut();
        state.orbit_by_pixels(delta_x_px, delta_y_px);

        if let Some(ui) = weak_orbit.upgrade() {
            let camera = state.camera();
            ui.set_camera_azimuth_deg(camera.azimuth_deg);
            ui.set_camera_elevation_deg(camera.elevation_deg);
            ui.set_camera_distance_au(camera.distance_au);
            ui.set_camera_fov_deg(camera.fov_deg);
            refresh_scene(&ui, &state);
        }
    });

    let weak_zoom = ui.as_weak();
    let state_zoom = Rc::clone(&app_state);
    ui.on_zoom_camera(move |delta_steps| {
        let mut state = state_zoom.borrow_mut();
        state.zoom_by_steps(delta_steps);

        if let Some(ui) = weak_zoom.upgrade() {
            let camera = state.camera();
            ui.set_camera_azimuth_deg(camera.azimuth_deg);
            ui.set_camera_elevation_deg(camera.elevation_deg);
            ui.set_camera_distance_au(camera.distance_au);
            ui.set_camera_fov_deg(camera.fov_deg);
            refresh_scene(&ui, &state);
        }
    });

    let weak_pan = ui.as_weak();
    let state_pan = Rc::clone(&app_state);
    ui.on_pan_camera(move |delta_x_px, delta_y_px, viewport_width_px, viewport_height_px| {
        let mut state = state_pan.borrow_mut();
        state.pan_by_pixels(delta_x_px, delta_y_px, viewport_width_px, viewport_height_px);

        if let Some(ui) = weak_pan.upgrade() {
            refresh_scene(&ui, &state);
        }
    });

    let weak_key = ui.as_weak();
    let state_key = Rc::clone(&app_state);
    ui.on_camera_key(move |action| {
        let mut state = state_key.borrow_mut();

        if let Some(ui) = weak_key.upgrade() {
            let width = ui.get_scene_width().max(1.0);
            let height = ui.get_scene_height().max(1.0);

            match action.as_str() {
                "pan_left" => state.pan_by_pixels(-28.0, 0.0, width, height),
                "pan_right" => state.pan_by_pixels(28.0, 0.0, width, height),
                "pan_up" => state.pan_by_pixels(0.0, -28.0, width, height),
                "pan_down" => state.pan_by_pixels(0.0, 28.0, width, height),
                "orbit_left" => state.orbit_by_degrees(-2.5, 0.0),
                "orbit_right" => state.orbit_by_degrees(2.5, 0.0),
                "orbit_up" => state.orbit_by_degrees(0.0, -2.0),
                "orbit_down" => state.orbit_by_degrees(0.0, 2.0),
                "zoom_in" => state.zoom_by_steps(1.0),
                "zoom_out" => state.zoom_by_steps(-1.0),
                _ => {}
            }

            let camera = state.camera();
            ui.set_camera_azimuth_deg(camera.azimuth_deg);
            ui.set_camera_elevation_deg(camera.elevation_deg);
            ui.set_camera_distance_au(camera.distance_au);
            ui.set_camera_fov_deg(camera.fov_deg);
            refresh_scene(&ui, &state);
        }
    });

    ui.run()
}

fn refresh_scene(ui: &MainWindow, state: &AppState) {
    let width = ui.get_scene_width().max(1.0);
    let height = ui.get_scene_height().max(1.0);
    let frame = state.build_scene_frame(width, height);

    let sprites: Vec<BodySprite> = frame
        .bodies
        .into_iter()
        .map(|body| BodySprite {
            name: body.name.into(),
            x_px: body.x_px,
            y_px: body.y_px,
            radius_px: body.radius_px,
            color: Color::from_rgb_u8(body.color_rgb.0, body.color_rgb.1, body.color_rgb.2),
        })
        .collect();

    let model = Rc::new(VecModel::from(sprites));
    ui.set_body_sprites(ModelRc::from(model));
}
