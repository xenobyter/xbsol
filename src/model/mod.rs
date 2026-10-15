use chrono::{DateTime, NaiveDate, Utc};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyId {
    Sun,
    Mercury,
    Venus,
    Earth,
    Moon,
    Mars,
    Phobos,
    Deimos,
}

impl BodyId {
    pub const fn horizons_command(self) -> &'static str {
        match self {
            BodyId::Sun => "10",
            BodyId::Mercury => "199",
            BodyId::Venus => "299",
            BodyId::Earth => "399",
            BodyId::Moon => "301",
            BodyId::Mars => "499",
            BodyId::Phobos => "401",
            BodyId::Deimos => "402",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            BodyId::Sun => "Sun",
            BodyId::Mercury => "Mercury",
            BodyId::Venus => "Venus",
            BodyId::Earth => "Earth",
            BodyId::Moon => "Moon",
            BodyId::Mars => "Mars",
            BodyId::Phobos => "Phobos",
            BodyId::Deimos => "Deimos",
        }
    }

    pub const fn radius_km(self) -> f64 {
        match self {
            BodyId::Sun => 696_340.0,
            BodyId::Mercury => 2_439.7,
            BodyId::Venus => 6_051.8,
            BodyId::Earth => 6_371.01,
            BodyId::Moon => 1_737.53,
            BodyId::Mars => 3_389.5,
            BodyId::Phobos => 11.2667,
            BodyId::Deimos => 6.2,
        }
    }

    pub const fn min_visible_radius_px(self) -> f64 {
        match self {
            BodyId::Sun => 16.0,
            BodyId::Mercury => 3.0,
            BodyId::Venus => 5.0,
            BodyId::Earth => 7.0,
            BodyId::Moon => 4.0,
            BodyId::Mars => 6.0,
            BodyId::Phobos => 2.0,
            BodyId::Deimos => 2.0,
        }
    }

    pub const fn color_rgb(self) -> (u8, u8, u8) {
        match self {
            BodyId::Sun => (255, 191, 0),
            BodyId::Mercury => (150, 150, 150),
            BodyId::Venus => (209, 160, 110),
            BodyId::Earth => (85, 140, 255),
            BodyId::Moon => (190, 190, 200),
            BodyId::Mars => (201, 102, 70),
            BodyId::Phobos => (138, 126, 116),
            BodyId::Deimos => (168, 160, 146),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BodyMeta {
    pub body_id: BodyId,
}

impl BodyMeta {
    pub const fn command(self) -> &'static str {
        self.body_id.horizons_command()
    }
}

pub const BODIES: &[BodyMeta] = &[
    BodyMeta { body_id: BodyId::Sun },
    BodyMeta { body_id: BodyId::Mercury },
    BodyMeta { body_id: BodyId::Venus },
    BodyMeta { body_id: BodyId::Earth },
    BodyMeta { body_id: BodyId::Moon },
    BodyMeta { body_id: BodyId::Mars },
    BodyMeta { body_id: BodyId::Phobos },
    BodyMeta { body_id: BodyId::Deimos },
];

#[derive(Debug, Clone)]
pub struct StateVector {
    pub epoch: DateTime<Utc>,
    pub x_km: f64,
    pub y_km: f64,
    pub z_km: f64,
    pub vx_km_s: f64,
    pub vy_km_s: f64,
    pub vz_km_s: f64,
}

#[derive(Debug, Clone)]
pub struct EphemerisRecord {
    pub body_id: BodyId,
    pub center: String,
    pub ref_plane: String,
    pub step: String,
    pub vector: StateVector,
}

#[derive(Debug, Default)]
pub struct EphemerisTable {
    pub rows: Vec<EphemerisRecord>,
}

impl EphemerisTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extend(&mut self, records: Vec<EphemerisRecord>) {
        self.rows.extend(records);
    }

    pub fn build_day_index(&self) -> EphemerisDayIndex {
        let mut grouped: BTreeMap<NaiveDate, HashMap<BodyId, StateVector>> = BTreeMap::new();

        for row in &self.rows {
            let day = row.vector.epoch.date_naive();
            grouped.entry(day).or_default().insert(row.body_id, row.vector.clone());
        }

        let days: Vec<DaySnapshot> = grouped.into_iter().map(|(date, by_body)| DaySnapshot { date, by_body }).collect();

        let day_lookup = days.iter().enumerate().map(|(idx, day)| (day.date, idx)).collect();

        EphemerisDayIndex {
            days,
            day_lookup,
            current_day_idx: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DaySnapshot {
    pub date: NaiveDate,
    pub by_body: HashMap<BodyId, StateVector>,
}

#[derive(Debug, Clone, Default)]
pub struct EphemerisDayIndex {
    pub days: Vec<DaySnapshot>,
    pub day_lookup: HashMap<NaiveDate, usize>,
    pub current_day_idx: usize,
}

impl EphemerisDayIndex {
    pub fn len(&self) -> usize {
        self.days.len()
    }

    pub fn is_empty(&self) -> bool {
        self.days.is_empty()
    }

    pub fn current_day(&self) -> Option<&DaySnapshot> {
        if self.is_empty() {
            return None;
        }
        self.days.get(self.current_day_idx)
    }

    pub fn next_day(&mut self) -> Option<&DaySnapshot> {
        if self.is_empty() {
            return None;
        }
        if self.current_day_idx + 1 < self.days.len() {
            self.current_day_idx += 1;
        }
        self.days.get(self.current_day_idx)
    }

    pub fn prev_day(&mut self) -> Option<&DaySnapshot> {
        if self.is_empty() {
            return None;
        }
        if self.current_day_idx > 0 {
            self.current_day_idx -= 1;
        }
        self.days.get(self.current_day_idx)
    }

    pub fn get_day(&self, date: NaiveDate) -> Option<&DaySnapshot> {
        let idx = self.day_lookup.get(&date)?;
        self.days.get(*idx)
    }
}
