// Inspired by ironbar: https://github.com/JakeStanger/ironbar/blob/master/src/desktop_file.rs

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use freedesktop_icon_lookup::Cache;
use fuzzy_matcher::FuzzyMatcher;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use ts_rs::TS;
use walkdir::{DirEntry, WalkDir};
use std::io::Result;

#[derive(Debug, Clone)]
enum DesktopFileRef {
    Unloaded(PathBuf),
    Loaded(DesktopFile),
}

impl DesktopFileRef {
    async fn get(&mut self, cache: Arc<Mutex<Cache>>) -> Result<DesktopFile> {
        match self {
            DesktopFileRef::Unloaded(path) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let path = path.clone();

                tauri::async_runtime::spawn(async move {
                    tx.send(Self::load(&path, &cache).await)
                });

                let file = rx.await
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to load desktop file"))?
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                *self = DesktopFileRef::Loaded(file.clone());

                Ok(file)
            }
            DesktopFileRef::Loaded(file) => Ok(file.clone()),
        }
    }

    async fn load(file_path: &Path, cache: &Mutex<Cache>) -> Result<DesktopFile> {
        let file = tokio::fs::File::open(file_path).await?;
        let cache = cache.lock().await;

        let mut desktop_file = DesktopFile::new(
            file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        let mut lines = BufReader::new(file).lines();

        let mut current_section = String::from("Desktop Entry");
        // We use a map because there can be duplicate sections that contribute to a single action
        let mut actions_map: HashMap<String, DesktopAction> = HashMap::new();
        let mut action_order: Vec<String> = Vec::new();

        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Sections
            if line.starts_with('[') && line.ends_with(']') {
                let section = &line[1..line.len() - 1];
                if section == "Desktop Entry" {
                    current_section = section.to_string();
                } else if let Some(action_name) = section.strip_prefix("Desktop Action ") {
                    current_section = format!("Desktop Action {}", action_name);
                    if !actions_map.contains_key(action_name) {
                    actions_map.insert(
                        action_name.to_string(),
                        DesktopAction {
                        name: None,
                        exec: None,
                        icon: None,
                        },
                    );
                    action_order.push(action_name.to_string());
                    }
                } else {
                    current_section = section.to_string();
                }
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();

            if current_section == "Desktop Entry" {
                match key {
                    "Name" => {
                        desktop_file.name.get_or_insert_with(|| value.to_string());
                    }
                    "Comment" => {
                        desktop_file.comment.get_or_insert_with(|| value.to_string());
                    }
                    "Type" => {
                        desktop_file.app_type.get_or_insert_with(|| value.to_string());
                    }
                    "StartupWMClass" => {
                        desktop_file.startup_wm_class.get_or_insert_with(|| value.to_string());
                    }
                    "Exec" => {
                        desktop_file.exec.get_or_insert_with(|| value.to_string());
                    }
                    "Icon" => {
                        if desktop_file.icon_path.is_none() {
                            desktop_file.icon_path = cache.lookup(value, None);
                        }
                    }
                    "Categories" => {
                        if desktop_file.categories.is_empty() {
                            desktop_file.categories = value.split(';').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                        }
                    }
                    "Keywords" => {
                        if desktop_file.keywords.is_empty() {
                            desktop_file.keywords = value.split(';').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                        }
                    }
                    "NoDisplay" => {
                        if desktop_file.no_display.is_none() {
                            desktop_file.no_display = Some(value.parse().unwrap_or(false));
                        }
                    }
                    _ => {}
                }
            } else if let Some(action_name) = current_section.strip_prefix("Desktop Action ") {
                let action = actions_map.entry(action_name.to_string()).or_insert_with(|| DesktopAction {
                    name: None,
                    exec: None,
                    icon: None,
                });
                match key {
                    "Name" => {
                        action.name.get_or_insert_with(|| value.to_string());
                    }
                    "Exec" => {
                        action.exec.get_or_insert_with(|| value.to_string());
                    }
                    "Icon" => {
                        action.icon.get_or_insert_with(|| value.to_string());
                    }
                    _ => {}
                }
            } else {
                // Unknown section type; just continue
            }
        }

        // Preserve order of actions as in file
        for action_name in action_order {
            if let Some(action) = actions_map.remove(&action_name) {
                desktop_file.actions.push(action);
            }
        }

        Ok(desktop_file)
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/DesktopFile.ts")]
pub struct DesktopFile {
    pub file_name: String,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub app_type: Option<String>,
    pub startup_wm_class: Option<String>,
    pub exec: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub no_display: Option<bool>,

    pub actions: Vec<DesktopAction>
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../bindings/DesktopFile.ts")]
pub struct DesktopAction {
    pub name: Option<String>,
    pub exec: Option<String>,
    pub icon: Option<String>
}

impl DesktopFile {
    fn new(file_name: String) -> Self {
        Self {
            file_name,
            name: None,
            comment: None,
            app_type: None,
            startup_wm_class: None,
            exec: None,
            icon_path: None,
            categories: vec![],
            keywords: vec![],
            no_display: None,
            actions: vec![]
        }
    }
}

type FileMap = HashMap<String, DesktopFileRef>;

/// Desktop file cache and resolver.
///
/// Files are lazy-loaded as required on resolution.
#[derive(Clone)]
pub struct DesktopFiles {
    files: Arc<Mutex<FileMap>>,
    pub icon_cache: Arc<Mutex<Cache>>
}

impl DesktopFiles {
    /// Creates a new instance, scanning the disk to generate a list of (unloaded) file refs.
    pub fn new() -> Self {
        let desktop_files: FileMap = dirs()
            .iter()
            .flat_map(|path| files(path))
            .map(|file| {
                (
                    file.file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_string()
                        .into(),
                    DesktopFileRef::Unloaded(file),
                )
            })
            .collect();
        
        let mut cache = Cache::new().expect("Failed to create icon cache");
        cache.load_default().expect("Failed to load icons from theme");
        
        Self {
            files: Arc::new(Mutex::new(desktop_files)),
            icon_cache: Arc::new(Mutex::new(cache))
        }
    }

    pub async fn get_all(&self) -> Result<Vec<DesktopFile>> {
        let mut files = self.files.lock().await;

        let mut res = Vec::with_capacity(files.len());
        for file in files.values_mut() {
            let file = file.get(self.icon_cache.clone()).await?;
            res.push(file);
        }

        Ok(res)
    }

    pub async fn fuzzy_search(&self, query: String) -> Result<Vec<DesktopFile>> {
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().ignore_case();
        
        let desktop_files = self.get_all().await?;
        
        // TODO: Rank by most recently launched?

        let mut rankings = desktop_files
            .iter()
            .filter_map(|file| {
                if file.no_display.unwrap_or(false) || file.name.is_none() {
                    return None; // Skip files marked as NoDisplay
                }

                // Match name, action names, or keywords
                let name_score = matcher.fuzzy_match(
                    &file.name.as_ref().unwrap(),
                    &query,
                ).map(|v| v * 3 / 2); // Boost name matches

                let action_score = file.actions.iter()
                    .filter_map(|action| action.name.as_ref().and_then(|n| matcher.fuzzy_match(n, &query)))
                    .max()
                    .map(|v| v * 5 / 4); // Slightly boost action matches over keywords

                let keywords_score = file.keywords.iter()
                    .filter_map(|keyword| matcher.fuzzy_match(keyword, &query))
                    .max();

                if name_score.is_none() && action_score.is_none() && keywords_score.is_none() {
                    return None;
                }

                let max_score = name_score.unwrap_or(0)
                    .max(keywords_score.unwrap_or(0))
                    .max(action_score.unwrap_or(0));

                Some((file, max_score))
            })
            .collect::<Vec<_>>();
        
        rankings.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(rankings.iter().map(|(entry, _)| (*entry).clone()).collect::<Vec<_>>())
    }

    pub async fn reload(&self) -> Result<()> {
        let mut file_data = self.files.lock().await;
        *file_data = dirs()
            .iter()
            .flat_map(|path| files(path))
            .map(|file| {
                (
                    file.file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_string().into(),
                    DesktopFileRef::Unloaded(file),
                )
            })
            .collect();
        let mut cache = self.icon_cache.lock().await;
        let themes = cache.themes().map(|s| s.to_string()).collect::<Vec<_>>();
        for theme in themes {
            if cache.load(theme.clone()).is_err() {
                println!("Failed to reload theme {}", theme);
            }
        }
        Ok(())
    }
}

/// Gets a list of paths to all directories
/// containing `.applications` files.
fn dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"), // system installed apps
        PathBuf::from("/var/lib/flatpak/exports/share/applications"), // flatpak apps
    ];

    let xdg_dirs = env::var("XDG_DATA_DIRS");
    if let Ok(xdg_dirs) = xdg_dirs {
        for mut xdg_dir in env::split_paths(&xdg_dirs) {
            xdg_dir.push("applications");
            dirs.push(xdg_dir);
        }
    }

    let user_dir = dirs::data_local_dir(); // user installed apps
    if let Some(mut user_dir) = user_dir {
        user_dir.push("applications");
        dirs.push(user_dir);
    }

    dirs.into_iter().filter(|dir| dir.exists()).collect()
}

/// Gets a list of all `.applications` files in the provided directory.
///
/// The directory is recursed to a maximum depth of 5.
fn files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .max_depth(5)
        .into_iter()
        .filter_map(|entry| {
            match entry {
                Ok(e) if e.file_type().is_file() => Some(e),
                _ => None,
            }
        })
        .map(DirEntry::into_path)
        .filter(|file| file.extension().unwrap_or_default() == "desktop")
        .collect()
}
