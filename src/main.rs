mod horizon;

use chrono::NaiveDateTime;
use horizon::get_ephemerides;

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

fn main() {
    env_logger::init();

    let client = reqwest::Client::new();
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    match runtime.block_on(get_ephemerides(&client, "399")) {
        Ok(data) => {
            println!("Horizon data fetched successfully.");
            println!("{:#?}", data);
        }
        Err(error) => {
            eprintln!("Failed to fetch Horizon data: {error}");
            std::process::exit(1);
        }
    }
}
