use crate::models::ParsedResultEntry;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResultsError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone)]
pub struct ParsedSession {
    pub session_type: String,
    pub track: String,
    pub entries: Vec<ParsedResultEntry>,
}

#[derive(Debug, Deserialize)]
struct RawResults {
    #[serde(rename = "TrackName", alias = "track", alias = "Track")]
    track_name: Option<String>,
    #[serde(rename = "Type", alias = "type", alias = "SessionType")]
    session_type: Option<String>,
    #[serde(rename = "Cars", alias = "cars", alias = "Result")]
    cars: Option<Vec<RawCar>>,
    #[serde(rename = "Result", alias = "results")]
    result: Option<Vec<RawCar>>,
}

#[derive(Debug, Deserialize)]
struct RawCar {
    #[serde(rename = "DriverName", alias = "driverName", alias = "Name")]
    driver_name: Option<String>,
    #[serde(rename = "DriverGuid", alias = "driverGuid", alias = "Guid", alias = "SteamID")]
    driver_guid: Option<String>,
    #[serde(rename = "CarModel", alias = "Model", alias = "car")]
    car_model: Option<String>,
    #[serde(rename = "RacePosition", alias = "Position", alias = "Pos")]
    race_position: Option<i64>,
    #[serde(rename = "BestLap", alias = "bestLap")]
    best_lap: Option<i64>,
    #[serde(rename = "NumLaps", alias = "Laps", alias = "laps")]
    num_laps: Option<i64>,
    #[serde(rename = "TotalTime", alias = "totalTime")]
    total_time: Option<i64>,
}

pub fn parse_results_json(raw: &str, file_name: Option<&str>) -> Result<ParsedSession, ResultsError> {
    let doc: RawResults =
        serde_json::from_str(raw).map_err(|e| ResultsError::Parse(e.to_string()))?;

    let cars = doc
        .cars
        .or(doc.result)
        .ok_or_else(|| ResultsError::Parse("no Cars/Result array in JSON".into()))?;

    if cars.is_empty() {
        return Err(ResultsError::Message(
            "session had zero drivers — file may be empty".into(),
        ));
    }

    let session_type = doc
        .session_type
        .or_else(|| file_name.and_then(session_type_from_filename))
        .unwrap_or_else(|| "R".to_string());

    let track = doc
        .track_name
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let mut entries: Vec<ParsedResultEntry> = cars
        .into_iter()
        .enumerate()
        .map(|(i, car)| {
            let laps = car.num_laps.unwrap_or(0).max(0) as u32;
            let position = car
                .race_position
                .map(|p| p.max(1) as u32)
                .or(Some((i + 1) as u32));
            let best_lap = car.best_lap.filter(|t| *t > 0);
            ParsedResultEntry {
                driver_name: car
                    .driver_name
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| format!("Driver {}", i + 1)),
                driver_guid: car.driver_guid.filter(|g| !g.is_empty()),
                car_model: car.car_model,
                position,
                best_lap_ms: best_lap,
                laps,
                total_time_ms: car.total_time.filter(|t| *t > 0),
                dnf: laps == 0,
            }
        })
        .collect();

    if entries.iter().all(|e| e.laps == 0) {
        return Err(ResultsError::Message(
            "zero laps completed — Kunos may not write valid results".into(),
        ));
    }

    entries.sort_by_key(|e| e.position.unwrap_or(u32::MAX));

    Ok(ParsedSession {
        session_type: normalize_session_type(&session_type),
        track,
        entries,
    })
}

pub fn session_type_from_filename(name: &str) -> Option<String> {
    let stem = Path::new(name).file_stem()?.to_str()?;
    let last = stem.chars().last()?;
    match last.to_ascii_uppercase() {
        'P' | 'Q' | 'R' => Some(last.to_ascii_uppercase().to_string()),
        _ => None,
    }
}

fn normalize_session_type(raw: &str) -> String {
    match raw.to_ascii_uppercase().as_str() {
        "PRACTICE" | "P" => "P".to_string(),
        "QUALIFY" | "QUALIFYING" | "Q" => "Q".to_string(),
        "RACE" | "R" => "R".to_string(),
        other if other.len() == 1 => other.to_ascii_uppercase(),
        _ => "R".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "TrackName": "ks_nordschleife",
        "Type": "RACE",
        "Cars": [
            {
                "DriverName": "A. Rossi",
                "DriverGuid": "76561198000000001",
                "CarModel": "abarth500",
                "RacePosition": 1,
                "BestLap": 120456,
                "NumLaps": 5,
                "TotalTime": 602280
            },
            {
                "DriverName": "B. Müller",
                "DriverGuid": "76561198000000002",
                "CarModel": "abarth500",
                "RacePosition": 2,
                "BestLap": 121000,
                "NumLaps": 5,
                "TotalTime": 605500
            }
        ]
    }"#;

    #[test]
    fn parses_race_json() {
        let parsed = parse_results_json(SAMPLE, Some("260630_120000_R.json")).unwrap();
        assert_eq!(parsed.session_type, "R");
        assert_eq!(parsed.track, "ks_nordschleife");
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].driver_name, "A. Rossi");
        assert_eq!(parsed.entries[0].laps, 5);
    }

    #[test]
    fn filename_session_type() {
        assert_eq!(
            session_type_from_filename("260630_120000_Q.json").as_deref(),
            Some("Q")
        );
    }
}
