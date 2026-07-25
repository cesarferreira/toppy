use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub cpu_cores_expanded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cpu_cores_expanded: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };

        parse(&contents).unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = config_path() else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(path)?;
        write!(
            file,
            "cpu_cores_expanded = {}\n",
            self.cpu_cores_expanded
        )?;
        Ok(())
    }
}

fn config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("toppy"));
        }
    }

    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").join("toppy"))
}

fn config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.toml"))
}

fn parse(contents: &str) -> Option<Config> {
    let mut cpu_cores_expanded = false;

    for line in contents.lines() {
        let line = line.split('#').next()?.trim();
        if line.is_empty() {
            continue;
        }

        let (key, value) = line.split_once('=')?;
        if key.trim() != "cpu_cores_expanded" {
            continue;
        }

        cpu_cores_expanded = match value.trim() {
            "true" => true,
            "false" => false,
            _ => return None,
        };
    }

    Some(Config { cpu_cores_expanded })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cpu_cores_expanded() {
        let cfg = parse("cpu_cores_expanded = true\n").unwrap();
        assert!(cfg.cpu_cores_expanded);

        let cfg = parse("# comment\ncpu_cores_expanded = false\n").unwrap();
        assert!(!cfg.cpu_cores_expanded);
    }

    #[test]
    fn parse_rejects_invalid_values() {
        assert!(parse("cpu_cores_expanded = maybe").is_none());
    }
}
