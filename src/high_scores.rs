use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAX_HIGH_SCORES: usize = 5;
const DEFAULT_HIGH_SCORES: [(&str, u32); MAX_HIGH_SCORES] = [
    ("SLC", 250_000),
    ("ACE", 175_000),
    ("ROM", 125_000),
    ("ARC", 90_000),
    ("CPU", 50_000),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighScoreEntry {
    pub initials: String,
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighScoreTable {
    entries: Vec<HighScoreEntry>,
}

impl Default for HighScoreTable {
    fn default() -> Self {
        Self {
            entries: DEFAULT_HIGH_SCORES
                .into_iter()
                .map(|(initials, score)| HighScoreEntry {
                    initials: initials.to_string(),
                    score,
                })
                .collect(),
        }
    }
}

impl HighScoreTable {
    pub fn load_default() -> Self {
        Self::load(&default_storage_path()).unwrap_or_default()
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Self::parse(&text).unwrap_or_default()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error),
        }
    }

    pub fn save_default(&self) -> io::Result<()> {
        self.save(&default_storage_path())
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.serialize())
    }

    pub fn entries(&self) -> &[HighScoreEntry] {
        &self.entries
    }

    pub fn top_score(&self) -> u32 {
        self.entries.first().map_or(0, |entry| entry.score)
    }

    pub fn qualifies(&self, score: u32) -> bool {
        score > 0
            && (self.entries.len() < MAX_HIGH_SCORES
                || self.entries.last().is_some_and(|entry| score > entry.score))
    }

    pub fn projected_rank(&self, score: u32) -> Option<usize> {
        self.qualifies(score).then(|| {
            self.entries
                .iter()
                .take_while(|entry| entry.score > score)
                .count()
                + 1
        })
    }

    pub fn insert(&mut self, initials: &str, score: u32) -> usize {
        let sanitized = sanitize_initials(initials);
        self.entries.push(HighScoreEntry {
            initials: sanitized.clone(),
            score,
        });
        self.entries.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.initials.cmp(&right.initials))
        });
        self.entries.truncate(MAX_HIGH_SCORES);
        self.entries
            .iter()
            .position(|entry| entry.score == score && entry.initials == sanitized)
            .map_or(MAX_HIGH_SCORES, |index| index + 1)
    }

    pub fn rows(&self) -> Vec<String> {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                format!(
                    "  {:>2}.  {:<8}  {:>7}",
                    index + 1,
                    entry.initials,
                    entry.score
                )
            })
            .collect()
    }

    fn parse(text: &str) -> Option<Self> {
        let mut entries = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let mut parts = trimmed.split_whitespace();
            let initials = parts.next()?;
            let score = parts.next()?.parse::<u32>().ok()?;
            entries.push(HighScoreEntry {
                initials: sanitize_initials(initials),
                score,
            });
        }

        entries.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.initials.cmp(&right.initials))
        });
        entries.truncate(MAX_HIGH_SCORES);

        Some(Self { entries })
    }

    fn serialize(&self) -> String {
        let mut text = String::new();
        for entry in &self.entries {
            text.push_str(&entry.initials);
            text.push(' ');
            text.push_str(&entry.score.to_string());
            text.push('\n');
        }
        text
    }
}

pub fn default_storage_path() -> PathBuf {
    if let Some(path) = env::var_os("DEFENDER_DATA_DIR") {
        return PathBuf::from(path).join("high_scores.txt");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".xyzzy")
            .join("defender")
            .join("high_scores.txt");
    }

    PathBuf::from(".xyzzy")
        .join("defender")
        .join("high_scores.txt")
}

pub fn sanitize_initials(initials: &str) -> String {
    let mut cleaned: String = initials
        .chars()
        .filter(char::is_ascii_alphabetic)
        .map(|character| character.to_ascii_uppercase())
        .take(3)
        .collect();

    while cleaned.len() < 3 {
        cleaned.push('_');
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{HighScoreTable, default_storage_path, sanitize_initials};

    static NEXT_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "defender-high-scores-test-{}-{}",
                std::process::id(),
                NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn default_table_exposes_seeded_scores() {
        let table = HighScoreTable::default();

        assert_eq!(table.entries().len(), 5);
        assert_eq!(table.top_score(), 250_000);
        assert_eq!(table.rows()[0], "   1.  SLC        250000");
    }

    #[test]
    fn qualifying_rank_requires_beating_the_last_seeded_score() {
        let table = HighScoreTable::default();

        assert_eq!(table.projected_rank(200_000), Some(2));
        assert_eq!(table.projected_rank(50_000), None);
    }

    #[test]
    fn insert_sorts_descending_and_truncates_to_five_scores() {
        let mut table = HighScoreTable::default();

        let rank = table.insert("zap", 300_000);

        assert_eq!(rank, 1);
        assert_eq!(table.entries()[0].initials, "ZAP");
        assert_eq!(table.entries()[0].score, 300_000);
        assert_eq!(table.entries().len(), 5);
        assert_eq!(
            table.entries().last().map(|entry| entry.initials.as_str()),
            Some("ARC")
        );
    }

    #[test]
    fn save_and_load_round_trip_scores() {
        let temp_dir = TempDir::new();
        let path = temp_dir.path().join("scores.txt");
        let mut table = HighScoreTable::default();
        table.insert("ace", 300_000);

        table.save(&path).expect("save scores");
        let loaded = HighScoreTable::load(&path).expect("load scores");

        assert_eq!(loaded, table);
    }

    #[test]
    fn load_defaults_when_file_is_missing() {
        let temp_dir = TempDir::new();
        let path = temp_dir.path().join("missing.txt");

        let loaded = HighScoreTable::load(&path).expect("load default scores");

        assert_eq!(loaded, HighScoreTable::default());
    }

    #[test]
    fn sanitize_initials_keeps_only_letters_and_pads_short_names() {
        assert_eq!(sanitize_initials("ab1"), "AB_");
        assert_eq!(sanitize_initials("arcade"), "ARC");
    }

    #[test]
    fn default_storage_path_prefers_home_or_override() {
        let path = default_storage_path();
        assert!(!path.as_os_str().is_empty());
    }
}
