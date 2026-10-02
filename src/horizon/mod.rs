//! Client und Parser für die [JPL Horizons REST-API](https://ssd.jpl.nasa.gov/horizons/).
//!
//! Dieses Modul bietet Funktionen zum Abrufen und Aufbereiten von Ephemeriden-Daten
//! (Positions- und Geschwindigkeitsvektoren) für Himmelskörper im Sonnensystem.
use crate::Ephem;
use chrono::NaiveDateTime;
use serde::Deserialize;

// Die fehlende Struct definieren, damit Serde das JSON-Feld "result" findet
#[derive(Deserialize)]
struct HorizonsJsonResponse {
    result: String,
}

/// Lädt Ephemeriden für ein angegebenes Objekt herunter und parst diese.
///
/// # Argumente
///
/// * `client` - Der wiederverwendbare HTTP-Client.
/// * `command` - Der JPL-Objektbezeichner (z. B. `"399"` für die Erde).
///
/// # Fehler
///
/// Gibt einen Fehler zurück, wenn:
/// * Der HTTP-Request fehlschlägt.
/// * Die NASA-API eine Fehlermeldung im Antworttext zurückliefert.
/// * Das Parsen des Textinhalts oder der Datumsangaben fehlschlägt.
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    // Hilfs-Funktion, um eine minimale, valide NASA-Antwort zu simulieren
    fn create_mock_json(result_text: &str) -> String {
        format!(
            r#"{{"signature":{{"source":"NASA","version":"1.2"}},"result":"{}"}}"#,
            result_text.replace('\n', "\\n") // escape Newlines für valides JSON
        )
    }

    #[test]
    fn test_parse_valid_ephemerides() {
        // Ein echter Datenblock (1 Tag) aus der NASA Response
        let mock_result = "\
*******************************************************************************
$$SOE
2461314.500000000 = A.D. 2026-Oct-01 00:00:00.0000 TDB
 X = 1.485257166019058E+08 Y = 1.946860462820820E+07 Z =-2.483661492046900E+03
 VX=-4.343910994914796E+00 VY= 2.941844372763652E+01 VZ=-1.658292238307268E-03
$$EOE
*******************************************************************************";

        let json_data = create_mock_json(mock_result);
        let result = parse_horizons_response(&json_data, "399");

        // Prüfen, ob das Parsen erfolgreich war
        assert!(result.is_ok(), "Parser sollte bei validen Daten Ok zurückgeben");
        let ephems = result.unwrap();

        // Prüfen, ob genau ein Datensatz extrahiert wurde
        assert_eq!(ephems.len(), 1);

        let data = &ephems[0];
        assert_eq!(data.body, "399");

        // Datum prüfen (Sollte exakt der 01. Oktober 2026 sein)
        let expected_date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap().and_hms_micro_opt(0, 0, 0, 0).unwrap();
        assert_eq!(data.date, expected_date);

        // Physikalische Werte prüfen
        assert_eq!(data.x, 148525716.6019058);
        assert_eq!(data.vx, -4.343910994914796);
    }

    #[test]
    fn test_parse_nasa_error_response() {
        // Ein simulierter Fehlertext der NASA (z.B. bei falschem Planeten-Code)
        let mock_result = "ERROR: Missing COMMAND specification\\n For system help, send email to...";
        let json_data = create_mock_json(mock_result);

        let result = parse_horizons_response(&json_data, "INVALID");

        // Der Parser muss hier ein Err zurückliefern!
        assert!(result.is_err(), "Parser hätte bei einer Fehlermeldung abbrechen müssen");

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("NASA API Fehler"), "Fehlermeldung war unerwartet: {}", error_msg);
    }

    #[test]
    fn test_parse_empty_section() {
        // Was passiert, wenn die SOE/EOE Tags da sind, aber keine Zeilen dazwischen?
        let mock_result = "$$SOE\n$$EOE";
        let json_data = create_mock_json(mock_result);

        let result = parse_horizons_response(&json_data, "399");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0, "Vektor sollte bei leeren SOE-Tags leer sein");
    }
}
