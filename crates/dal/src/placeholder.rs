//! Placeholder scanning, so parameter mistakes surface as clear errors.
//!
//! SQLite and MySQL both use positional `?`, which means a parameterised query
//! written for one runs unchanged on the other — that is the portable dialect
//! IronRoot targets. Postgres instead uses numbered `$1`, `$2`, so the same SQL
//! is *not* portable there.
//!
//! IronRoot does not rewrite your SQL. Rewriting `?` into `$1` means editing
//! statement text, and getting that wrong corrupts queries in ways that are
//! hard to notice — a `?` inside a string literal or comment is not a
//! placeholder. Instead the scanner below counts *real* placeholders and the
//! `*_with` methods reject mismatches before anything reaches the database,
//! turning a confusing driver error into an actionable one.
//!
//! The scanner skips over the constructs where a `?` or `$` is not a
//! placeholder: single- and double-quoted strings (with `''`/`""` doubling and
//! backslash escapes), MySQL backtick identifiers, `--` line comments, `/* */`
//! block comments (nested, as Postgres allows), and Postgres dollar-quoted
//! bodies such as `$$ ... $$` or `$tag$ ... $tag$`.

use crate::{Backend, DalError};

/// Placeholders found in a statement.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Scan {
    /// Count of positional `?` placeholders (SQLite / MySQL style).
    pub(crate) qmark: usize,
    /// Distinct `$n` indices, in ascending order (Postgres style).
    pub(crate) numbered: Vec<usize>,
}

/// Walk `sql`, counting placeholders that are not inside a literal or comment.
pub(crate) fn scan(sql: &str) -> Scan {
    let b = sql.as_bytes();
    let mut out = Scan::default();
    let mut i = 0usize;

    while i < b.len() {
        match b[i] {
            // --- line comment -------------------------------------------------
            b'-' if b.get(i + 1) == Some(&b'-') => {
                i += 2;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            // --- block comment, nested ---------------------------------------
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let mut depth = 1usize;
                i += 2;
                while i < b.len() && depth > 0 {
                    if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                        depth += 1;
                        i += 2;
                    } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            // --- quoted string / identifier ----------------------------------
            q @ (b'\'' | b'"' | b'`') => {
                i += 1;
                while i < b.len() {
                    if b[i] == b'\\' && q != b'`' {
                        // Backslash escapes: MySQL by default, Postgres in E'..'.
                        i += 2;
                    } else if b[i] == q {
                        if b.get(i + 1) == Some(&q) {
                            i += 2; // doubled quote is a literal quote
                        } else {
                            i += 1;
                            break;
                        }
                    } else {
                        i += 1;
                    }
                }
            }
            // --- `$` : dollar-quoted body, or a numbered placeholder ---------
            b'$' => {
                if let Some((tag_end, tag)) = dollar_tag(b, i) {
                    // Skip to the matching closing tag; unterminated means the
                    // rest of the statement is body.
                    match find(b, tag_end, tag) {
                        Some(close) => i = close + tag.len(),
                        None => i = b.len(),
                    }
                } else if b
                    .get(i + 1)
                    .is_some_and(|c| c.is_ascii_digit() && *c != b'0')
                {
                    let mut j = i + 1;
                    let mut n = 0usize;
                    while j < b.len() && b[j].is_ascii_digit() {
                        n = n.saturating_mul(10).saturating_add((b[j] - b'0') as usize);
                        j += 1;
                    }
                    if !out.numbered.contains(&n) {
                        out.numbered.push(n);
                    }
                    i = j;
                } else {
                    i += 1;
                }
            }
            // --- `?` : placeholder, unless it is a Postgres jsonb operator ---
            b'?' => {
                // `?|`, `?&` and `??` are jsonb operators, not placeholders.
                match b.get(i + 1) {
                    Some(b'|') | Some(b'&') => i += 2,
                    Some(b'?') => i += 2,
                    _ => {
                        out.qmark += 1;
                        i += 1;
                    }
                }
            }
            _ => i += 1,
        }
    }

    out.numbered.sort_unstable();
    out
}

/// If a `$` at `start` opens a dollar-quoted string, return the offset just
/// past the opening tag and the tag itself (e.g. `$$` or `$fn$`).
fn dollar_tag(b: &[u8], start: usize) -> Option<(usize, &[u8])> {
    let mut j = start + 1;
    while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
        // A tag may not begin with a digit; `$1` is a placeholder.
        if j == start + 1 && b[j].is_ascii_digit() {
            return None;
        }
        j += 1;
    }
    if b.get(j) == Some(&b'$') {
        Some((j + 1, &b[start..=j]))
    } else {
        None
    }
}

fn find(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || from >= haystack.len() {
        return None;
    }
    (from..=haystack.len().saturating_sub(needle.len()))
        .find(|&i| &haystack[i..i + needle.len()] == needle)
}

/// Reject parameter/placeholder mismatches before they reach the driver.
pub(crate) fn validate(sql: &str, backend: Backend, n_params: usize) -> Result<(), DalError> {
    let found = scan(sql);

    match backend {
        // Both use positional `?`. This is the portable dialect: a query
        // written here runs unchanged on the other backend.
        Backend::Sqlite | Backend::MySql => {
            if found.qmark != n_params {
                return Err(DalError::Placeholder(format!(
                    "statement has {} `?` placeholder(s) but {} parameter(s) were supplied",
                    found.qmark, n_params
                )));
            }
            if !found.numbered.is_empty() && found.qmark == 0 && n_params > 0 {
                return Err(DalError::Placeholder(format!(
                    "statement uses Postgres-style numbered placeholders ($1, $2) but the \
                     backend is {backend}, which expects positional `?`"
                )));
            }
            Ok(())
        }
        Backend::Postgres => {
            if found.qmark > 0 {
                return Err(DalError::Placeholder(format!(
                    "statement uses {} positional `?` placeholder(s), but Postgres expects \
                     numbered placeholders ($1, $2, ...). Note that `?` is also a jsonb \
                     operator — write it as `?` only inside a quoted string, or use the \
                     jsonb_exists() function instead",
                    found.qmark
                )));
            }
            let highest = found.numbered.last().copied().unwrap_or(0);
            if highest != n_params {
                return Err(DalError::Placeholder(format!(
                    "statement references up to ${highest} but {n_params} parameter(s) were \
                     supplied"
                )));
            }
            // $1..$n must all be present: $1,$3 with two params binds nothing to $3.
            let expected: Vec<usize> = (1..=n_params).collect();
            if found.numbered != expected {
                return Err(DalError::Placeholder(format!(
                    "statement placeholders {:?} are not the contiguous run $1..${} that {} \
                     parameter(s) require",
                    found.numbered, n_params, n_params
                )));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(sql: &str) -> usize {
        scan(sql).qmark
    }

    #[test]
    fn counts_plain_positional_placeholders() {
        assert_eq!(q("SELECT * FROM t WHERE a = ? AND b = ?"), 2);
        assert_eq!(q("SELECT 1"), 0);
    }

    #[test]
    fn ignores_question_marks_inside_string_literals() {
        assert_eq!(q("SELECT '?' FROM t"), 0);
        assert_eq!(q("SELECT * FROM t WHERE a = ? AND b = 'why?'"), 1);
        // doubled quote is an escaped quote, not the end of the literal
        assert_eq!(q("SELECT 'it''s ? here' , ?"), 1);
        // backslash escape (MySQL default, and Postgres E'..')
        assert_eq!(q(r"SELECT 'a\'? b', ?"), 1);
    }

    #[test]
    fn ignores_question_marks_in_identifiers_and_comments() {
        assert_eq!(q(r#"SELECT "we?ird" FROM t WHERE a = ?"#), 1);
        assert_eq!(q("SELECT `back?tick` FROM t WHERE a = ?"), 1);
        assert_eq!(q("SELECT 1 -- is this ? a placeholder\n, ?"), 1);
        assert_eq!(q("SELECT /* ? nope */ ?"), 1);
        assert_eq!(q("SELECT /* outer /* ? inner */ still ? */ ?"), 1);
    }

    #[test]
    fn ignores_jsonb_operators() {
        assert_eq!(q("SELECT * FROM t WHERE data ?| array['a']"), 0);
        assert_eq!(q("SELECT * FROM t WHERE data ?& array['a']"), 0);
    }

    #[test]
    fn collects_numbered_placeholders() {
        assert_eq!(scan("SELECT $1, $2, $1").numbered, vec![1, 2]);
        assert_eq!(scan("SELECT $10").numbered, vec![10]);
    }

    #[test]
    fn dollar_quoted_bodies_are_not_placeholders() {
        assert_eq!(scan("SELECT $$ hi $1 ? $$").numbered, Vec::<usize>::new());
        assert_eq!(q("SELECT $$ ? $$"), 0);
        assert_eq!(scan("SELECT $tag$ $1 $tag$, $1").numbered, vec![1]);
    }

    #[test]
    fn sqlite_and_mysql_accept_the_same_portable_sql() {
        let sql = "SELECT * FROM t WHERE a = ? AND b = ?";
        assert!(validate(sql, Backend::Sqlite, 2).is_ok());
        assert!(validate(sql, Backend::MySql, 2).is_ok());
    }

    #[test]
    fn count_mismatch_is_rejected() {
        let err = validate("SELECT ? , ?", Backend::Sqlite, 1).unwrap_err();
        assert!(matches!(err, DalError::Placeholder(_)), "{err}");
        assert!(validate("SELECT ?", Backend::MySql, 0).is_err());
        assert!(validate("SELECT 1", Backend::Sqlite, 0).is_ok());
    }

    #[test]
    fn postgres_rejects_question_marks_with_a_hint() {
        let err = validate("SELECT * FROM t WHERE a = ?", Backend::Postgres, 1).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("$1"), "{msg}");
    }

    #[test]
    fn postgres_requires_a_contiguous_run() {
        assert!(validate("SELECT $1, $2", Backend::Postgres, 2).is_ok());
        assert!(validate("SELECT $1, $3", Backend::Postgres, 2).is_err());
        assert!(validate("SELECT $1", Backend::Postgres, 2).is_err());
        assert!(validate("SELECT 1", Backend::Postgres, 0).is_ok());
    }
}
