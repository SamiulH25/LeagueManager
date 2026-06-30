/// F1-style points for race finish positions 1–10.
pub const DEFAULT_RACE_POINTS: [u32; 10] = [25, 18, 15, 12, 10, 8, 6, 4, 2, 1];

pub fn points_for_position(position: u32) -> u32 {
    if position == 0 {
        return 0;
    }
    let idx = position.saturating_sub(1) as usize;
    DEFAULT_RACE_POINTS.get(idx).copied().unwrap_or(0)
}

pub fn session_label(session_type: &str) -> &'static str {
    match session_type.to_ascii_uppercase().as_str() {
        "P" | "PRACTICE" => "Practice",
        "Q" | "QUALIFY" => "Qualifying",
        "R" | "RACE" => "Race",
        _ => "Session",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race_points_table() {
        assert_eq!(points_for_position(1), 25);
        assert_eq!(points_for_position(10), 1);
        assert_eq!(points_for_position(11), 0);
    }
}
