use super::{BodySprite, MainWindow};
use crate::app_state::AppState;
use slint::{Color, Image, ModelRc, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub(super) type SkinCache = HashMap<String, Option<Image>>;

pub(super) fn refresh_scene(
    ui: &MainWindow,
    state: &AppState,
    skin_cache: &Rc<RefCell<SkinCache>>,
) {
    let width = ui.get_scene_width().max(1.0);
    let height = ui.get_scene_height().max(1.0);
    let frame = state.build_scene_frame(width, height);

    let sprites: Vec<BodySprite> = frame
        .bodies
        .into_iter()
        .map(|body| {
            let tier = skin_tier_for_radius(body.radius_px);
            let texture_opt =
                resolve_skin_image(&mut skin_cache.borrow_mut(), &body.sprite_stem, tier);
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

    let trails: Vec<BodySprite> = frame
        .trails
        .into_iter()
        .map(|trail| BodySprite {
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
        })
        .collect();

    let model = Rc::new(VecModel::from(sprites));
    ui.set_body_sprites(ModelRc::from(model));
    let trail_model = Rc::new(VecModel::from(trails));
    ui.set_trail_sprites(ModelRc::from(trail_model));
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
