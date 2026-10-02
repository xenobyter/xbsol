// Zugriffe auf die Horizon API des JPL (https://ssd.jpl.nasa.gov/horizons/).
use crate::Ephem;
use chrono::NaiveDateTime;
use serde::Deserialize;

// Die fehlende Struct definieren, damit Serde das JSON-Feld "result" findet
#[derive(Deserialize)]
struct HorizonsJsonResponse {
    result: String,
}

pub async fn get_ephemerides(client: &reqwest::Client, command: &str) -> Result<Vec<Ephem>, Box<dyn std::error::Error>> {
    let data = get_horizons_data(client, command).await?;
    // Mit dem '?' extrahieren wir das Result aus dem Parser und werfen Fehler hoch
    let parsed_data = parse_horizons_response(&data, command)?;
    Ok(parsed_data)
}

async fn get_horizons_data(client: &reqwest::Client, command: &str) -> Result<String, reqwest::Error> {
    let url = "https://ssd.jpl.nasa.gov/api/horizons.api";

    let params = [
        ("format", "json"),
        ("COMMAND", command),
        ("CENTER", "500@10"),
        ("MAKE_EPHEM", "YES"),
        ("OBJ_DATA", "NO"),
        ("EPHEM_TYPE", "VECTORS"),
        ("START_TIME", "2026-10-01"),
        ("STOP_TIME", "2026-10-08"),
        ("STEP_SIZE", "1 d"),
        ("VEC_TABLE", "2"),
        ("OUT_UNITS", "KM-S"),
        ("REF_PLANE", "ECLIPTIC"),
    ];
    let response = client.get(url).query(&params).send().await?;

    let text = response.text().await?;
    log::debug!("Horizons API Response für {}:\n{}", command, text);
    Ok(text)
}

fn parse_horizons_response(json_data: &str, command: &str) -> Result<Vec<Ephem>, Box<dyn std::error::Error>> {
    let response: HorizonsJsonResponse = serde_json::from_str(json_data)?;

    // Schneller Abbruch, falls die NASA eine Fehlermeldung schickt
    if response.result.contains("ERROR:") || response.result.contains("Ambiguous") || response.result.contains("Missing") {
        let first_line = response.result.lines().next().unwrap_or("Unbekannter Fehler");
        return Err(format!("NASA API Fehler für '{}': {}", command, first_line).into());
    }

    // Hilfsfunktion zum Bereinigen und Parsen der 3D-Vektoren
    let parse_vector = |line: &str, chars_to_remove: &[char]| -> Option<(f64, f64, f64)> {
        let clean = line.replace(chars_to_remove, "");
        let mut tokens = clean.split_whitespace();

        let x = tokens.next()?.parse::<f64>().ok()?;
        let y = tokens.next()?.parse::<f64>().ok()?;
        let z = tokens.next()?.parse::<f64>().ok()?;

        Some((x, y, z))
    };

    let lines: Vec<&str> = response
        .result
        .lines()
        .map(|line| line.trim())
        .skip_while(|line| !line.starts_with("$$SOE")) // Überspringe alles vor $$SOE
        .skip(1) // Überspringe die $$SOE-Zeile selbst
        .take_while(|line| !line.starts_with("$$EOE")) // Nimm alle Zeilen bis $$EOE
        .filter(|line| !line.is_empty()) // Ignoriere leere Zeilen
        .collect();

    // 3er-Gruppen verarbeiten und fehlerhafte Datensätze direkt filtern
    let ephemerides: Vec<Ephem> = lines
        .chunks(3)
        .filter_map(|chunk| {
            // Versuche, den Chunk in ein festes 3er-Array umzuwandeln
            let [line_date, line_pos, line_vel] = <[&str; 3]>::try_from(chunk).ok()?;

            // Datum nach dem " = " extrahieren
            let clean_date = line_date
                .split('=')
                .nth(1)?
                .trim()
                .trim_start_matches("A.D. ")
                .split("TDB")
                .next()?
                .split("UT")
                .next()?
                .trim();

            // Datum parsen
            let date = NaiveDateTime::parse_from_str(clean_date, "%Y-%b-%d %H:%M:%S%.f")
                .unwrap_or_else(|_| chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc());

            // Vektoren extrahieren
            let (x, y, z) = parse_vector(line_pos, &['X', 'Y', 'Z', '='])?;
            let (vx, vy, vz) = parse_vector(line_vel, &['V', 'X', 'Y', 'Z', '='])?;

            Some(Ephem {
                date,
                body: command.to_string(),
                x,
                y,
                z,
                vx,
                vy,
                vz,
            })
        })
        .collect();

    Ok(ephemerides)
}
