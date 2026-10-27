//! Lädt und speichert den benutzerdefinierten Horizons-Datumsbereich.
//!
//! Die Einstellungen liegen als `settings.json` im plattformspezifischen
//! Konfigurationsverzeichnis der Anwendung.
use crate::horizon::HorizonsDateRange;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct StoredSettings {
    /// Startdatum im ISO-Format `YYYY-MM-DD`.
    start_date: String,
    /// Enddatum im ISO-Format `YYYY-MM-DD`.
    stop_date: String,
}

/// Lädt den gespeicherten Horizons-Datumsbereich.
///
/// Fehlt die Einstellungsdatei oder kann sie nicht gelesen, deserialisiert
/// oder als gültiger Datumsbereich interpretiert werden, wird der
/// Standardzeitraum zurückgegeben. Fehlerhafte oder nicht lesbare Einstellungen
/// werden protokolliert.
pub fn load_date_range() -> HorizonsDateRange {
    let Some(path) = settings_file_path() else {
        log::warn!("Kein User-Konfigurationsverzeichnis gefunden; nutze Standardzeitraum");
        return HorizonsDateRange::default();
    };

    let stored = match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<StoredSettings>(&content) {
            Ok(settings) => settings,
            Err(error) => {
                log::warn!("Horizons-Einstellungen sind ungültig; nutze Standardzeitraum: {error}");
                return HorizonsDateRange::default();
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return HorizonsDateRange::default();
        }
        Err(error) => {
            log::warn!("Horizons-Einstellungen konnten nicht gelesen werden; nutze Standardzeitraum: {error}");
            return HorizonsDateRange::default();
        }
    };

    match HorizonsDateRange::from_iso_dates(&stored.start_date, &stored.stop_date) {
        Ok(date_range) => date_range,
        Err(error) => {
            log::warn!("Horizons-Einstellungen sind ungültig; nutze Standardzeitraum: {error}");
            HorizonsDateRange::default()
        }
    }
}

/// Speichert den Horizons-Datumsbereich als JSON in der Benutzerkonfiguration.
///
/// Das Konfigurationsverzeichnis wird bei Bedarf angelegt.
///
/// # Fehler
///
/// Gibt einen Fehler zurück, wenn kein Konfigurationspfad ermittelt werden
/// kann, das Verzeichnis nicht angelegt werden kann oder die Einstellungen
/// nicht serialisiert beziehungsweise geschrieben werden können.
pub fn save_date_range(date_range: HorizonsDateRange) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_file_path().ok_or("Kein User-Konfigurationsverzeichnis gefunden")?;
    let parent = path.parent().ok_or("Ungültiger Pfad für Horizons-Einstellungen")?;
    std::fs::create_dir_all(parent)?;

    let stored = StoredSettings {
        start_date: date_range.start_time(),
        stop_date: date_range.stop_time(),
    };
    std::fs::write(path, serde_json::to_vec_pretty(&stored)?)?;
    Ok(())
}

/// Ermittelt den plattformspezifischen Pfad zur Einstellungsdatei.
fn settings_file_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "xbsol", "xbsol").map(|directories| directories.config_dir().join("settings.json"))
}
