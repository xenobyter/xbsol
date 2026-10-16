use crate::model::{BodyId, EphemerisDayIndex, StateVector};

const AU_KM: f64 = 149_597_870.7;
const DAY_PROGRESS_RESET: f64 = 0.0;
const DAY_PROGRESS_THRESHOLD: f64 = 1.0;
const CLIP_NEAR_AU: f64 = 0.001;
const CLIP_FAR_AU: f64 = 12.0;
const MIN_ASPECT: f64 = 1e-6;
const NDC_TO_PIXEL_SCALE: f64 = 0.5;
const BODY_RADIUS_SCALE_PX: f64 = 260_000.0;
const MIN_DEPTH_FOR_RADIUS: f64 = 0.2;

#[derive(Debug, Clone)]
pub struct RenderBody {
    pub name: String,
    pub x_px: f32,
    pub y_px: f32,
    pub radius_px: f32,
    pub depth: f32,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone)]
pub struct SceneFrame {
    pub bodies: Vec<RenderBody>,
}

#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    pub azimuth_deg: f32,
    pub elevation_deg: f32,
    pub distance_au: f32,
    pub fov_deg: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            // Start mit deutlich erhöhter Perspektive auf die Ekliptik,
            // damit die Planetenebene "von oben" besser lesbar ist.
            azimuth_deg: DEFAULT_AZIMUTH_DEG,
            elevation_deg: DEFAULT_ELEVATION_DEG,
            distance_au: DEFAULT_DISTANCE_AU,
            fov_deg: DEFAULT_FOV_DEG,
        }
    }
}

const DEFAULT_AZIMUTH_DEG: f32 = 135.0;
const DEFAULT_ELEVATION_DEG: f32 = 58.0;
const DEFAULT_DISTANCE_AU: f32 = 4.5;
const DEFAULT_FOV_DEG: f32 = 50.0;
const MERCURY_DISTANCE_AU: f32 = 1.2;
const VENUS_DISTANCE_AU: f32 = 1.2;
const EARTH_DISTANCE_AU: f32 = 0.015;
const MOON_DISTANCE_AU: f32 = 0.12;
const MARS_DISTANCE_AU: f32 = 0.0015;
const MARS_FOV_DEG: f32 = 35.0;
const MIN_ELEVATION_DEG: f32 = -85.0;
const MAX_ELEVATION_DEG: f32 = 85.0;
const MIN_DISTANCE_AU: f32 = 0.0015;
const MAX_DISTANCE_AU: f32 = 12.0;
const ORBIT_DEG_PER_PIXEL: f32 = 0.25;
const ZOOM_FACTOR_PER_STEP: f32 = 1.12;
const MIN_RADIUS_ZOOM_SCALE: f64 = 0.35;
const MAX_RADIUS_ZOOM_SCALE: f64 = 6.0;

#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Copy)]
struct CameraBasis {
    camera_pos: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
}

#[derive(Debug, Clone, Copy)]
struct ProjectionParams {
    near: f64,
    far: f64,
    tan_half_fov: f64,
    aspect: f64,
}

impl Vec3 {
    fn dot(self, rhs: Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    fn cross(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }

    fn normalize(self) -> Option<Vec3> {
        let n = self.norm();
        if n <= f64::EPSILON {
            return None;
        }
        Some(Vec3 {
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
        })
    }
}

use std::ops::{Add, Sub};

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Vec3 {
    fn scale(self, s: f64) -> Vec3 {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

pub struct AppState {
    day_index: EphemerisDayIndex,
    camera: CameraState,
    camera_target_au: Vec3,
    auto_play: bool,
    current_day_progress: f64,
    selected_center: BodyId,
}

impl AppState {
    fn max_visible_radius_px(body_id: BodyId) -> f64 {
        match body_id {
            BodyId::Sun => 180.0,
            BodyId::Mercury => 14.0,
            BodyId::Venus => 34.0,
            BodyId::Earth => 42.0,
            BodyId::Moon => 14.0,
            BodyId::Mars => 24.0,
            BodyId::Phobos => 4.0,
            BodyId::Deimos => 4.0,
        }
    }

    pub fn new(day_index: EphemerisDayIndex) -> Self {
        Self {
            day_index,
            camera: CameraState::default(),
            camera_target_au: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            auto_play: false,
            current_day_progress: DAY_PROGRESS_RESET,
            selected_center: BodyId::Sun,
        }
    }

    pub fn selected_center(&self) -> BodyId {
        self.selected_center
    }

    pub fn set_center(&mut self, body_id: BodyId) {
        if self.selected_center == body_id {
            return;
        }

        self.selected_center = body_id;
        self.camera_target_au = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        self.camera = Self::preset_camera_for_center(body_id);
    }

    fn preset_camera_for_center(body_id: BodyId) -> CameraState {
        let distance_au = match body_id {
            BodyId::Sun => DEFAULT_DISTANCE_AU,
            BodyId::Mercury => MERCURY_DISTANCE_AU,
            BodyId::Venus => VENUS_DISTANCE_AU,
            BodyId::Earth => EARTH_DISTANCE_AU,
            BodyId::Moon => MOON_DISTANCE_AU,
            BodyId::Mars => MARS_DISTANCE_AU,
            BodyId::Phobos | BodyId::Deimos => MARS_DISTANCE_AU,
        };

        let fov_deg = match body_id {
            BodyId::Mars | BodyId::Phobos | BodyId::Deimos => MARS_FOV_DEG,
            _ => DEFAULT_FOV_DEG,
        };

        CameraState {
            azimuth_deg: DEFAULT_AZIMUTH_DEG,
            elevation_deg: DEFAULT_ELEVATION_DEG,
            distance_au,
            fov_deg,
        }
    }

    pub fn date_text(&self) -> String {
        match self.day_index.current_day() {
            Some(day) => format!("{}  |  Bodies: {}", day.date, day.by_body.len()),
            None => "No ephemeris data available".to_string(),
        }
    }

    pub fn prev_day(&mut self) {
        self.current_day_progress = DAY_PROGRESS_RESET;
        let _ = self.day_index.prev_day();
    }

    pub fn next_day(&mut self) {
        self.current_day_progress = DAY_PROGRESS_RESET;
        let _ = self.day_index.next_day();
    }

    pub fn toggle_auto_play(&mut self) {
        self.auto_play = !self.auto_play;
    }

    pub fn is_auto_play(&self) -> bool {
        self.auto_play
    }

    pub fn advance_auto_step(&mut self, dt_days: f64) {
        if self.day_index.is_empty() {
            return;
        }

        self.current_day_progress += dt_days;
        while self.current_day_progress >= DAY_PROGRESS_THRESHOLD {
            if self.day_index.current_day_idx + 1 < self.day_index.days.len() {
                self.day_index.current_day_idx += 1;
                self.current_day_progress -= DAY_PROGRESS_THRESHOLD;
            } else {
                self.current_day_progress = DAY_PROGRESS_RESET;
                break;
            }
        }
    }

    pub fn camera(&self) -> CameraState {
        self.camera
    }

    pub fn orbit_by_pixels(&mut self, delta_x_px: f32, delta_y_px: f32) {
        self.orbit_by_degrees(-delta_x_px * ORBIT_DEG_PER_PIXEL, delta_y_px * ORBIT_DEG_PER_PIXEL);
    }

    pub fn orbit_by_degrees(&mut self, delta_azimuth_deg: f32, delta_elevation_deg: f32) {
        self.camera.azimuth_deg += delta_azimuth_deg;
        self.camera.elevation_deg = (self.camera.elevation_deg + delta_elevation_deg)
            .clamp(MIN_ELEVATION_DEG, MAX_ELEVATION_DEG);
    }

    pub fn zoom_by_steps(&mut self, steps: f32) {
        if steps.abs() <= f32::EPSILON {
            return;
        }

        let factor = ZOOM_FACTOR_PER_STEP.powf(-steps);
        self.camera.distance_au = (self.camera.distance_au * factor).clamp(MIN_DISTANCE_AU, MAX_DISTANCE_AU);
    }

    pub fn pan_by_pixels(&mut self, delta_x_px: f32, delta_y_px: f32, viewport_width_px: f32, viewport_height_px: f32) {
        let width_px = viewport_width_px.max(1.0) as f64;
        let height_px = viewport_height_px.max(1.0) as f64;
        let aspect = (width_px / height_px).max(MIN_ASPECT);
        let tan_half_fov = ((self.camera.fov_deg as f64).to_radians() * 0.5).tan();
        let half_height_world = self.camera.distance_au as f64 * tan_half_fov;
        let half_width_world = half_height_world * aspect;

        let world_per_px_x = (2.0 * half_width_world) / width_px;
        let world_per_px_y = (2.0 * half_height_world) / height_px;

        let basis = Self::camera_basis(self.camera, self.camera_target_au);
        let delta_right = basis.right.scale(-(delta_x_px as f64) * world_per_px_x);
        let delta_up = basis.up.scale((delta_y_px as f64) * world_per_px_y);
        self.camera_target_au = self.camera_target_au + delta_right + delta_up;
    }

    fn interpolate_body_state(current: Option<&StateVector>, next: Option<&StateVector>, alpha: f64) -> Option<StateVector> {
        match (current, next) {
            (Some(cur), Some(nxt)) => {
                let t = alpha.clamp(0.0, 1.0);
                Some(StateVector {
                    epoch: cur.epoch,
                    x_km: cur.x_km + (nxt.x_km - cur.x_km) * t,
                    y_km: cur.y_km + (nxt.y_km - cur.y_km) * t,
                    z_km: cur.z_km + (nxt.z_km - cur.z_km) * t,
                    vx_km_s: cur.vx_km_s,
                    vy_km_s: cur.vy_km_s,
                    vz_km_s: cur.vz_km_s,
                })
            }
            (Some(cur), None) => Some(cur.clone()),
            (None, Some(nxt)) => Some(nxt.clone()),
            (None, None) => None,
        }
    }

    fn selected_center_point_au(&self, day: &crate::model::DaySnapshot, alpha: f64) -> Vec3 {
        if self.selected_center == BodyId::Sun {
            return Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        }

        let next_day = self.day_index.days.get(self.day_index.current_day_idx + 1);
        let current_state = day.by_body.get(&self.selected_center);
        let next_state = next_day.and_then(|d| d.by_body.get(&self.selected_center));
        let Some(state) = Self::interpolate_body_state(current_state, next_state, alpha) else {
            return Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        };

        Self::body_point_au(self.selected_center, &state)
    }

    fn projection_params(camera: CameraState, width_px: f32, height_px: f32) -> ProjectionParams {
        let fov = (camera.fov_deg as f64).to_radians();
        let far = (camera.distance_au as f64 + 5.0).max(CLIP_FAR_AU);
        ProjectionParams {
            near: CLIP_NEAR_AU,
            far,
            tan_half_fov: (fov * 0.5).tan(),
            aspect: (width_px as f64 / height_px as f64).max(MIN_ASPECT),
        }
    }

    fn camera_basis(camera: CameraState, target: Vec3) -> CameraBasis {
        let az = (camera.azimuth_deg as f64).to_radians();
        let el = (camera.elevation_deg as f64).to_radians();
        let distance = camera.distance_au as f64;

        let offset = Vec3 {
            x: distance * el.cos() * az.cos(),
            y: distance * el.cos() * az.sin(),
            z: distance * el.sin(),
        };
        let camera_pos = target + offset;

        let forward = (target - camera_pos).normalize().unwrap_or(Vec3 { x: 0.0, y: 0.0, z: -1.0 });
        let world_up = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
        let right = forward.cross(world_up).normalize().unwrap_or(Vec3 { x: 1.0, y: 0.0, z: 0.0 });
        let up = right.cross(forward).normalize().unwrap_or(world_up);

        CameraBasis {
            camera_pos,
            forward,
            right,
            up,
        }
    }

    fn body_point_au(body_id: BodyId, state: &StateVector) -> Vec3 {
        match body_id {
            BodyId::Sun => Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            BodyId::Mercury | BodyId::Venus | BodyId::Earth | BodyId::Moon | BodyId::Mars | BodyId::Phobos | BodyId::Deimos => Vec3 {
                x: state.x_km / AU_KM,
                y: state.y_km / AU_KM,
                z: state.z_km / AU_KM,
            },
        }
    }

    fn world_bodies_for_frame(&self, day: &crate::model::DaySnapshot, alpha: f64) -> Vec<(BodyId, Vec3)> {
        let next_day = self.day_index.days.get(self.day_index.current_day_idx + 1);
        let center_point = self.selected_center_point_au(day, alpha);
        let mut world_bodies = Vec::new();

        let body_ids: Vec<_> = day.by_body.keys().copied().collect();
        for &body_id in &body_ids {
            let current_state = day.by_body.get(&body_id);
            let next_state = next_day.and_then(|d| d.by_body.get(&body_id));
            let Some(state) = Self::interpolate_body_state(current_state, next_state, alpha) else {
                continue;
            };

            let absolute_point = Self::body_point_au(body_id, &state);
            world_bodies.push((body_id, absolute_point - center_point));
        }

        world_bodies
    }

    fn project_body(
        body_id: BodyId,
        point: Vec3,
        basis: CameraBasis,
        params: ProjectionParams,
        zoom_scale: f64,
        width_px: f32,
        height_px: f32,
    ) -> Option<RenderBody> {
        let rel = point - basis.camera_pos;
        let cam_x = rel.dot(basis.right);
        let cam_y = rel.dot(basis.up);
        let cam_z = rel.dot(basis.forward);

        if cam_z <= params.near || cam_z >= params.far {
            return None;
        }

        let x_ndc = cam_x / (cam_z * params.tan_half_fov * params.aspect);
        let y_ndc = cam_y / (cam_z * params.tan_half_fov);

        if !x_ndc.is_finite() || !y_ndc.is_finite() {
            return None;
        }

        let x_px = ((x_ndc * NDC_TO_PIXEL_SCALE + NDC_TO_PIXEL_SCALE) * width_px as f64) as f32;
        let y_px = ((NDC_TO_PIXEL_SCALE - y_ndc * NDC_TO_PIXEL_SCALE) * height_px as f64) as f32;

        let radius_au = body_id.radius_km() / AU_KM;
        let zoom_scale = zoom_scale.clamp(MIN_RADIUS_ZOOM_SCALE, MAX_RADIUS_ZOOM_SCALE);
        let base_radius_px = ((radius_au * BODY_RADIUS_SCALE_PX) / cam_z.max(MIN_DEPTH_FOR_RADIUS)) * zoom_scale;
        let min_visible_px = body_id.min_visible_radius_px();
        let max_visible_px = Self::max_visible_radius_px(body_id);
        let radius_px = base_radius_px.clamp(min_visible_px, max_visible_px) as f32;

        Some(RenderBody {
            name: body_id.name().to_string(),
            x_px,
            y_px,
            radius_px,
            depth: cam_z as f32,
            color_rgb: body_id.color_rgb(),
        })
    }

    pub fn build_scene_frame(&self, width_px: f32, height_px: f32) -> SceneFrame {
        let Some(day) = self.day_index.current_day() else {
            return SceneFrame { bodies: Vec::new() };
        };

        let alpha = self.current_day_progress.clamp(0.0, 1.0);

        let basis = Self::camera_basis(self.camera, self.camera_target_au);
        let params = Self::projection_params(self.camera, width_px, height_px);
        let zoom_scale = (DEFAULT_DISTANCE_AU as f64 / self.camera.distance_au as f64).clamp(MIN_RADIUS_ZOOM_SCALE, MAX_RADIUS_ZOOM_SCALE);
        let world_bodies = self.world_bodies_for_frame(day, alpha);

        let mut projected = Vec::new();

        for (body_id, point) in world_bodies {
            if let Some(render_body) = Self::project_body(body_id, point, basis, params, zoom_scale, width_px, height_px) {
                projected.push(render_body);
            }
        }

        projected.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        SceneFrame { bodies: projected }
    }
}
