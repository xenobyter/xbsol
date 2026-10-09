mod horizon;
mod model;
use horizon::get_ephemerides;
use model::{BODIES, EphemerisTable};

fn main() {
    env_logger::init();

    let client = reqwest::Client::new();
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let mut table = EphemerisTable::new();

    for body in BODIES {
        match runtime.block_on(get_ephemerides(&client, *body)) {
            Ok(data) => {
                println!(
                    "Horizon data for {} ({}) fetched successfully: {} records.",
                    body.body_id.name(),
                    body.command(),
                    data.len()
                );
                table.extend(data);
            }
            Err(error) => {
                eprintln!("Failed to fetch Horizon data for {}: {error}", body.body_id.name());
                std::process::exit(1);
            }
        }
    }

    println!("Total in-memory ephemeris rows: {}", table.rows.len());

    let day_index = table.build_day_index();
    println!("Indexed days: {}", day_index.len());
}
