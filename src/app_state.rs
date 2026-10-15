use crate::model::{BodyId, EphemerisDayIndex, StateVector};

const AU_KM: f64 = 149_597_870.7;
const DAY_PROGRESS_RESET: f64 = 0.0;
const DAY_PROGRESS_THRESHOLD: f64 = 1.0;
const CLIP_NEAR_AU: f64 = 0.01;
const CLIP_FAR_AU: f64 = 12.0;
const MIN_ASPECT: f64 = 1e-6;
const NDC_TO_PIXEL_SCALE: f64 = 0.5;
const BODY_RADIUS_SCALE_PX: f64 = 260_000.0;
const MIN_DEPTH_FOR_RADIUS: f64 = 0.2;
const MAX_VISIBLE_RADIUS_PX: f64 = 70.0;

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
            // Diagonal, etwas weiter entfernter Blick auf das innere Sonnensystem,
            // damit Sonne, Merkur, Venus und Erde gut gleichzeitig sichtbar sind.
            azimuth_deg: DEFAULT_AZIMUTH_DEG,
            elevation_deg: DEFAULT_ELEVATION_DEG,
            distance_au: DEFAULT_DISTANCE_AU,
            fov_deg: DEFAULT_FOV_DEG,
        }
    }
}

const DEFAULT_AZIMUTH_DEG: f32 = 135.0;
const DEFAULT_ELEVATION_DEG: f32 = 22.0;
const DEFAULT_DISTANCE_AU: f32 = 4.5;
const DEFAULT_FOV_DEG: f32 = 50.0;

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

pub struct AppState {
    day_index: EphemerisDayIndex,
    camera: CameraState,
    auto_play: bool,
    current_day_progress: f64,
}

impl AppState {
    pub fn new(day_index: EphemerisDayIndex) -> Self {
        Self {
            day_index,
            camera: CameraState::default(),
            auto_play: false,
            current_day_progress: DAY_PROGRESS_RESET,
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

    fn projection_params(camera: CameraState, width_px: f32, height_px: f32) -> ProjectionParams {
        let fov = (camera.fov_deg as f64).to_radians();
        ProjectionParams {
            near: CLIP_NEAR_AU,
            far: CLIP_FAR_AU,
            tan_half_fov: (fov * 0.5).tan(),
            aspect: (width_px as f64 / height_px as f64).max(MIN_ASPECT),
        }
    }

    fn camera_basis(camera: CameraState) -> CameraBasis {
        let az = (camera.azimuth_deg as f64).to_radians();
        let el = (camera.elevation_deg as f64).to_radians();
        let distance = camera.distance_au as f64;

        let target = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let camera_pos = Vec3 {
            x: distance * el.cos() * az.cos(),
            y: distance * el.cos() * az.sin(),
            z: distance * el.sin(),
        };

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
        let mut world_bodies = Vec::new();

        let body_ids: Vec<_> = day.by_body.keys().copied().collect();
        for &body_id in &body_ids {
            let current_state = day.by_body.get(&body_id);
            let next_state = next_day.and_then(|d| d.by_body.get(&body_id));
            let Some(state) = Self::interpolate_body_state(current_state, next_state, alpha) else {
                continue;
            };

            world_bodies.push((body_id, Self::body_point_au(body_id, &state)));
        }

        world_bodies
    }

    fn project_body(
        body_id: BodyId,
        point: Vec3,
        basis: CameraBasis,
        params: ProjectionParams,
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
        let base_radius_px = (radius_au * BODY_RADIUS_SCALE_PX) / cam_z.max(MIN_DEPTH_FOR_RADIUS);
        let min_visible_px = body_id.min_visible_radius_px();
        let radius_px = base_radius_px.clamp(min_visible_px, MAX_VISIBLE_RADIUS_PX) as f32;

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

        let basis = Self::camera_basis(self.camera);
        let params = Self::projection_params(self.camera, width_px, height_px);
        let world_bodies = self.world_bodies_for_frame(day, alpha);

        let mut projected = Vec::new();

        for (body_id, point) in world_bodies {
            if let Some(render_body) = Self::project_body(body_id, point, basis, params, width_px, height_px) {
                projected.push(render_body);
            }
        }

        projected.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        SceneFrame { bodies: projected }
    }
}
