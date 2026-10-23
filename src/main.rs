mod app_state;
mod horizon;
mod model;
mod settings;
mod ui;
use horizon::get_ephemerides_for_bodies;

const EXIT_RUNTIME_ERROR: i32 = 1;

fn main() {
    env_logger::init();

    let client = reqwest::Client::new();
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let date_range = settings::load_date_range();

    let table = match runtime.block_on(get_ephemerides_for_bodies(&client, date_range)) {
        Ok(table) => table,
        Err(error) => {
            eprintln!("Failed to fetch Horizon data: {error}");
            std::process::exit(EXIT_RUNTIME_ERROR);
        }
    };

    println!("Total in-memory ephemeris rows: {}", table.rows.len());

    let day_index = table.build_day_index();
    println!("Indexed days: {}", day_index.len());

    if let Err(error) = ui::run_ui(day_index, date_range, client, runtime.handle().clone()) {
        eprintln!("Failed to run UI: {error}");
        std::process::exit(EXIT_RUNTIME_ERROR);
    }
}
