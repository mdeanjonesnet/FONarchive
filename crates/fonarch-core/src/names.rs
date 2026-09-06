/// Cross-platform file/folder name from an Adobe or OpenType string.
pub fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_us = false;
    for ch in name.chars() {
        let repl = match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_whitespace() => '_',
            c if c.is_control() => '_',
            c => c,
        };
        if repl == '_' {
            if prev_us {
                continue;
            }
            prev_us = true;
            out.push('_');
        } else {
            prev_us = false;
            out.push(repl);
        }
    }
    let out = out.trim_matches(|c: char| c == '_' || c == '.' || c == ' ');
    if out.is_empty() {
        return "unnamed".to_string();
    }
    let mut out = out.to_string();
    if is_windows_reserved(&out) {
        out.push('_');
    }
    if out.len() > 120 {
        out.truncate(120);
        while out.ends_with('_') {
            out.pop();
        }
        if out.is_empty() {
            return "unnamed".to_string();
        }
    }
    out
}

fn is_windows_reserved(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name);
    matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

/// Family folder + `{family}_{variation}{ext}` file name. Adobe `familyName`
/// splits (CondorWide vs CondorCond) are kept as-is.
pub fn dest_parts(family: &str, variation: &str, ext: &str) -> (String, String) {
    let fam = sanitize(family);
    let var = sanitize(if variation.trim().is_empty() {
        "Regular"
    } else {
        variation
    });
    let file = format!("{fam}_{var}{ext}");
    (fam, file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaces_and_illegal() {
        assert_eq!(sanitize("Proxima Nova"), "Proxima_Nova");
        assert_eq!(sanitize("A<b>:c"), "A_b_c");
        assert_eq!(sanitize("   "), "unnamed");
        assert_eq!(sanitize("foo___bar"), "foo_bar");
    }

    #[test]
    fn reserved_windows() {
        assert_eq!(sanitize("CON"), "CON_");
        assert_eq!(sanitize("nul"), "nul_");
    }

    #[test]
    fn dest_keeps_family_split() {
        let (dir, file) = dest_parts("CondorWide", "Bold", ".otf");
        assert_eq!(dir, "CondorWide");
        assert_eq!(file, "CondorWide_Bold.otf");
    }
}
