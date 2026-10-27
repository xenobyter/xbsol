use super::*;

#[derive(Debug, Clone)]
pub struct RenderBody {
    pub name: String,
    pub sprite_stem: String,
    pub x_px: f32,
    pub y_px: f32,
    pub radius_px: f32,
    pub depth: f32,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone)]
pub struct SceneFrame {
    pub bodies: Vec<RenderBody>,
    pub trails: Vec<RenderTrailPoint>,
}

#[derive(Debug, Clone)]
pub struct RenderTrailPoint {
    pub x_px: f32,
    pub y_px: f32,
    pub radius_px: f32,
    pub depth: f32,
    pub color_rgb: (u8, u8, u8),
    pub alpha: f32,
}

const CLIP_NEAR_AU: f64 = 0.001;
const CLIP_FAR_AU: f64 = 12.0;
const NDC_TO_PIXEL_SCALE: f64 = 0.5;
const BODY_RADIUS_SCALE_PX: f64 = 260_000.0;
const MIN_DEPTH_FOR_RADIUS: f64 = 0.2;
const MIN_RADIUS_ZOOM_SCALE: f64 = 0.35;
const MAX_RADIUS_ZOOM_SCALE: f64 = 6.0;
const TRAIL_POINT_RADIUS_PX: f32 = 0.8;
const TRAIL_ALPHA_MIN: f32 = 0.16;
const TRAIL_ALPHA_MAX: f32 = 0.78;

#[derive(Debug, Clone, Copy)]
struct ProjectionParams {
    near: f64,
    far: f64,
    tan_half_fov: f64,
    aspect: f64,
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
            BodyId::Jupiter => 78.0,
            BodyId::Saturn => 72.0,
            BodyId::Uranus => 62.0,
            BodyId::Neptune => 62.0,
            BodyId::Io => 10.0,
            BodyId::Europa => 10.0,
            BodyId::Ganymede => 12.0,
            BodyId::Callisto => 12.0,
            BodyId::Mimas => 5.0,
            BodyId::Enceladus => 6.0,
            BodyId::Tethys => 8.0,
            BodyId::Dione => 8.0,
            BodyId::Rhea => 10.0,
            BodyId::Titan => 14.0,
            BodyId::Iapetus => 10.0,
            BodyId::Miranda => 5.0,
            BodyId::Ariel => 7.0,
            BodyId::Umbriel => 7.0,
            BodyId::Titania => 9.0,
            BodyId::Oberon => 9.0,
            BodyId::Triton => 10.0,
            BodyId::Nereid => 5.0,
            BodyId::Proteus => 6.0,
            BodyId::Phobos => 4.0,
            BodyId::Deimos => 4.0,
        }
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

    fn world_bodies_for_frame(
        &self,
        day: &crate::model::DaySnapshot,
        alpha: f64,
    ) -> Vec<(BodyId, Vec3)> {
        let center_point = self.selected_center_point_au(day, alpha);
        let mut world_bodies = Vec::new();

        let body_ids: Vec<_> = day.by_body.keys().copied().collect();
        for &body_id in &body_ids {
            let Some(state) = self.interpolated_body_state(day, body_id, alpha) else {
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
        let (x_px, y_px, cam_z) = Self::project_point(point, basis, params, width_px, height_px)?;

        let radius_au = body_id.radius_km() / AU_KM;
        let zoom_scale = zoom_scale.clamp(MIN_RADIUS_ZOOM_SCALE, MAX_RADIUS_ZOOM_SCALE);
        let base_radius_px =
            ((radius_au * BODY_RADIUS_SCALE_PX) / cam_z.max(MIN_DEPTH_FOR_RADIUS)) * zoom_scale;
        let min_visible_px = body_id.min_visible_radius_px();
        let max_visible_px = Self::max_visible_radius_px(body_id);
        let radius_px = base_radius_px.clamp(min_visible_px, max_visible_px) as f32;

        Some(RenderBody {
            name: body_id.name().to_string(),
            sprite_stem: body_id.sprite_stem().to_string(),
            x_px,
            y_px,
            radius_px,
            depth: cam_z as f32,
            color_rgb: body_id.color_rgb(),
        })
    }

    fn project_point(
        point: Vec3,
        basis: CameraBasis,
        params: ProjectionParams,
        width_px: f32,
        height_px: f32,
    ) -> Option<(f32, f32, f64)> {
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

        Some((x_px, y_px, cam_z))
    }

    fn build_trails_for_frame(
        &self,
        basis: CameraBasis,
        params: ProjectionParams,
        width_px: f32,
        height_px: f32,
    ) -> Vec<RenderTrailPoint> {
        if !self.show_trails {
            return Vec::new();
        }

        let mut trails = Vec::new();

        for (&body_id, points) in &self.trail_history_au {
            if body_id == self.selected_center || !Self::is_trail_body(body_id) || points.is_empty()
            {
                continue;
            }

            let body_color = body_id.color_rgb();
            let denom = (points.len().saturating_sub(1).max(1)) as f32;

            for (idx, relative_point) in points.iter().copied().enumerate() {
                let Some((x_px, y_px, cam_z)) =
                    Self::project_point(relative_point, basis, params, width_px, height_px)
                else {
                    continue;
                };

                let t = idx as f32 / denom;
                let alpha = TRAIL_ALPHA_MIN + (TRAIL_ALPHA_MAX - TRAIL_ALPHA_MIN) * t;

                trails.push(RenderTrailPoint {
                    x_px,
                    y_px,
                    radius_px: TRAIL_POINT_RADIUS_PX,
                    depth: cam_z as f32,
                    color_rgb: body_color,
                    alpha,
                });
            }
        }

        trails.sort_by(|a, b| {
            b.depth
                .partial_cmp(&a.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        trails
    }

    pub fn build_scene_frame(&self, width_px: f32, height_px: f32) -> SceneFrame {
        let Some(day) = self.day_index.current_day() else {
            return SceneFrame {
                bodies: Vec::new(),
                trails: Vec::new(),
            };
        };

        let alpha = self.current_day_progress.clamp(0.0, 1.0);

        let basis = Self::camera_basis(self.camera, self.camera_target_au);
        let params = Self::projection_params(self.camera, width_px, height_px);
        let zoom_scale = (DEFAULT_DISTANCE_AU as f64 / self.camera.distance_au as f64)
            .clamp(MIN_RADIUS_ZOOM_SCALE, MAX_RADIUS_ZOOM_SCALE);
        let world_bodies = self.world_bodies_for_frame(day, alpha);

        let mut projected = Vec::new();

        for (body_id, point) in world_bodies {
            if let Some(render_body) = Self::project_body(
                body_id, point, basis, params, zoom_scale, width_px, height_px,
            ) {
                projected.push(render_body);
            }
        }

        projected.sort_by(|a, b| {
            b.depth
                .partial_cmp(&a.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let trails = self.build_trails_for_frame(basis, params, width_px, height_px);

        SceneFrame {
            bodies: projected,
            trails,
        }
    }
}
