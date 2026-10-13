use crate::app_state::AppState;
use crate::model::EphemerisDayIndex;
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

pub fn run_ui(day_index: EphemerisDayIndex) -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let app_state = Rc::new(RefCell::new(AppState::new(day_index)));

    ui.set_date_text(app_state.borrow().date_text().into());

    let weak_prev = ui.as_weak();
    let state_prev = Rc::clone(&app_state);
    ui.on_prev_day(move || {
        state_prev.borrow_mut().prev_day();
        if let Some(ui) = weak_prev.upgrade() {
            ui.set_date_text(state_prev.borrow().date_text().into());
        }
    });

    let weak_next = ui.as_weak();
    let state_next = Rc::clone(&app_state);
    ui.on_next_day(move || {
        state_next.borrow_mut().next_day();
        if let Some(ui) = weak_next.upgrade() {
            ui.set_date_text(state_next.borrow().date_text().into());
        }
    });

    ui.run()
}
