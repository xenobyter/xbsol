//! Client und Parser für die [JPL Horizons REST-API](https://ssd.jpl.nasa.gov/horizons/).
//!
//! Dieses Modul bietet Funktionen zum Abrufen und Aufbereiten von Ephemeriden-Daten
//! (Positions- und Geschwindigkeitsvektoren) für Himmelskörper im Sonnensystem.
use crate::model::{BodyMeta, EphemerisRecord, StateVector};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::fmt;
use std::path::{Path, PathBuf};

const HORIZONS_CENTER: &str = "500@10";
const HORIZONS_REF_PLANE: &str = "ECLIPTIC";
const HORIZONS_STEP_SIZE: &str = "1 d";
const HORIZONS_START_TIME: &str = "2026-01-01";
const HORIZONS_STOP_TIME: &str = "2026-12-31";
const HORIZONS_CACHE_DIR: &str = "cache/horizons";

#[derive(Debug)]
enum HorizonsParseError {
    ApiError { command: String, first_line: String },
    MissingSoeEoeMarkers,
    IncompleteChunk { line_count: usize },
    InvalidDateLine { line: String },
    InvalidDateFormat { value: String, source: chrono::ParseError },
    InvalidVectorLine { line: String },
    InvalidVectorValue { token: String, source: std::num::ParseFloatError },
}

impl fmt::Display for HorizonsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApiError { command, first_line } => {
                write!(f, "NASA API Fehler für '{}': {}", command, first_line)
            }
            Self::MissingSoeEoeMarkers => write!(f, "Horizons-Response enthält keine $$SOE/$$EOE Marker"),
            Self::IncompleteChunk { line_count } => write!(f, "Unvollständiger Datensatzblock mit {} Zeilen", line_count),
            Self::InvalidDateLine { line } => write!(f, "Ungültige Datumszeile: {}", line),
            Self::InvalidDateFormat { value, .. } => write!(f, "Ungültiges Datumsformat: {}", value),
            Self::InvalidVectorLine { line } => write!(f, "Ungültige Vektorzeile: {}", line),
            Self::InvalidVectorValue { token, .. } => write!(f, "Ungültiger numerischer Wert in Vektorzeile: {}", token),
        }
    }
}

impl std::error::Error for HorizonsParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidDateFormat { source, .. } => Some(source),
            Self::InvalidVectorValue { source, .. } => Some(source),
            _ => None,
        }
    }
}

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
/// * `body` - Metadaten zum Himmelskörper (inkl. JPL-Objektbezeichner).
///
/// # Fehler
///
/// Gibt einen Fehler zurück, wenn:
/// * Der HTTP-Request fehlschlägt.
/// * Die NASA-API eine Fehlermeldung im Antworttext zurückliefert.
/// * Das Parsen des Textinhalts oder der Datumsangaben fehlschlägt.
pub async fn get_ephemerides(client: &reqwest::Client, body: BodyMeta) -> Result<Vec<EphemerisRecord>, Box<dyn std::error::Error>> {
    let command = body.command();

    if let Some(cached_json) = load_cached_horizons_data(command)? {
        log::info!(
            "Nutze lokalen Horizons-Cache für COMMAND={} im Zeitraum {} bis {}",
            command,
            HORIZONS_START_TIME,
            HORIZONS_STOP_TIME
        );
        let parsed_data = parse_horizons_response(&cached_json, body)?;
        return Ok(parsed_data);
    }

    let data = get_horizons_data(client, command).await?;
    // Mit dem '?' extrahieren wir das Result aus dem Parser und werfen Fehler hoch
    let parsed_data = parse_horizons_response(&data, body)?;

    if let Err(error) = save_cached_horizons_data(command, &data) {
        log::warn!("Konnte Horizons-Cache nicht speichern für COMMAND={}: {}", command, error);
    }

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
        ("START_TIME", HORIZONS_START_TIME),
        ("STOP_TIME", HORIZONS_STOP_TIME),
        ("STEP_SIZE", HORIZONS_STEP_SIZE),
        ("VEC_TABLE", "2"),
        ("OUT_UNITS", "KM-S"),
        ("REF_PLANE", HORIZONS_REF_PLANE),
    ];
    let response = client.get(url).query(&params).send().await?;

    let text = response.text().await?;
    log::debug!("Horizons API Response für {}:\n{}", command, text);
    Ok(text)
}

fn load_cached_horizons_data(command: &str) -> Result<Option<String>, std::io::Error> {
    let path = cache_file_path(command);
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    Ok(Some(content))
}

fn save_cached_horizons_data(command: &str, content: &str) -> Result<(), std::io::Error> {
    let path = cache_file_path(command);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

fn cache_file_path(command: &str) -> PathBuf {
    Path::new(HORIZONS_CACHE_DIR).join(format!(
        "{}_{}_{}.json",
        sanitize_for_file_name(command),
        sanitize_for_file_name(HORIZONS_START_TIME),
        sanitize_for_file_name(HORIZONS_STOP_TIME),
    ))
}

fn sanitize_for_file_name(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn parse_horizons_response(json_data: &str, body: BodyMeta) -> Result<Vec<EphemerisRecord>, Box<dyn std::error::Error>> {
    let response: HorizonsJsonResponse = serde_json::from_str(json_data)?;
    let command = body.command();

    ensure_no_api_error(&response.result, command)?;
    let lines = extract_data_lines(&response.result)?;

    let mut ephemerides = Vec::new();
    for chunk in lines.chunks(3) {
        if chunk.len() != 3 {
            return Err(HorizonsParseError::IncompleteChunk { line_count: chunk.len() }.into());
        }

        let record = parse_record_chunk(body, chunk[0], chunk[1], chunk[2])?;
        ephemerides.push(record);
    }

    Ok(ephemerides)
}

fn ensure_no_api_error(result: &str, command: &str) -> Result<(), HorizonsParseError> {
    if result.contains("ERROR:") || result.contains("Ambiguous") || result.contains("Missing") {
        let first_line = result.lines().next().unwrap_or("Unbekannter Fehler").to_string();
        return Err(HorizonsParseError::ApiError {
            command: command.to_string(),
            first_line,
        });
    }
    Ok(())
}

fn extract_data_lines(result: &str) -> Result<Vec<&str>, HorizonsParseError> {
    let mut in_data = false;
    let mut seen_soe = false;
    let mut seen_eoe = false;
    let mut lines = Vec::new();

    for line in result.lines().map(|l| l.trim()) {
        if line.starts_with("$$SOE") {
            seen_soe = true;
            in_data = true;
            continue;
        }
        if line.starts_with("$$EOE") {
            seen_eoe = true;
            break;
        }
        if in_data && !line.is_empty() {
            lines.push(line);
        }
    }

    if !seen_soe || !seen_eoe {
        return Err(HorizonsParseError::MissingSoeEoeMarkers);
    }

    Ok(lines)
}

fn parse_record_chunk(body: BodyMeta, line_date: &str, line_pos: &str, line_vel: &str) -> Result<EphemerisRecord, HorizonsParseError> {
    let epoch = parse_epoch(line_date)?;
    let (x, y, z) = parse_vector3(line_pos, &['X', 'Y', 'Z', '='])?;
    let (vx, vy, vz) = parse_vector3(line_vel, &['V', 'X', 'Y', 'Z', '='])?;

    Ok(EphemerisRecord {
        body_id: body.body_id,
        center: HORIZONS_CENTER.to_string(),
        ref_plane: HORIZONS_REF_PLANE.to_string(),
        step: HORIZONS_STEP_SIZE.to_string(),
        vector: StateVector {
            epoch,
            x_km: x,
            y_km: y,
            z_km: z,
            vx_km_s: vx,
            vy_km_s: vy,
            vz_km_s: vz,
        },
    })
}

fn parse_epoch(line_date: &str) -> Result<DateTime<Utc>, HorizonsParseError> {
    let clean_date = line_date
        .split('=')
        .nth(1)
        .ok_or_else(|| HorizonsParseError::InvalidDateLine {
            line: line_date.to_string(),
        })?
        .trim()
        .trim_start_matches("A.D. ")
        .split("TDB")
        .next()
        .ok_or_else(|| HorizonsParseError::InvalidDateLine {
            line: line_date.to_string(),
        })?
        .split("UT")
        .next()
        .ok_or_else(|| HorizonsParseError::InvalidDateLine {
            line: line_date.to_string(),
        })?
        .trim()
        .to_string();

    let date = NaiveDateTime::parse_from_str(&clean_date, "%Y-%b-%d %H:%M:%S%.f").map_err(|source| HorizonsParseError::InvalidDateFormat {
        value: clean_date,
        source,
    })?;

    Ok(DateTime::<Utc>::from_naive_utc_and_offset(date, Utc))
}

fn parse_vector3(line: &str, chars_to_remove: &[char]) -> Result<(f64, f64, f64), HorizonsParseError> {
    let clean = line.replace(chars_to_remove, "");
    let mut tokens = clean.split_whitespace();

    let x_token = tokens.next().ok_or_else(|| HorizonsParseError::InvalidVectorLine {
        line: line.to_string(),
    })?;
    let y_token = tokens.next().ok_or_else(|| HorizonsParseError::InvalidVectorLine {
        line: line.to_string(),
    })?;
    let z_token = tokens.next().ok_or_else(|| HorizonsParseError::InvalidVectorLine {
        line: line.to_string(),
    })?;

    let x = x_token
        .parse::<f64>()
        .map_err(|source| HorizonsParseError::InvalidVectorValue {
            token: x_token.to_string(),
            source,
        })?;
    let y = y_token
        .parse::<f64>()
        .map_err(|source| HorizonsParseError::InvalidVectorValue {
            token: y_token.to_string(),
            source,
        })?;
    let z = z_token
        .parse::<f64>()
        .map_err(|source| HorizonsParseError::InvalidVectorValue {
            token: z_token.to_string(),
            source,
        })?;

    Ok((x, y, z))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BodyId;
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
        let result = parse_horizons_response(&json_data, BodyMeta { body_id: BodyId::Earth });

        // Prüfen, ob das Parsen erfolgreich war
        assert!(result.is_ok(), "Parser sollte bei validen Daten Ok zurückgeben");
        let ephems = result.unwrap();

        // Prüfen, ob genau ein Datensatz extrahiert wurde
        assert_eq!(ephems.len(), 1);

        let data = &ephems[0];
        assert_eq!(data.body_id, BodyId::Earth);

        // Datum prüfen (Sollte exakt der 01. Oktober 2026 sein)
        let expected_date = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap().and_hms_micro_opt(0, 0, 0, 0).unwrap();
        assert_eq!(data.vector.epoch.naive_utc(), expected_date);

        // Physikalische Werte prüfen
        assert_eq!(data.vector.x_km, 148525716.6019058);
        assert_eq!(data.vector.vx_km_s, -4.343910994914796);
    }

    #[test]
    fn test_parse_nasa_error_response() {
        // Ein simulierter Fehlertext der NASA (z.B. bei falschem Planeten-Code)
        let mock_result = "ERROR: Missing COMMAND specification\\n For system help, send email to...";
        let json_data = create_mock_json(mock_result);

        let result = parse_horizons_response(&json_data, BodyMeta { body_id: BodyId::Earth });

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

        let result = parse_horizons_response(&json_data, BodyMeta { body_id: BodyId::Earth });

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0, "Vektor sollte bei leeren SOE-Tags leer sein");
    }
}
