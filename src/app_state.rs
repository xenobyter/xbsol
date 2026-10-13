use crate::model::EphemerisDayIndex;

const AU_KM: f64 = 149_597_870.7;
const SUN_RADIUS_KM: f64 = 696_340.0;

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
            azimuth_deg: 135.0,
            elevation_deg: 22.0,
            distance_au: 4.5,
            fov_deg: 50.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
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
            current_day_progress: 0.0,
        }
    }

    pub fn date_text(&self) -> String {
        match self.day_index.current_day() {
            Some(day) => format!("{}  |  Bodies: {}", day.date, day.by_body.len()),
            None => "No ephemeris data available".to_string(),
        }
    }

    pub fn prev_day(&mut self) {
        self.current_day_progress = 0.0;
        let _ = self.day_index.prev_day();
    }

    pub fn next_day(&mut self) {
        self.current_day_progress = 0.0;
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
        while self.current_day_progress >= 1.0 {
            if self.day_index.current_day_idx + 1 < self.day_index.days.len() {
                self.day_index.current_day_idx += 1;
                self.current_day_progress -= 1.0;
            } else {
                self.current_day_progress = 0.0;
                break;
            }
        }
    }

    pub fn camera(&self) -> CameraState {
        self.camera
    }

    fn interpolate_body_state(current: Option<&crate::model::StateVector>, next: Option<&crate::model::StateVector>, alpha: f64) -> Option<crate::model::StateVector> {
        match (current, next) {
            (Some(cur), Some(nxt)) => {
                let t = alpha.clamp(0.0, 1.0);
                Some(crate::model::StateVector {
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

    pub fn build_scene_frame(&self, width_px: f32, height_px: f32) -> SceneFrame {
        let Some(day) = self.day_index.current_day() else {
            return SceneFrame { bodies: Vec::new() };
        };

        let current_idx = self.day_index.current_day_idx;
        let next_day = self.day_index.days.get(current_idx + 1);
        let alpha = self.current_day_progress.clamp(0.0, 1.0);

        let camera = self.camera;
        let az = (camera.azimuth_deg as f64).to_radians();
        let el = (camera.elevation_deg as f64).to_radians();
        let distance = camera.distance_au as f64;

        let target = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let camera_pos = Vec3 {
            x: distance * el.cos() * az.cos(),
            y: distance * el.cos() * az.sin(),
            z: distance * el.sin(),
        };
        let forward = (target - camera_pos).normalize().unwrap_or(Vec3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        });

        let world_up = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };
        let right = forward.cross(world_up).normalize().unwrap_or(Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        });
        let up = right.cross(forward).normalize().unwrap_or(world_up);

        let near = 0.01_f64;
        let far = 12.0_f64;
        let fov = (camera.fov_deg as f64).to_radians();
        let tan_half_fov = (fov * 0.5).tan();
        let aspect = (width_px as f64 / height_px as f64).max(1e-6);

        let mut projected = Vec::new();
        let mut world_bodies = Vec::new();

        let body_ids: Vec<_> = day.by_body.keys().copied().collect();
        for &body_id in &body_ids {
            let current_state = day.by_body.get(&body_id);
            let next_state = next_day.and_then(|d| d.by_body.get(&body_id));
            let interp = Self::interpolate_body_state(current_state, next_state, alpha);

            let Some(state) = interp else {
                continue;
            };

            let point = match body_id {
                crate::model::BodyId::Sun => Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                crate::model::BodyId::Mercury
                | crate::model::BodyId::Venus
                | crate::model::BodyId::Earth
                | crate::model::BodyId::Moon
                | crate::model::BodyId::Mars
                | crate::model::BodyId::Phobos
                | crate::model::BodyId::Deimos => Vec3 {
                    x: state.x_km / AU_KM,
                    y: state.y_km / AU_KM,
                    z: state.z_km / AU_KM,
                },
            };

            world_bodies.push((body_id, point));
        }

        for (body_id, point) in world_bodies {
            let rel = point - camera_pos;
            let cam_x = rel.dot(right);
            let cam_y = rel.dot(up);
            let cam_z = rel.dot(forward);

            if cam_z <= near || cam_z >= far {
                continue;
            }

            let x_ndc = cam_x / (cam_z * tan_half_fov * aspect);
            let y_ndc = cam_y / (cam_z * tan_half_fov);

            if !x_ndc.is_finite() || !y_ndc.is_finite() {
                continue;
            }

            let x_px = ((x_ndc * 0.5 + 0.5) * width_px as f64) as f32;
            let y_px = ((0.5 - y_ndc * 0.5) * height_px as f64) as f32;

            let radius_au = body_id.radius_km() / AU_KM;
            let base_radius_px = (radius_au * 260_000.0) / cam_z.max(0.2);
            let min_visible_px = match body_id.name() {
                "Sun" => 16.0,
                "Mercury" => 3.0,
                "Venus" => 5.0,
                "Earth" => 7.0,
                "Moon" => 4.0,
                "Mars" => 6.0,
                "Phobos" => 2.0,
                "Deimos" => 2.0,
                _ => 4.0,
            };
            let base_radius_px = base_radius_px.clamp(min_visible_px, 70.0) as f32;

            let color = match body_id.name() {
                "Sun" => (255, 191, 0),
                "Mercury" => (150, 150, 150),
                "Venus" => (209, 160, 110),
                "Earth" => (85, 140, 255),
                "Moon" => (190, 190, 200),
                "Mars" => (201, 102, 70),
                "Phobos" => (138, 126, 116),
                "Deimos" => (168, 160, 146),
                _ => (220, 220, 220),
            };

            projected.push(RenderBody {
                name: body_id.name().to_string(),
                x_px,
                y_px,
                radius_px: base_radius_px,
                depth: cam_z as f32,
                color_rgb: color,
            });
        }

        projected.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        SceneFrame { bodies: projected }
    }
}
