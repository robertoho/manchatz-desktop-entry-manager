use ini::Ini;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub path: PathBuf,
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub comment: String,
    pub terminal: bool,
    pub categories: String,
    pub entry_type: String,
    pub mime_types: Vec<String>,
    pub mime_extensions: HashMap<String, String>,
}

impl DesktopEntry {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let conf = Ini::load_from_file(path)?;
        let section = conf
            .section(Some("Desktop Entry"))
            .ok_or("Missing Desktop Entry section")?;

        Ok(DesktopEntry {
            path: path.to_path_buf(),
            name: section.get("Name").unwrap_or("").to_string(),
            exec: section.get("Exec").unwrap_or("").to_string(),
            icon: section.get("Icon").unwrap_or("").to_string(),
            comment: section.get("Comment").unwrap_or("").to_string(),
            terminal: section.get("Terminal").unwrap_or("false") == "true",
            categories: section.get("Categories").unwrap_or("").to_string(),
            entry_type: section.get("Type").unwrap_or("Application").to_string(),
            mime_types: section
                .get("MimeType")
                .unwrap_or("")
                .split(';')
                .filter_map(|value| {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
                .collect(),
            mime_extensions: section
                .get("X-Manager-MimeExtensions")
                .unwrap_or("")
                .split(';')
                .filter_map(|pair| {
                    let trimmed = pair.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    let mut parts = trimmed.splitn(2, '=');
                    let mime = parts.next()?.trim();
                    let ext = parts.next()?.trim();
                    if mime.is_empty() {
                        return None;
                    }
                    Some((mime.to_string(), ext.to_string()))
                })
                .collect(),
        })
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conf = Ini::new();
        let mime_value = if self.mime_types.is_empty() {
            String::new()
        } else {
            format!("{};", self.mime_types.join(";"))
        };
        let mut extensions: Vec<_> = self
            .mime_extensions
            .iter()
            .map(|(mime, ext)| format!("{}={}", mime, ext))
            .collect();
        extensions.sort();
        let extensions_value = extensions.join(";");

        conf.with_section(Some("Desktop Entry"))
            .set("Type", &self.entry_type)
            .set("Name", &self.name)
            .set("Exec", &self.exec)
            .set("Icon", &self.icon)
            .set("Comment", &self.comment)
            .set("Terminal", if self.terminal { "true" } else { "false" })
            .set("Categories", &self.categories)
            .set("MimeType", &mime_value)
            .set("X-Manager-MimeExtensions", &extensions_value);

        conf.write_to_file(&self.path)?;
        Ok(())
    }
}

pub fn scan_desktop_files() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();

    let home_path = format!(
        "{}/.local/share/applications",
        std::env::var("HOME").unwrap_or_default()
    );

    let search_paths = vec![
        "/usr/share/applications",
        "/usr/local/share/applications",
        home_path.as_str(),
        "/var/lib/snapd/desktop/applications",  // Snap applications
        "/var/lib/flatpak/exports/share/applications",  // Flatpak applications
    ];

    for dir in search_paths {
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(desktop_entry) = DesktopEntry::from_file(&path) {
                        // Skip entries with NoDisplay=true
                        entries.push(desktop_entry);
                    } else {
                        eprintln!("Failed to parse: {}", path.display());
                    }
                }
            }
        } else {
            // Silently skip directories that don't exist or aren't accessible
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    entries
}
