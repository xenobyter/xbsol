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
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Io,
    Europa,
    Ganymede,
    Callisto,
    Mimas,
    Enceladus,
    Tethys,
    Dione,
    Rhea,
    Titan,
    Iapetus,
    Miranda,
    Ariel,
    Umbriel,
    Titania,
    Oberon,
    Triton,
    Nereid,
    Proteus,
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
            BodyId::Jupiter => "599",
            BodyId::Saturn => "699",
            BodyId::Uranus => "799",
            BodyId::Neptune => "899",
            BodyId::Io => "501",
            BodyId::Europa => "502",
            BodyId::Ganymede => "503",
            BodyId::Callisto => "504",
            BodyId::Mimas => "601",
            BodyId::Enceladus => "602",
            BodyId::Tethys => "603",
            BodyId::Dione => "604",
            BodyId::Rhea => "605",
            BodyId::Titan => "606",
            BodyId::Iapetus => "608",
            BodyId::Miranda => "705",
            BodyId::Ariel => "701",
            BodyId::Umbriel => "702",
            BodyId::Titania => "703",
            BodyId::Oberon => "704",
            BodyId::Triton => "801",
            BodyId::Nereid => "802",
            BodyId::Proteus => "808",
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
            BodyId::Jupiter => "Jupiter",
            BodyId::Saturn => "Saturn",
            BodyId::Uranus => "Uranus",
            BodyId::Neptune => "Neptune",
            BodyId::Io => "Io",
            BodyId::Europa => "Europa",
            BodyId::Ganymede => "Ganymede",
            BodyId::Callisto => "Callisto",
            BodyId::Mimas => "Mimas",
            BodyId::Enceladus => "Enceladus",
            BodyId::Tethys => "Tethys",
            BodyId::Dione => "Dione",
            BodyId::Rhea => "Rhea",
            BodyId::Titan => "Titan",
            BodyId::Iapetus => "Iapetus",
            BodyId::Miranda => "Miranda",
            BodyId::Ariel => "Ariel",
            BodyId::Umbriel => "Umbriel",
            BodyId::Titania => "Titania",
            BodyId::Oberon => "Oberon",
            BodyId::Triton => "Triton",
            BodyId::Nereid => "Nereid",
            BodyId::Proteus => "Proteus",
            BodyId::Phobos => "Phobos",
            BodyId::Deimos => "Deimos",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        BODIES
            .iter()
            .map(|body| body.body_id)
            .find(|body_id| body_id.name() == name)
    }

    pub const fn radius_km(self) -> f64 {
        match self {
            BodyId::Sun => 696_340.0,
            BodyId::Mercury => 2_439.7,
            BodyId::Venus => 6_051.8,
            BodyId::Earth => 6_371.01,
            BodyId::Moon => 1_737.53,
            BodyId::Mars => 3_389.5,
            BodyId::Jupiter => 69_911.0,
            BodyId::Saturn => 58_232.0,
            BodyId::Uranus => 25_362.0,
            BodyId::Neptune => 24_622.0,
            BodyId::Io => 1_821.6,
            BodyId::Europa => 1_560.8,
            BodyId::Ganymede => 2_634.1,
            BodyId::Callisto => 2_410.3,
            BodyId::Mimas => 198.2,
            BodyId::Enceladus => 252.1,
            BodyId::Tethys => 531.1,
            BodyId::Dione => 561.4,
            BodyId::Rhea => 763.8,
            BodyId::Titan => 2_574.7,
            BodyId::Iapetus => 734.5,
            BodyId::Miranda => 235.8,
            BodyId::Ariel => 578.9,
            BodyId::Umbriel => 584.7,
            BodyId::Titania => 788.9,
            BodyId::Oberon => 761.4,
            BodyId::Triton => 1_353.4,
            BodyId::Nereid => 170.0,
            BodyId::Proteus => 210.0,
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
            BodyId::Jupiter => 10.0,
            BodyId::Saturn => 9.0,
            BodyId::Uranus => 8.0,
            BodyId::Neptune => 8.0,
            BodyId::Io => 3.0,
            BodyId::Europa => 3.0,
            BodyId::Ganymede => 4.0,
            BodyId::Callisto => 4.0,
            BodyId::Mimas => 2.0,
            BodyId::Enceladus => 2.0,
            BodyId::Tethys => 2.0,
            BodyId::Dione => 2.0,
            BodyId::Rhea => 3.0,
            BodyId::Titan => 4.0,
            BodyId::Iapetus => 3.0,
            BodyId::Miranda => 2.0,
            BodyId::Ariel => 2.0,
            BodyId::Umbriel => 2.0,
            BodyId::Titania => 3.0,
            BodyId::Oberon => 3.0,
            BodyId::Triton => 3.0,
            BodyId::Nereid => 2.0,
            BodyId::Proteus => 2.0,
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
            BodyId::Jupiter => (214, 176, 127),
            BodyId::Saturn => (220, 198, 140),
            BodyId::Uranus => (152, 218, 232),
            BodyId::Neptune => (95, 130, 235),
            BodyId::Io => (236, 210, 120),
            BodyId::Europa => (207, 192, 166),
            BodyId::Ganymede => (150, 136, 126),
            BodyId::Callisto => (126, 116, 104),
            BodyId::Mimas => (170, 164, 154),
            BodyId::Enceladus => (214, 220, 226),
            BodyId::Tethys => (196, 194, 188),
            BodyId::Dione => (188, 180, 170),
            BodyId::Rhea => (174, 170, 162),
            BodyId::Titan => (198, 160, 108),
            BodyId::Iapetus => (128, 116, 100),
            BodyId::Miranda => (176, 166, 154),
            BodyId::Ariel => (202, 196, 190),
            BodyId::Umbriel => (148, 142, 136),
            BodyId::Titania => (170, 164, 156),
            BodyId::Oberon => (158, 150, 142),
            BodyId::Triton => (201, 196, 186),
            BodyId::Nereid => (166, 160, 152),
            BodyId::Proteus => (138, 132, 126),
            BodyId::Phobos => (138, 126, 116),
            BodyId::Deimos => (168, 160, 146),
        }
    }

    pub const fn sprite_stem(self) -> &'static str {
        match self {
            BodyId::Sun => "sun",
            BodyId::Mercury => "mercury",
            BodyId::Venus => "venus",
            BodyId::Earth => "earth",
            BodyId::Moon => "moon",
            BodyId::Mars => "mars",
            BodyId::Jupiter => "jupiter",
            BodyId::Saturn => "saturn",
            BodyId::Uranus => "uranus",
            BodyId::Neptune => "neptune",
            BodyId::Io => "io",
            BodyId::Europa => "europa",
            BodyId::Ganymede => "ganymede",
            BodyId::Callisto => "callisto",
            BodyId::Mimas => "mimas",
            BodyId::Enceladus => "enceladus",
            BodyId::Tethys => "tethys",
            BodyId::Dione => "dione",
            BodyId::Rhea => "rhea",
            BodyId::Titan => "titan",
            BodyId::Iapetus => "iapetus",
            BodyId::Miranda => "miranda",
            BodyId::Ariel => "ariel",
            BodyId::Umbriel => "umbriel",
            BodyId::Titania => "titania",
            BodyId::Oberon => "oberon",
            BodyId::Triton => "triton",
            BodyId::Nereid => "nereid",
            BodyId::Proteus => "proteus",
            BodyId::Phobos => "phobos",
            BodyId::Deimos => "deimos",
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
    BodyMeta { body_id: BodyId::Jupiter },
    BodyMeta { body_id: BodyId::Saturn },
    BodyMeta { body_id: BodyId::Uranus },
    BodyMeta { body_id: BodyId::Neptune },
    BodyMeta { body_id: BodyId::Io },
    BodyMeta { body_id: BodyId::Europa },
    BodyMeta { body_id: BodyId::Ganymede },
    BodyMeta { body_id: BodyId::Callisto },
    BodyMeta { body_id: BodyId::Mimas },
    BodyMeta {
        body_id: BodyId::Enceladus,
    },
    BodyMeta { body_id: BodyId::Tethys },
    BodyMeta { body_id: BodyId::Dione },
    BodyMeta { body_id: BodyId::Rhea },
    BodyMeta { body_id: BodyId::Titan },
    BodyMeta { body_id: BodyId::Iapetus },
    BodyMeta { body_id: BodyId::Miranda },
    BodyMeta { body_id: BodyId::Ariel },
    BodyMeta { body_id: BodyId::Umbriel },
    BodyMeta { body_id: BodyId::Titania },
    BodyMeta { body_id: BodyId::Oberon },
    BodyMeta { body_id: BodyId::Triton },
    BodyMeta { body_id: BodyId::Nereid },
    BodyMeta { body_id: BodyId::Proteus },
    BodyMeta { body_id: BodyId::Phobos },
    BodyMeta { body_id: BodyId::Deimos },
];

#[cfg(test)]
mod tests {
    use super::BodyId;

    #[test]
    fn resolves_body_names() {
        assert_eq!(BodyId::from_name("Earth"), Some(BodyId::Earth));
        assert_eq!(BodyId::from_name(" unknown "), None);
    }
}

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
