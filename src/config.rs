use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub cpu_cores_expanded: bool,
    pub show_full_command: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cpu_cores_expanded: false,
            show_full_command: true,
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
            "cpu_cores_expanded = {}\nshow_full_command = {}\n",
            self.cpu_cores_expanded, self.show_full_command
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
    let mut config = Config::default();

    for line in contents.lines() {
        let line = line.split('#').next()?.trim();
        if line.is_empty() {
            continue;
        }

        let (key, value) = line.split_once('=')?;
        let field = match key.trim() {
            "cpu_cores_expanded" => &mut config.cpu_cores_expanded,
            "show_full_command" => &mut config.show_full_command,
            _ => continue,
        };

        *field = match value.trim() {
            "true" => true,
            "false" => false,
            _ => return None,
        };
    }

    Some(config)
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
    fn parse_show_full_command() {
        let cfg = parse("cpu_cores_expanded = true\n").unwrap();
        assert!(cfg.show_full_command, "defaults to true when missing");

        let cfg = parse("cpu_cores_expanded = false\nshow_full_command = false\n").unwrap();
        assert!(!cfg.show_full_command);
    }

    #[test]
    fn parse_rejects_invalid_values() {
        assert!(parse("cpu_cores_expanded = maybe").is_none());
    }
}
