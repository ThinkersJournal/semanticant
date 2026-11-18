use crate::CoreError;

/// Where a piece of data came from and on what terms it may be used. Required for every data pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// What the data is derived from (a resource name, not a URL).
    pub source: String,
    /// SPDX licence expression of the DATA, e.g. `CC-BY-SA-4.0` or `MIT OR Apache-2.0`.
    pub licence: String,
    /// Credit line the licence requires or the author asks for.
    pub credit: String,
    /// Where it was obtained.
    pub url: Option<String>,
    /// Date retrieved, `YYYY-MM-DD`.
    pub retrieved: Option<String>,
}

impl Provenance {
    /// Check the record is complete enough to ship: source, licence and credit non-empty,
    /// licence a plausible SPDX expression, date (if any) shaped `YYYY-MM-DD`.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.source.trim().is_empty() {
            return Err(CoreError::EmptyField("source"));
        }
        if self.credit.trim().is_empty() {
            return Err(CoreError::EmptyField("credit"));
        }
        if self.licence.trim().is_empty() {
            return Err(CoreError::EmptyField("licence"));
        }
        check_spdx_shape(&self.licence)?;
        if let Some(d) = &self.retrieved {
            check_date(d)?;
        }
        Ok(())
    }
}

/// Shape check only (not a licence-list lookup): tokens of `[A-Za-z0-9.+-]`, parentheses, and the
/// operators AND / OR / WITH, with no empty operand.
fn check_spdx_shape(expr: &str) -> Result<(), CoreError> {
    let bad = || CoreError::InvalidLicence(expr.to_string());
    let spaced = expr.replace('(', " ( ").replace(')', " ) ");
    let mut depth = 0i32;
    let mut expect_operand = true;
    for tok in spaced.split_whitespace() {
        match tok {
            "(" if expect_operand => depth += 1,
            ")" if !expect_operand => {
                depth -= 1;
                if depth < 0 {
                    return Err(bad());
                }
            }
            "AND" | "OR" | "WITH" if !expect_operand => expect_operand = true,
            t if expect_operand
                && !matches!(t, "(" | ")" | "AND" | "OR" | "WITH")
                && t.bytes().all(|b| {
                    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+' | b':')
                }) =>
            {
                expect_operand = false
            }
            _ => return Err(bad()),
        }
    }
    if expect_operand || depth != 0 {
        Err(bad())
    } else {
        Ok(())
    }
}

fn check_date(d: &str) -> Result<(), CoreError> {
    let b = d.as_bytes();
    let shape = b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
    if !shape {
        return Err(CoreError::InvalidDate(d.to_string()));
    }
    let month: u32 = d[5..7].parse().unwrap_or(0);
    let day: u32 = d[8..10].parse().unwrap_or(0);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(CoreError::InvalidDate(d.to_string()));
    }
    Ok(())
}
