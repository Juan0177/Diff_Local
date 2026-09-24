use serde::Serialize;
use similar::{DiffOp, TextDiff};
use std::fs;
use std::path::Path;

/// Soft warning threshold (50 MiB).
pub const WARN_BYTES: u64 = 50 * 1024 * 1024;
/// Hard refusal threshold (100 MiB).
pub const MAX_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RowKind {
    Equal,
    Insert,
    Delete,
    Replace,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InlineSpan {
    pub text: String,
    /// True when this fragment differs between left and right.
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiffRow {
    pub kind: RowKind,
    pub left_no: Option<u32>,
    pub right_no: Option<u32>,
    pub left_text: Option<String>,
    pub right_text: Option<String>,
    /// Word/char-level spans for in-line highlight (replace rows with both sides).
    pub left_spans: Option<Vec<InlineSpan>>,
    pub right_spans: Option<Vec<InlineSpan>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiffStats {
    pub equal: usize,
    pub insert: usize,
    pub delete: usize,
    pub replace: usize,
    pub total_rows: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffResult {
    pub left_path: String,
    pub right_path: String,
    pub left_bytes: u64,
    pub right_bytes: u64,
    pub warning: Option<String>,
    pub stats: DiffStats,
    pub rows: Vec<DiffRow>,
    /// Indices into `rows` that are not `Equal` (for prev/next navigation).
    pub change_indices: Vec<usize>,
}

#[derive(Debug)]
pub enum DiffError {
    Io(String),
    TooLarge { path: String, size: u64 },
}

impl DiffError {
    pub fn message(&self) -> String {
        match self {
            DiffError::Io(msg) => msg.clone(),
            DiffError::TooLarge { path, size } => format!(
                "Il file «{}» è troppo grande ({:.1} MiB). Limite massimo: {} MiB.",
                path,
                *size as f64 / (1024.0 * 1024.0),
                MAX_BYTES / (1024 * 1024)
            ),
        }
    }
}

fn split_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&str> = text.split('\n').collect();
    // Preserve empty trailing line semantics: a trailing newline yields an
    // empty final element from split; TextDiff::from_lines strips that.
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn strip_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

/// Word-level when whitespace is present, otherwise character-level.
fn compute_inline_spans(old: &str, new: &str) -> (Vec<InlineSpan>, Vec<InlineSpan>) {
    use similar::ChangeTag;

    let use_words = old.chars().any(char::is_whitespace)
        || new.chars().any(char::is_whitespace);

    let mut left: Vec<InlineSpan> = Vec::new();
    let mut right: Vec<InlineSpan> = Vec::new();

    // TextDiff::from_words / from_chars borrow the inputs for the lifetime of the diff.
    if use_words {
        let diff = TextDiff::from_words(old, new);
        for change in diff.iter_all_changes() {
            let text = change.value().to_string();
            match change.tag() {
                ChangeTag::Equal => {
                    left.push(InlineSpan {
                        text: text.clone(),
                        changed: false,
                    });
                    right.push(InlineSpan {
                        text,
                        changed: false,
                    });
                }
                ChangeTag::Delete => {
                    left.push(InlineSpan {
                        text,
                        changed: true,
                    });
                }
                ChangeTag::Insert => {
                    right.push(InlineSpan {
                        text,
                        changed: true,
                    });
                }
            }
        }
    } else {
        let diff = TextDiff::from_chars(old, new);
        for change in diff.iter_all_changes() {
            let text = change.value().to_string();
            match change.tag() {
                ChangeTag::Equal => {
                    left.push(InlineSpan {
                        text: text.clone(),
                        changed: false,
                    });
                    right.push(InlineSpan {
                        text,
                        changed: false,
                    });
                }
                ChangeTag::Delete => {
                    left.push(InlineSpan {
                        text,
                        changed: true,
                    });
                }
                ChangeTag::Insert => {
                    right.push(InlineSpan {
                        text,
                        changed: true,
                    });
                }
            }
        }
    }

    (left, right)
}

fn row(
    kind: RowKind,
    left_no: Option<u32>,
    right_no: Option<u32>,
    left_text: Option<String>,
    right_text: Option<String>,
) -> DiffRow {
    let (left_spans, right_spans) = match (&kind, &left_text, &right_text) {
        (RowKind::Replace, Some(l), Some(r)) => {
            let (ls, rs) = compute_inline_spans(l, r);
            (Some(ls), Some(rs))
        }
        _ => (None, None),
    };

    DiffRow {
        kind,
        left_no,
        right_no,
        left_text,
        right_text,
        left_spans,
        right_spans,
    }
}

/// Build aligned side-by-side rows from two text buffers.
pub fn compute_text_diff(left: &str, right: &str) -> (Vec<DiffRow>, DiffStats, Vec<usize>) {
    let left_lines = split_lines(left);
    let right_lines = split_lines(right);

    let left_owned: Vec<String> = left_lines.iter().map(|l| strip_cr(l).to_string()).collect();
    let right_owned: Vec<String> = right_lines.iter().map(|l| strip_cr(l).to_string()).collect();
    let left_refs: Vec<&str> = left_owned.iter().map(String::as_str).collect();
    let right_refs: Vec<&str> = right_owned.iter().map(String::as_str).collect();

    let diff = TextDiff::from_slices(&left_refs, &right_refs);

    let mut rows: Vec<DiffRow> = Vec::new();
    let mut stats = DiffStats {
        equal: 0,
        insert: 0,
        delete: 0,
        replace: 0,
        total_rows: 0,
    };
    let mut change_indices: Vec<usize> = Vec::new();

    for op in diff.ops() {
        match *op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                for i in 0..len {
                    rows.push(row(
                        RowKind::Equal,
                        Some((old_index + i + 1) as u32),
                        Some((new_index + i + 1) as u32),
                        Some(left_owned[old_index + i].clone()),
                        Some(right_owned[new_index + i].clone()),
                    ));
                    stats.equal += 1;
                }
            }
            DiffOp::Delete {
                old_index,
                old_len,
                ..
            } => {
                for i in 0..old_len {
                    change_indices.push(rows.len());
                    rows.push(row(
                        RowKind::Delete,
                        Some((old_index + i + 1) as u32),
                        None,
                        Some(left_owned[old_index + i].clone()),
                        None,
                    ));
                    stats.delete += 1;
                }
            }
            DiffOp::Insert {
                new_index,
                new_len,
                ..
            } => {
                for i in 0..new_len {
                    change_indices.push(rows.len());
                    rows.push(row(
                        RowKind::Insert,
                        None,
                        Some((new_index + i + 1) as u32),
                        None,
                        Some(right_owned[new_index + i].clone()),
                    ));
                    stats.insert += 1;
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let max = old_len.max(new_len);
                for i in 0..max {
                    let left = if i < old_len {
                        Some(left_owned[old_index + i].clone())
                    } else {
                        None
                    };
                    let right = if i < new_len {
                        Some(right_owned[new_index + i].clone())
                    } else {
                        None
                    };
                    let left_no = if i < old_len {
                        Some((old_index + i + 1) as u32)
                    } else {
                        None
                    };
                    let right_no = if i < new_len {
                        Some((new_index + i + 1) as u32)
                    } else {
                        None
                    };

                    let kind = match (&left, &right) {
                        (Some(_), Some(_)) => RowKind::Replace,
                        (Some(_), None) => RowKind::Delete,
                        (None, Some(_)) => RowKind::Insert,
                        (None, None) => unreachable!(),
                    };

                    change_indices.push(rows.len());
                    rows.push(row(kind, left_no, right_no, left, right));

                    match kind {
                        RowKind::Replace => stats.replace += 1,
                        RowKind::Delete => stats.delete += 1,
                        RowKind::Insert => stats.insert += 1,
                        RowKind::Equal => {}
                    }
                }
            }
        }
    }

    stats.total_rows = rows.len();
    (rows, stats, change_indices)
}

fn read_text_file(path: &Path) -> Result<(String, u64), DiffError> {
    let meta = fs::metadata(path).map_err(|e| {
        DiffError::Io(format!(
            "Impossibile leggere i metadati di «{}»: {}",
            path.display(),
            e
        ))
    })?;
    let size = meta.len();
    if size > MAX_BYTES {
        return Err(DiffError::TooLarge {
            path: path.display().to_string(),
            size,
        });
    }

    let bytes = fs::read(path).map_err(|e| {
        DiffError::Io(format!(
            "Impossibile aprire «{}»: {}",
            path.display(),
            e
        ))
    })?;

    // UTF-8 with lossy fallback for latin-1-ish / mixed files.
    let text = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(err) => String::from_utf8_lossy(err.as_bytes()).into_owned(),
    };

    Ok((text, size))
}

pub fn compute_diff_files(left_path: &str, right_path: &str) -> Result<DiffResult, DiffError> {
    let left_p = Path::new(left_path);
    let right_p = Path::new(right_path);

    let (left_text, left_bytes) = read_text_file(left_p)?;
    let (right_text, right_bytes) = read_text_file(right_p)?;

    let mut warning = None;
    if left_bytes > WARN_BYTES || right_bytes > WARN_BYTES {
        warning = Some(format!(
            "Uno o entrambi i file superano {} MiB: il calcolo del diff può richiedere più memoria e tempo.",
            WARN_BYTES / (1024 * 1024)
        ));
    }

    let (rows, stats, change_indices) = compute_text_diff(&left_text, &right_text);

    Ok(DiffResult {
        left_path: left_path.to_string(),
        right_path: right_path.to_string(),
        left_bytes,
        right_bytes,
        warning,
        stats,
        rows,
        change_indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_files() {
        let (rows, stats, changes) = compute_text_diff("a\nb\nc\n", "a\nb\nc\n");
        assert_eq!(stats.equal, 3);
        assert_eq!(stats.insert, 0);
        assert_eq!(stats.delete, 0);
        assert_eq!(stats.replace, 0);
        assert!(changes.is_empty());
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].kind, RowKind::Equal);
    }

    #[test]
    fn insert_lines() {
        let (rows, stats, changes) = compute_text_diff("a\nc\n", "a\nb\nc\n");
        assert_eq!(stats.insert, 1);
        assert_eq!(stats.equal, 2);
        assert_eq!(changes.len(), 1);
        assert!(rows.iter().any(|r| r.kind == RowKind::Insert));
        let insert = rows.iter().find(|r| r.kind == RowKind::Insert).unwrap();
        assert_eq!(insert.right_text.as_deref(), Some("b"));
        assert!(insert.left_text.is_none());
    }

    #[test]
    fn delete_lines() {
        let (rows, stats, _) = compute_text_diff("a\nb\nc\n", "a\nc\n");
        assert_eq!(stats.delete, 1);
        assert_eq!(stats.equal, 2);
        let del = rows.iter().find(|r| r.kind == RowKind::Delete).unwrap();
        assert_eq!(del.left_text.as_deref(), Some("b"));
        assert!(del.right_text.is_none());
    }

    #[test]
    fn replace_lines() {
        let (rows, stats, changes) = compute_text_diff("hello\nworld\n", "hello\nplanet\n");
        assert_eq!(stats.equal, 1);
        assert!(stats.replace >= 1 || stats.delete + stats.insert >= 1);
        assert!(!changes.is_empty());
        assert!(rows.iter().any(|r| {
            matches!(r.kind, RowKind::Replace | RowKind::Delete | RowKind::Insert)
        }));
    }

    #[test]
    fn inline_highlights_within_line() {
        let (rows, stats, _) =
            compute_text_diff("alpha beta gamma\n", "alpha BETA gamma\n");
        assert_eq!(stats.replace, 1);
        let row = rows.iter().find(|r| r.kind == RowKind::Replace).unwrap();
        let left = row.left_spans.as_ref().expect("left spans");
        let right = row.right_spans.as_ref().expect("right spans");
        assert!(left.iter().any(|s| s.changed && s.text.contains("beta")));
        assert!(right.iter().any(|s| s.changed && s.text.contains("BETA")));
        assert!(left.iter().any(|s| !s.changed && s.text.contains("alpha")));
        assert!(right.iter().any(|s| !s.changed && s.text.contains("gamma")));
    }

    #[test]
    fn inline_char_level_without_spaces() {
        let (rows, _, _) = compute_text_diff("hello\n", "hallo\n");
        let row = rows.iter().find(|r| r.kind == RowKind::Replace).unwrap();
        let left = row.left_spans.as_ref().unwrap();
        let right = row.right_spans.as_ref().unwrap();
        assert!(left.iter().any(|s| s.changed));
        assert!(right.iter().any(|s| s.changed));
        let left_joined: String = left.iter().map(|s| s.text.as_str()).collect();
        let right_joined: String = right.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(left_joined, "hello");
        assert_eq!(right_joined, "hallo");
    }

    #[test]
    fn empty_both() {
        let (rows, stats, changes) = compute_text_diff("", "");
        assert!(rows.is_empty());
        assert_eq!(stats.total_rows, 0);
        assert!(changes.is_empty());
    }

    #[test]
    fn crlf_normalized() {
        let (rows, stats, _) = compute_text_diff("a\r\nb\r\n", "a\nb\n");
        assert_eq!(stats.equal, 2);
        assert_eq!(rows[0].left_text.as_deref(), Some("a"));
        assert_eq!(rows[0].right_text.as_deref(), Some("a"));
    }

    #[test]
    fn large_file_smoke() {
        // ~20k lines with sparse edits — ensures the path stays responsive.
        let mut left = String::with_capacity(400_000);
        let mut right = String::with_capacity(400_000);
        for i in 0..20_000 {
            left.push_str(&format!("line-{i}-common-content\n"));
            if i % 500 == 0 {
                right.push_str(&format!("line-{i}-CHANGED-content\n"));
            } else {
                right.push_str(&format!("line-{i}-common-content\n"));
            }
        }
        let (rows, stats, changes) = compute_text_diff(&left, &right);
        assert_eq!(rows.len(), 20_000);
        assert_eq!(stats.replace, 40); // 0,500,...,19500 => 40 replacements
        assert_eq!(changes.len(), 40);
    }

    #[test]
    fn compute_diff_files_roundtrip() {
        let dir = std::env::temp_dir().join("diff_local_smoke");
        let _ = fs::create_dir_all(&dir);
        let left = dir.join("left.txt");
        let right = dir.join("right.txt");
        fs::write(&left, "one\ntwo\nthree\n").unwrap();
        fs::write(&right, "one\nTWO\nthree\nfour\n").unwrap();

        let result = compute_diff_files(
            left.to_str().unwrap(),
            right.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(result.stats.equal, 2);
        assert!(result.stats.replace >= 1 || result.stats.delete + result.stats.insert >= 1);
        assert!(result.stats.insert >= 1);
        assert!(!result.change_indices.is_empty());
        assert!(result.warning.is_none());
    }
}
