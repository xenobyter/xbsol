use crate::app_state::AppState;
use crate::model::EphemerisDayIndex;
use slint::{Color, Image, ModelRc, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

slint::include_modules!();

type SkinCache = HashMap<String, Option<Image>>;

pub fn run_ui(day_index: EphemerisDayIndex) -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let app_state = Rc::new(RefCell::new(AppState::new(day_index)));
    let skin_cache = Rc::new(RefCell::new(SkinCache::new()));

    {
        let state = app_state.borrow();
        ui.set_date_text(state.date_text().into());
        let camera = state.camera();
        ui.set_camera_azimuth_deg(camera.azimuth_deg);
        ui.set_camera_elevation_deg(camera.elevation_deg);
        ui.set_camera_distance_au(camera.distance_au);
        ui.set_camera_fov_deg(camera.fov_deg);
        ui.set_selected_center_name(state.selected_center().name().to_string().into());
        ui.set_auto_step_days(state.auto_step_days() as f32);
    }

    refresh_scene(&ui, &app_state.borrow(), &skin_cache);

    let weak_prev = ui.as_weak();
    let state_prev = Rc::clone(&app_state);
    let cache_prev = Rc::clone(&skin_cache);
    ui.on_prev_day(move || {
        state_prev.borrow_mut().prev_day();
        if let Some(ui) = weak_prev.upgrade() {
            ui.set_date_text(state_prev.borrow().date_text().into());
            refresh_scene(&ui, &state_prev.borrow(), &cache_prev);
        }
    });

    let weak_next = ui.as_weak();
    let state_next = Rc::clone(&app_state);
    let cache_next = Rc::clone(&skin_cache);
    ui.on_next_day(move || {
        state_next.borrow_mut().next_day();
        if let Some(ui) = weak_next.upgrade() {
            ui.set_date_text(state_next.borrow().date_text().into());
            refresh_scene(&ui, &state_next.borrow(), &cache_next);
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
    let cache_select_center = Rc::clone(&skin_cache);
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
            "Io" => crate::model::BodyId::Io,
            "Europa" => crate::model::BodyId::Europa,
            "Ganymede" => crate::model::BodyId::Ganymede,
            "Callisto" => crate::model::BodyId::Callisto,
            "Phobos" => crate::model::BodyId::Phobos,
            "Deimos" => crate::model::BodyId::Deimos,
            _ => crate::model::BodyId::Sun,
        };

        state.set_center(target);

        if let Some(ui) = weak_select_center.upgrade() {
            let camera = state.camera();
            ui.set_selected_center_name(target.name().to_string().into());
            ui.set_camera_azimuth_deg(camera.azimuth_deg);
            ui.set_camera_elevation_deg(camera.elevation_deg);
            ui.set_camera_distance_au(camera.distance_au);
            ui.set_camera_fov_deg(camera.fov_deg);
            refresh_scene(&ui, &state, &cache_select_center);
        }
    });

    let weak_tick = ui.as_weak();
    let state_tick = Rc::clone(&app_state);
    let cache_tick = Rc::clone(&skin_cache);
    ui.on_auto_tick(move || {
        if state_tick.borrow().is_auto_play() {
            let mut state = state_tick.borrow_mut();
            let dt_days = state.auto_step_days();
            state.advance_auto_step(dt_days);
            let current = state.date_text();
            if let Some(ui) = weak_tick.upgrade() {
                ui.set_date_text(current.into());
                refresh_scene(&ui, &state, &cache_tick);
            }
        }
    });

    let state_step = Rc::clone(&app_state);
    ui.on_set_auto_step_days(move |days| {
        state_step.borrow_mut().set_auto_step_days(days as f64);
    });

    let weak_resize = ui.as_weak();
    let state_resize = Rc::clone(&app_state);
    let cache_resize = Rc::clone(&skin_cache);
    ui.on_scene_resized(move |_viewport_width_px, _viewport_height_px| {
        if let Some(ui) = weak_resize.upgrade() {
            refresh_scene(&ui, &state_resize.borrow(), &cache_resize);
        }
    });

    let weak_orbit = ui.as_weak();
    let state_orbit = Rc::clone(&app_state);
    let cache_orbit = Rc::clone(&skin_cache);
    ui.on_orbit_camera(move |delta_x_px, delta_y_px| {
        let mut state = state_orbit.borrow_mut();
        state.orbit_by_pixels(delta_x_px, delta_y_px);

        if let Some(ui) = weak_orbit.upgrade() {
            let camera = state.camera();
            ui.set_camera_azimuth_deg(camera.azimuth_deg);
            ui.set_camera_elevation_deg(camera.elevation_deg);
            ui.set_camera_distance_au(camera.distance_au);
            ui.set_camera_fov_deg(camera.fov_deg);
            refresh_scene(&ui, &state, &cache_orbit);
        }
    });

    let weak_zoom = ui.as_weak();
    let state_zoom = Rc::clone(&app_state);
    let cache_zoom = Rc::clone(&skin_cache);
    ui.on_zoom_camera(move |delta_steps| {
        let mut state = state_zoom.borrow_mut();
        state.zoom_by_steps(delta_steps);

        if let Some(ui) = weak_zoom.upgrade() {
            let camera = state.camera();
            ui.set_camera_azimuth_deg(camera.azimuth_deg);
            ui.set_camera_elevation_deg(camera.elevation_deg);
            ui.set_camera_distance_au(camera.distance_au);
            ui.set_camera_fov_deg(camera.fov_deg);
            refresh_scene(&ui, &state, &cache_zoom);
        }
    });

    let weak_pan = ui.as_weak();
    let state_pan = Rc::clone(&app_state);
    let cache_pan = Rc::clone(&skin_cache);
    ui.on_pan_camera(move |delta_x_px, delta_y_px, viewport_width_px, viewport_height_px| {
        let mut state = state_pan.borrow_mut();
        state.pan_by_pixels(delta_x_px, delta_y_px, viewport_width_px, viewport_height_px);

        if let Some(ui) = weak_pan.upgrade() {
            refresh_scene(&ui, &state, &cache_pan);
        }
    });

    let weak_key = ui.as_weak();
    let state_key = Rc::clone(&app_state);
    let cache_key = Rc::clone(&skin_cache);
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
            refresh_scene(&ui, &state, &cache_key);
        }
    });

    ui.run()
}

fn refresh_scene(ui: &MainWindow, state: &AppState, skin_cache: &Rc<RefCell<SkinCache>>) {
    let width = ui.get_scene_width().max(1.0);
    let height = ui.get_scene_height().max(1.0);
    let frame = state.build_scene_frame(width, height);

    let sprites: Vec<BodySprite> = frame
        .bodies
        .into_iter()
        .map(|body| {
            let tier = skin_tier_for_radius(body.radius_px);
            let texture_opt = resolve_skin_image(&mut skin_cache.borrow_mut(), &body.sprite_stem, tier);
            let has_texture = texture_opt.is_some();
            let texture = texture_opt.unwrap_or_default();

            BodySprite {
                name: body.name.into(),
                texture,
                has_texture,
                x_px: body.x_px,
                y_px: body.y_px,
                radius_px: body.radius_px,
                color: Color::from_rgb_u8(body.color_rgb.0, body.color_rgb.1, body.color_rgb.2),
            }
        })
        .collect();

    let model = Rc::new(VecModel::from(sprites));
    ui.set_body_sprites(ModelRc::from(model));
}

fn skin_tier_for_radius(radius_px: f32) -> &'static str {
    if radius_px < 8.0 {
        "low"
    } else if radius_px < 20.0 {
        "medium"
    } else {
        "high"
    }
}

fn resolve_skin_image(cache: &mut SkinCache, stem: &str, tier: &str) -> Option<Image> {
    let cache_key = format!("{}:{}", stem, tier);
    if let Some(image) = cache.get(&cache_key) {
        return image.clone();
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exts = ["svg", "png", "webp", "gif"];
    let mut candidates = Vec::new();

    for ext in exts {
        candidates.push(manifest_dir.join(format!("assets/skins/{}_{}.{}", stem, tier, ext)));
    }
    for ext in exts {
        candidates.push(manifest_dir.join(format!("assets/skins/{}.{}", stem, ext)));
    }

    for path in candidates {
        if path.exists() {
            match Image::load_from_path(&path) {
                Ok(img) => {
                    cache.insert(cache_key.clone(), Some(img.clone()));
                    return Some(img);
                }
                Err(error) => {
                    log::warn!("Skin konnte nicht geladen werden ({}): {}", stem, error);
                }
            }
        }
    }

    cache.insert(cache_key, None);
    None
}
