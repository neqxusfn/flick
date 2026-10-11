use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Game,
    Distance,
    Raw,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub game: String,
    pub mode: Mode,
    pub sensitivity: f64,
    pub custom_yaw: f64,
    pub cm360: f64,
    pub dpi: f64,
    pub raw_counts: f64,
    pub trigger: u32,
    pub trigger_label: String,
    pub duration: u32,
    pub direction: Direction,
    pub flick_back: bool,
    pub process: String,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            id: "default".into(),
            name: "My profile".into(),
            game: "valorant".into(),
            mode: Mode::Game,
            sensitivity: 0.5,
            custom_yaw: 0.022,
            cm360: 32.66,
            dpi: 800.0,
            raw_counts: 5143.0,
            trigger: 5,
            trigger_label: "Mouse 4".into(),
            duration: 12,
            direction: Direction::Right,
            flick_back: false,
            process: String::new(),
        }
    }
}
impl Profile {
    pub fn yaw(&self) -> f64 {
        match self.game.as_str() {
            // Primary source: KovaaK's developer release notes, 2.0.1.0 (May 2020).
            // Explicitly documents Valorant 0.07 and Fortnite Percent 0.005555.
            // https://store.steampowered.com/news/posts/?appids=824270&enddate=1595643090&feed=steam_community_announcements
            "valorant" => 0.07,
            // Fortnite input uses the displayed percentage (e.g. 10, not 0.10).
            // Same developer source above; validate locally after game updates.
            "fortnite" => 0.005555,
            _ => self.custom_yaw,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    #[serde(default)]
    pub risk_acknowledged: bool,
    pub profiles: Vec<Profile>,
    pub active_id: String,
    pub auto_switch: bool,
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub theme: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            risk_acknowledged: false,
            profiles: vec![Profile::default()],
            active_id: "default".into(),
            auto_switch: false,
            start_with_windows: false,
            start_minimized: false,
            theme: "system".into(),
        }
    }
}
impl Settings {
    pub fn active(&self) -> &Profile {
        self.profiles
            .iter()
            .find(|p| p.id == self.active_id)
            .unwrap_or(&self.profiles[0])
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.profiles.is_empty() || self.profiles.len() > 50 {
            return Err("This profile file is not supported.".into());
        }
        if !["system", "light", "dark"].contains(&self.theme.as_str()) {
            return Err("Choose a supported appearance.".into());
        }
        let mut ids = HashSet::new();
        let mut processes = HashSet::new();
        for p in &self.profiles {
            if p.id.is_empty()
                || !ids.insert(&p.id)
                || p.name.trim().is_empty()
                || p.name.len() > 80
            {
                return Err("Give each profile a unique ID and a short name.".into());
            }
            if !["custom", "valorant", "fortnite"].contains(&p.game.as_str()) || p.duration > 100 {
                return Err("Check the game and turn duration.".into());
            }
            if !(3..=254).contains(&p.trigger) || [0x77, 0x78].contains(&p.trigger) {
                return Err("F8 and F9 are reserved. Choose another trigger.".into());
            }
            if p.trigger_label.len() > 64 {
                return Err("Choose a shorter trigger name.".into());
            }
            if !p.process.is_empty() {
                let process = p.process.to_lowercase();
                if !process.ends_with(".exe")
                    || process.contains(['/', '\\', ':'])
                    || !processes.insert(process)
                {
                    return Err(
                        "Use a unique process name, such as cs2.exe, for each profile.".into(),
                    );
                }
            }
            crate::math::exact_counts(p)?;
        }
        if !ids.contains(&self.active_id) {
            return Err("Select an existing profile.".into());
        }
        Ok(())
    }
}

pub fn path() -> Result<PathBuf, String> {
    // Deliberately independent of Tauri's identifier-based directory.
    std::env::var_os("APPDATA")
        .map(|p| PathBuf::from(p).join("Flick").join("profiles.json"))
        .ok_or_else(|| "Windows could not locate your settings folder.".into())
}
pub fn load() -> Result<Settings, String> {
    let path = path()?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let bytes = fs::read(path).map_err(|_| "Your profiles could not be opened.")?;
    let settings: Settings = serde_json::from_slice(&bytes)
        .map_err(|_| "Your profile file is damaged. It has been kept for recovery.")?;
    settings.validate()?;
    Ok(settings)
}
pub fn save(settings: &Settings) -> Result<(), String> {
    save_at(settings, &path()?)
}
fn save_at(settings: &Settings, path: &Path) -> Result<(), String> {
    use std::io::Write;
    settings.validate()?;
    fs::create_dir_all(path.parent().ok_or("Invalid settings folder.")?)
        .map_err(|_| "Your settings folder could not be created.")?;
    if let Ok(bytes) = fs::read(path) {
        let valid = serde_json::from_slice::<Settings>(&bytes).is_ok_and(|s| s.validate().is_ok());
        if !valid {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            fs::copy(path, path.with_extension(format!("recovery-{stamp}.json"))).map_err(|_| "The damaged profile file could not be backed up. Your settings were not changed.")?;
        }
    }
    let temporary = path.with_extension("tmp");
    let mut file = fs::File::create(&temporary).map_err(|_| "Your settings could not be saved.")?;
    file.write_all(&serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    // MoveFileExW replaces atomically on Windows, including an existing destination.
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let src: Vec<u16> = temporary.as_os_str().encode_wide().chain(Some(0)).collect();
    let dst: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe {
        MoveFileExW(
            src.as_ptr(),
            dst.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        return Err("Your settings could not be replaced. Check folder permissions.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialization_round_trip() {
        let mut settings = Settings::default();
        settings.profiles[0].name = "Training · 左".into();
        let serialized = serde_json::to_string(&settings).unwrap();
        let decoded: Settings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(decoded, settings);
        assert!(decoded.validate().is_ok());
    }
    #[test]
    fn older_settings_require_risk_acknowledgment() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        let mut old = value.as_object().unwrap().clone();
        old.remove("riskAcknowledged");
        let decoded: Settings = serde_json::from_value(old.into()).unwrap();
        assert!(!decoded.risk_acknowledged);
        assert!(decoded.validate().is_ok());
    }
    #[test]
    fn rejects_conflicts_and_empty_profiles() {
        let mut s = Settings::default();
        s.profiles[0].trigger = 0x77;
        assert!(s.validate().is_err());
        s.profiles.clear();
        assert!(s.validate().is_err());
    }
    #[test]
    fn atomic_save_replaces_previous_file() {
        let dir = std::env::temp_dir().join(format!("flick-test-{}", std::process::id()));
        let path = dir.join("profiles.json");
        let mut settings = Settings::default();
        save_at(&settings, &path).unwrap();
        settings.theme = "dark".into();
        save_at(&settings, &path).unwrap();
        let loaded: Settings = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded, settings);
        fs::remove_dir_all(dir).unwrap();
    }
}
