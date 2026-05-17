use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};

const MAX_ARGS_LEN: usize = 80;
const MAX_ERR_LEN: usize = 200;
const KEEP_WEEKS: i64 = 5;

fn audit_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("sodium"))
}

fn current_week_filename() -> String {
    let iso = Local::now().iso_week();
    format!("audit-{:04}-W{:02}.log", iso.year(), iso.week())
}

/// Append one TSV-formatted line to ~/.config/sodium/audit-YYYY-Www.log.
/// Columns: timestamp \t source \t repo \t action \t args \t result
///
/// One file per ISO week, with retention of the last KEEP_WEEKS weeks.
/// Older files are purged on each call (best-effort, cost is one read_dir).
///
/// Best-effort: write errors are silently swallowed so audit never breaks
/// a command. Use `SODIUM_AUDIT_DEBUG=1` to surface them on stderr.
pub fn log(source: &str, repo: &Path, action: &str, args: &str, result: Result<&str, &str>) {
    let dir = match audit_dir() {
        Some(d) => d,
        None => return,
    };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }

    purge_old_logs(&dir);

    let path = dir.join(current_week_filename());
    let ts = Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string();
    let result_field = match result {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("err: {}", truncate(e, MAX_ERR_LEN)),
    };
    let line = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\n",
        sanitize(&ts),
        sanitize(source),
        sanitize(&repo.display().to_string()),
        sanitize(action),
        sanitize(&truncate(args, MAX_ARGS_LEN)),
        sanitize(&result_field),
    );

    let write_result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(line.as_bytes()));

    if let Err(e) = write_result {
        if std::env::var("SODIUM_AUDIT_DEBUG").is_ok() {
            eprintln!("sodium audit: {}: {}", path.display(), e);
        }
    }
}

fn purge_old_logs(dir: &Path) {
    let now_iso = Local::now().iso_week();
    let current_monday = match NaiveDate::from_isoywd_opt(now_iso.year(), now_iso.week(), Weekday::Mon) {
        Some(d) => d,
        None => return,
    };
    let cutoff = current_monday - Duration::weeks(KEEP_WEEKS);

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if let Some((year, week)) = parse_audit_filename(&name_s) {
            if let Some(monday) = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon) {
                if monday < cutoff {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
}

fn parse_audit_filename(name: &str) -> Option<(i32, u32)> {
    let s = name.strip_prefix("audit-")?.strip_suffix(".log")?;
    let (y, w) = s.split_once("-W")?;
    let year = y.parse().ok()?;
    let week = w.parse().ok()?;
    Some((year, week))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    format!("{}…", cut)
}

fn sanitize(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}
