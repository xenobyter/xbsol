use chrono::{DateTime, NaiveDate, Utc};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyId {
    Earth,
    Moon,
}

impl BodyId {
    pub const fn horizons_command(self) -> &'static str {
        match self {
            BodyId::Earth => "399",
            BodyId::Moon => "301",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            BodyId::Earth => "Earth",
            BodyId::Moon => "Moon",
        }
    }

    pub const fn radius_km(self) -> f64 {
        match self {
            BodyId::Earth => 6_371.01,
            BodyId::Moon => 1_737.53,
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

pub const BODIES: &[BodyMeta] = &[BodyMeta { body_id: BodyId::Earth }, BodyMeta { body_id: BodyId::Moon }];

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
