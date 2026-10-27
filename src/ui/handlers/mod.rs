mod camera;
mod date_range;
mod navigation;

use super::MainWindow;
use super::scene::SkinCache;
use crate::app_state::AppState;
use crate::horizon::HorizonsDateRange;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::Handle;

pub(super) fn register_all(
    ui: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    skin_cache: &Rc<RefCell<SkinCache>>,
    active_range: &Rc<RefCell<HorizonsDateRange>>,
    client: &reqwest::Client,
    runtime: &Handle,
) {
    date_range::register(ui, app_state, skin_cache, active_range, client, runtime);
    navigation::register(ui, app_state, skin_cache);
    camera::register(ui, app_state, skin_cache);
}
