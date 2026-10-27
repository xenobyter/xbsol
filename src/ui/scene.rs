use super::{BodySprite, MainWindow};
use crate::app_state::{AppState, RenderBody, RenderTrailPoint};
use slint::{Color, Image, ModelRc, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default)]
pub(super) struct SkinCache {
    images: HashMap<(String, SkinTier), Option<Image>>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum SkinTier {
    Low,
    Medium,
    High,
}

impl SkinTier {
    fn for_radius(radius_px: f32) -> Self {
        if radius_px < 8.0 {
            Self::Low
        } else if radius_px < 20.0 {
            Self::Medium
        } else {
            Self::High
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl SkinCache {
    fn resolve(&mut self, stem: &str, tier: SkinTier) -> Option<Image> {
        let cache_key = (stem.to_owned(), tier);
        if let Some(image) = self.images.get(&cache_key) {
            return image.clone();
        }

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let extensions = ["svg", "png", "webp", "gif"];
        let mut candidates = Vec::new();

        for ext in extensions {
            candidates.push(manifest_dir.join(format!(
                "assets/skins/{}_{}.{}",
                stem,
                tier.as_str(),
                ext
            )));
        }
        for ext in extensions {
            candidates.push(manifest_dir.join(format!("assets/skins/{}.{}", stem, ext)));
        }

        for path in candidates {
            if path.exists() {
                match Image::load_from_path(&path) {
                    Ok(image) => {
                        self.images.insert(cache_key, Some(image.clone()));
                        return Some(image);
                    }
                    Err(error) => {
                        log::warn!("Skin konnte nicht geladen werden ({}): {}", stem, error);
                    }
                }
            }
        }

        self.images.insert(cache_key, None);
        None
    }
}

pub(super) fn refresh_scene(
    ui: &MainWindow,
    state: &AppState,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let width = ui.get_scene_width().max(1.0);
    let height = ui.get_scene_height().max(1.0);
    let frame = state.build_scene_frame(width, height);

    let sprites: Vec<BodySprite> = {
        let mut skin_cache = skin_cache.borrow_mut();
        frame
            .bodies
            .into_iter()
            .map(|body| body_to_sprite(body, &mut skin_cache))
            .collect()
    };

    let trails: Vec<BodySprite> = frame.trails.into_iter().map(trail_to_sprite).collect();

    let model = Rc::new(VecModel::from(sprites));
    ui.set_body_sprites(ModelRc::from(model));
    let trail_model = Rc::new(VecModel::from(trails));
    ui.set_trail_sprites(ModelRc::from(trail_model));
}

fn body_to_sprite(body: RenderBody, skin_cache: &mut SkinCache) -> BodySprite {
    let texture = skin_cache.resolve(&body.sprite_stem, SkinTier::for_radius(body.radius_px));

    BodySprite {
        name: body.name.into(),
        texture: texture.clone().unwrap_or_default(),
        has_texture: texture.is_some(),
        x_px: body.x_px,
        y_px: body.y_px,
        radius_px: body.radius_px,
        color: Color::from_rgb_u8(body.color_rgb.0, body.color_rgb.1, body.color_rgb.2),
    }
}

fn trail_to_sprite(trail: RenderTrailPoint) -> BodySprite {
    BodySprite {
        name: "trail".into(),
        texture: Image::default(),
        has_texture: false,
        x_px: trail.x_px,
        y_px: trail.y_px,
        radius_px: trail.radius_px,
        color: Color::from_argb_u8(
            (trail.alpha.clamp(0.0, 1.0) * 255.0) as u8,
            trail.color_rgb.0,
            trail.color_rgb.1,
            trail.color_rgb.2,
        ),
    }
}
