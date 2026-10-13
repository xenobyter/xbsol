use crate::model::EphemerisDayIndex;

pub struct AppState {
    day_index: EphemerisDayIndex,
}

impl AppState {
    pub fn new(day_index: EphemerisDayIndex) -> Self {
        Self { day_index }
    }

    pub fn date_text(&self) -> String {
        match self.day_index.current_day() {
            Some(day) => format!("{}  |  Bodies: {}", day.date, day.by_body.len()),
            None => "No ephemeris data available".to_string(),
        }
    }

    pub fn prev_day(&mut self) {
        let _ = self.day_index.prev_day();
    }

    pub fn next_day(&mut self) {
        let _ = self.day_index.next_day();
    }
}
