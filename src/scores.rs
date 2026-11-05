use chrono::Local;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScoreEntry {
    pub score: u32,
    pub date: String,
    pub highest_tile: u16,
    #[serde(default)]
    pub board: [[u16; 4]; 4],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HighScores {
    pub scores: Vec<ScoreEntry>,
}

impl HighScores {
    pub fn new() -> Self {
        Self { scores: Vec::new() }
    }
}

impl Default for HighScores {
    fn default() -> Self {
        Self::new()
    }
}

fn get_high_scores_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|path| path.join("2048").join("high-scores.json"))
}

fn get_legacy_high_scores_path() -> Option<PathBuf> {
    dirs::home_dir().map(|path| path.join(".2048_high_scores.json"))
}

pub fn load_high_scores() -> HighScores {
    if let Some(path) = get_high_scores_path() {
        if path.exists() {
            return read_high_scores(&path);
        }
    }

    if let Some(path) = get_legacy_high_scores_path() {
        if path.exists() {
            let scores = read_high_scores(&path);
            let _ = save_high_scores(&scores);
            return scores;
        }
    }

    HighScores::new()
}

fn read_high_scores(path: &PathBuf) -> HighScores {
    let data = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_high_scores(high_scores: &HighScores) -> io::Result<()> {
    if let Some(path) = get_high_scores_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(high_scores)?;
        fs::write(path, data)?;
    }
    Ok(())
}

pub fn add_high_score(high_scores: &mut HighScores, score: u32, board: &[[u16; 4]; 4]) {
    let highest_tile = *board.iter().flat_map(|row| row.iter()).max().unwrap_or(&0);
    let new_score = ScoreEntry {
        score,
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        highest_tile,
        board: *board,
    };

    high_scores.scores.push(new_score);
    high_scores.scores.sort_by_key(|entry| Reverse(entry.score));
    high_scores.scores.truncate(10);
    save_high_scores(high_scores).unwrap_or_default();
}

pub fn is_new_high_score(high_scores: &HighScores, score: u32) -> bool {
    if high_scores.scores.len() < 10 {
        return true;
    }
    score > high_scores.scores.last().map_or(0, |s| s.score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_entries_without_board_still_deserialize() {
        let entry: ScoreEntry =
            serde_json::from_str(r#"{"score":128,"date":"2026-11-05 12:00:00","highest_tile":64}"#)
                .expect("legacy score entry");
        assert_eq!(entry.board, [[0; 4]; 4]);
    }

    #[test]
    fn score_entry_keeps_the_final_board_snapshot() {
        let board = [[2, 4, 8, 16], [32, 64, 128, 256], [0; 4], [0; 4]];
        let entry = ScoreEntry {
            score: 2048,
            date: "2026-11-05 12:00:00".to_string(),
            highest_tile: 256,
            board,
        };
        let encoded = serde_json::to_string(&entry).expect("serialize score entry");
        let decoded: ScoreEntry = serde_json::from_str(&encoded).expect("deserialize score entry");
        assert_eq!(decoded.board, board);
    }
}
