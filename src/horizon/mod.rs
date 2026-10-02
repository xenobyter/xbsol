// Zugriffe auf die Horizon API des JPL (https://ssd.jpl.nasa.gov/horizons/).
use crate::Ephem;
use chrono::NaiveDateTime;
use serde::Deserialize;

// 1. Die fehlende Struct definieren, damit Serde das JSON-Feld "result" findet
#[derive(Deserialize)]
struct HorizonsJsonResponse {
    result: String,
}

// 2. Den Fehler-Typ im Rückgabewert flexibler gestalten (Box<dyn std::error::Error>)
// Da reqwest::Error und serde_json::Error aufeinandertreffen, ist dies der sauberste Weg.
pub async fn get(
    client: &reqwest::Client,
    command: &str,
) -> Result<Vec<Ephem>, Box<dyn std::error::Error>> {
    let data = get_horizons_data(client, command).await?;
    // Mit dem '?' extrahieren wir das Result aus dem Parser und werfen Fehler hoch
    let parsed_data = parse_horizons_response(&data, command)?;
    Ok(parsed_data)
}

async fn get_horizons_data(
    client: &reqwest::Client,
    command: &str,
) -> Result<String, reqwest::Error> {
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

    response.text().await
}

fn parse_horizons_response(
    json_data: &str,
    command: &str,
) -> Result<Vec<Ephem>, serde_json::Error> {
    let response: HorizonsJsonResponse = serde_json::from_str(json_data)?;

    let mut ephemerides = Vec::new();
    let mut is_ephemeris_section = false;
    let mut section_lines = Vec::new();

    for line in response.result.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("$$SOE") {
            is_ephemeris_section = true;
            continue;
        } else if trimmed.starts_with("$$EOE") {
            break; // Beendet die Schleife komplett (löst auch die Compiler-Warnung)
        }

        if is_ephemeris_section && !trimmed.is_empty() {
            section_lines.push(trimmed);
        }
    }

    // Zeilen in 3er-Gruppen verarbeiten
    for chunk in section_lines.chunks(3) {
        if chunk.len() < 3 {
            break;
        }

        let line_date = chunk[0];
        let line_pos = chunk[1];
        let line_vel = chunk[2];

        // Datum nach dem " = " extrahieren
        let date_str = line_date.split('=').nth(1).unwrap_or("").trim();
        // 1. Unwichtige NASA-Zusätze entfernen, damit das Pattern exakt passt
        let clean_date = date_str
            .trim_start_matches("A.D. ")
            .trim_end_matches(" TDB")
            .trim();

        // 2. Parsen mit dem neuen Fallback
        let date = NaiveDateTime::parse_from_str(clean_date, "%Y-%b-%d %H:%M:%S.%f")
            .unwrap_or_else(|_| chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc());

        // --- POSITIONEN (X, Y, Z) PARSEN ---
        // Wir säubern die Zeile von "X", "Y", "Z" und "="
        let clean_pos = line_pos
            .replace("X", "")
            .replace("Y", "")
            .replace("Z", "")
            .replace("=", "");
        let pos_tokens: Vec<&str> = clean_pos.split_whitespace().collect();
        if pos_tokens.len() < 3 {
            continue;
        }

        let x = pos_tokens[0].parse::<f64>().unwrap_or(0.0);
        let y = pos_tokens[1].parse::<f64>().unwrap_or(0.0);
        let z = pos_tokens[2].parse::<f64>().unwrap_or(0.0);

        // --- GESCHWINDIGKEITEN (VX, VY, VZ) PARSEN ---
        // Wir säubern die Zeile von "VX", "VY", "VZ" und "="
        let clean_vel = line_vel
            .replace("VX", "")
            .replace("VY", "")
            .replace("VZ", "")
            .replace("=", "");
        let vel_tokens: Vec<&str> = clean_vel.split_whitespace().collect();
        if vel_tokens.len() < 3 {
            continue;
        }

        let vx = vel_tokens[0].parse::<f64>().unwrap_or(0.0);
        let vy = vel_tokens[1].parse::<f64>().unwrap_or(0.0);
        let vz = vel_tokens[2].parse::<f64>().unwrap_or(0.0);

        ephemerides.push(Ephem {
            date: date,
            body: command.to_string(),
            x,
            y,
            z,
            vx,
            vy,
            vz,
        });
    }

    Ok(ephemerides)
}
