use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(crate) struct InputScript {
    rules: Vec<InputRule>,
}

impl InputScript {
    pub(crate) fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let mut stack = Vec::new();
        Self::from_path_inner(path.as_ref(), &mut stack)
    }

    fn from_path_inner(path: &Path, stack: &mut Vec<PathBuf>) -> Result<Self, Box<dyn Error>> {
        let path = fs::canonicalize(path)?;
        if stack.iter().any(|entry| entry == &path) {
            return Err(format!("recursive input script include: {}", path.display()).into());
        }
        stack.push(path.clone());
        let source = fs::read_to_string(&path)?;
        let script =
            Self::parse_with_base_dir(&source, path.parent().unwrap_or(Path::new(".")), stack)
                .map_err(|err| format!("{}: {err}", path.display()))?;
        stack.pop();
        Ok(script)
    }

    pub(crate) fn input_for_frame(&self, frame: u32) -> u16 {
        self.input_override_for_frame(frame).unwrap_or(0)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub(crate) fn input_override_for_frame(&self, frame: u32) -> Option<u16> {
        self.rules
            .iter()
            .filter(|rule| rule.start <= frame && frame <= rule.end)
            .map(|rule| rule.input)
            .last()
    }

    #[cfg(test)]
    fn parse(source: &str) -> Result<Self, Box<dyn Error>> {
        Self::parse_with_base_dir(source, Path::new("."), &mut Vec::new())
    }

    fn parse_with_base_dir(
        source: &str,
        base_dir: &Path,
        stack: &mut Vec<PathBuf>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut rules = Vec::new();
        for (line_no, raw_line) in source.lines().enumerate() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            let frame_spec = parts
                .next()
                .ok_or_else(|| format!("input script line {}: missing frame", line_no + 1))?;
            if frame_spec.eq_ignore_ascii_case("include") {
                let include_path = parts.next().ok_or_else(|| {
                    format!("input script line {}: include requires a path", line_no + 1)
                })?;
                if parts.next().is_some() {
                    return Err(format!(
                        "input script line {}: include takes one path",
                        line_no + 1
                    )
                    .into());
                }
                let include = Self::from_path_inner(&base_dir.join(include_path), stack)
                    .map_err(|err| format!("input script line {}: {err}", line_no + 1))?;
                rules.extend(include.rules);
                continue;
            }
            let buttons = parts.collect::<Vec<_>>().join("+");
            let (start, end) = parse_frame_spec(frame_spec)
                .map_err(|err| format!("input script line {}: {err}", line_no + 1))?;
            let input = parse_buttons(&buttons)
                .map_err(|err| format!("input script line {}: {err}", line_no + 1))?;
            rules.push(InputRule { start, end, input });
        }
        Ok(Self { rules })
    }
}

#[derive(Debug)]
struct InputRule {
    start: u32,
    end: u32,
    input: u16,
}

fn parse_frame_spec(spec: &str) -> Result<(u32, u32), String> {
    let parse_one = |s: &str| {
        s.parse::<u32>()
            .map_err(|_| format!("invalid frame number `{s}`"))
    };
    if let Some((start, end)) = spec.split_once("..") {
        let start = parse_one(start)?;
        let end = parse_one(end)?;
        if end < start {
            return Err(format!("invalid descending frame range `{spec}`"));
        }
        Ok((start, end))
    } else if let Some((start, end)) = spec.split_once('-') {
        let start = parse_one(start)?;
        let end = parse_one(end)?;
        if end < start {
            return Err(format!("invalid descending frame range `{spec}`"));
        }
        Ok((start, end))
    } else {
        let frame = parse_one(spec)?;
        Ok((frame, frame))
    }
}

fn parse_buttons(spec: &str) -> Result<u16, String> {
    if spec.is_empty() || spec.eq_ignore_ascii_case("none") {
        return Ok(0);
    }
    if let Some(hex) = spec.strip_prefix("0x").or_else(|| spec.strip_prefix("0X")) {
        return u16::from_str_radix(hex, 16)
            .map_err(|_| format!("invalid hex input word `{spec}`"));
    }

    let mut input = 0u16;
    for token in spec.split(['+', ',', '|']) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        input |= match token.to_ascii_uppercase().as_str() {
            "B" => 1 << 0,
            "Y" => 1 << 1,
            "SELECT" => 1 << 2,
            "START" => 1 << 3,
            "UP" => 1 << 4,
            "DOWN" => 1 << 5,
            "LEFT" => 1 << 6,
            "RIGHT" => 1 << 7,
            "A" => 1 << 8,
            "X" => 1 << 9,
            "L" => 1 << 10,
            "R" => 1 << 11,
            "NONE" => 0,
            other => return Err(format!("unknown button `{other}`")),
        };
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, process};

    #[test]
    fn parses_named_buttons_to_snes_serial_bits() {
        assert_eq!(parse_buttons("START").unwrap(), 0x0008);
        assert_eq!(parse_buttons("A+RIGHT").unwrap(), 0x0180);
        assert_eq!(parse_buttons("B,Y,SELECT").unwrap(), 0x0007);
        assert_eq!(parse_buttons("none").unwrap(), 0);
    }

    #[test]
    fn parses_input_script_ranges_with_last_rule_winning() {
        let script = InputScript::parse(
            "
            # wake title
            10..12 START
            12 NONE
            20 A+RIGHT
            ",
        )
        .unwrap();

        assert_eq!(script.input_for_frame(9), 0);
        assert_eq!(script.input_for_frame(10), 0x0008);
        assert_eq!(script.input_for_frame(11), 0x0008);
        assert_eq!(script.input_for_frame(12), 0);
        assert_eq!(script.input_for_frame(20), 0x0180);
    }

    #[test]
    fn input_script_distinguishes_missing_frame_from_explicit_none_override() {
        let script = InputScript::parse(
            "
            10 NONE
            20 A+RIGHT
            ",
        )
        .unwrap();

        assert_eq!(script.input_override_for_frame(9), None);
        assert_eq!(script.input_override_for_frame(10), Some(0));
        assert_eq!(script.input_override_for_frame(20), Some(0x0180));
    }

    #[test]
    fn parses_input_script_includes_relative_to_parent_file() {
        let dir = env::temp_dir().join(format!("z3rs-input-test-{}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        let base = dir.join("base.txt");
        let extended = dir.join("extended.txt");
        fs::write(&base, "10 START\n").unwrap();
        fs::write(&extended, "include base.txt\n20 A+RIGHT\n").unwrap();

        let script = InputScript::from_path(&extended).unwrap();

        assert_eq!(script.input_for_frame(10), 0x0008);
        assert_eq!(script.input_for_frame(20), 0x0180);
        fs::remove_file(base).unwrap();
        fs::remove_file(extended).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
