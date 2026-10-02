mod horizon;
mod model;
use horizon::get_ephemerides;
use model::BODIES;

use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Ephem {
    pub date: NaiveDateTime,
    pub body: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

#[derive(Debug)]
pub struct Body {
    pub id: &'static str,
    pub name: &'static str,
    pub radius: f64,
}

fn main() {
    env_logger::init();

    let client = reqwest::Client::new();
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    for body in BODIES {
        match runtime.block_on(get_ephemerides(&client, &body.id)) {
            Ok(data) => {
                println!("Horizon data for {} fetched successfully.", body.name);
                println!("{:#?}", data);
            }
            Err(error) => {
                eprintln!("Failed to fetch Horizon data for {}: {error}", body.name);
                std::process::exit(1);
            }
        }
    }
}
