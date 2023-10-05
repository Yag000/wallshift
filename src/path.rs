use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::configuration::Settings;

/// A wrapper for a path that can be either a file or a folder.
pub enum File {
    Image(ImagePath),
    Folder(AnimtaedFolder),
}

impl File {
    pub fn new(path: PathBuf) -> Option<Self> {
        if !path.exists() {
            None
        } else if path.is_dir() {
            Some(Self::Folder(AnimtaedFolder::new(path)))
        } else {
            Some(Self::Image(ImagePath::new(path)?))
        }
    }

    pub(crate) fn is_animated(&mut self, settings: &Settings) -> bool {
        match self {
            Self::Image(image) => image.is_animated(settings),
            Self::Folder(folder) => folder.path().exists(),
        }
    }

    pub(crate) fn get_animated_wallpaper_name(&self) -> String {
        match self {
            Self::Image(image) => image.get_animated_wallpaper_name(),
            Self::Folder(folder) => folder.name().to_owned(),
        }
    }
}

impl ToString for File {
    fn to_string(&self) -> String {
        match self {
            Self::Image(image) => image.to_string(),
            Self::Folder(folder) => folder.to_string(),
        }
    }
}

impl From<String> for File {
    fn from(path: String) -> Self {
        File::new(PathBuf::from(path)).unwrap()
    }
}

impl From<PathBuf> for File {
    fn from(path: PathBuf) -> Self {
        File::new(path).unwrap()
    }
}

/// A wrapper for a path that is a file.
/// It can be either an animated wallpaper or a static wallpaper.
pub struct ImagePath {
    path: PathBuf,
    animated_info: Option<AnimatedInfo>,
    animated_info_cached: bool,
}

impl ImagePath {
    pub fn new(path: PathBuf) -> Option<Self> {
        if !path.exists() || path.is_dir() {
            None
        } else {
            let instance = Self {
                path,
                animated_info: None,
                animated_info_cached: false,
            };
            Some(instance)
        }
    }

    /// Checks if the given path is an animated wallpaper, if the
    /// value has been set before it will return the cached value, else
    /// it will check if the path is an animated wallpaper and cache the result.
    ///
    /// An animated wallperper can be either a file or a folder. If it is a folder
    /// it must be contained in the wallpaper directory.
    /// If it is a file it must be contained in a folder that is contained in the wallpaper directory.
    ///  
    ///  Example:
    ///  wallpaper_dir
    ///  |--- folder1
    ///  |    |--- wallpaper1
    ///  |-- wallpaper2
    ///
    ///  folder1 is an animated wallpaper
    ///  wallpaper1 is an animated wallpaper
    ///  wallpaper2 is not an animated wallpaper
    pub fn is_animated(&mut self, settings: &Settings) -> bool {
        if self.animated_info_cached {
            return self.animated_info.is_some();
        }

        let is_animated = self.check_if_animated(settings);
        if is_animated {
            self.update_animated_info();
        }
        self.animated_info_cached = true;
        is_animated
    }

    /// Helper function for is_animated.
    fn check_if_animated(&mut self, settings: &Settings) -> bool {
        if let Some(parent) = self.path.parent() {
            if self.path.is_dir() {
                if let Some(str) = parent.to_str() {
                    return str == settings.wallpaper_dir;
                }
                return false;
            } else if let Some(parent) = parent.parent() {
                if let Some(str) = parent.to_str() {
                    return str == settings.wallpaper_dir;
                }
            } else {
                return false;
            }
        }
        false
    }

    fn update_animated_info(&mut self) {
        self.animated_info = Some(AnimatedInfo::new(&self.path));
    }

    /// Gets the name of the folder that contains the given path.
    /// If the folder name is not cached it will cache it.
    ///
    /// # Panics
    ///
    /// If the path is not an animated wallpaper it may panic.
    pub fn get_animated_wallpaper_name(&self) -> String {
        self.animated_info.as_ref().unwrap().animated_folder.clone()
    }

    pub fn get_animated_number(&self) -> Option<u32> {
        if let Some(info) = self.animated_info.as_ref() {
            return Some(info.animated_number);
        }
        None
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get_sleep_time(&mut self, settings: &Settings) -> u64 {
        if self.is_animated(settings) {
            let number_of_wallpapers =
                read_dir(self.path.parent().expect("failed to get parent directory"))
                    .expect("failed to open wallpaper directory")
                    .count();
            settings.sleep_time / number_of_wallpapers as u64
        } else {
            settings.sleep_time
        }
    }
}

impl ToString for ImagePath {
    fn to_string(&self) -> String {
        self.path.to_str().unwrap().to_owned()
    }
}

impl From<String> for ImagePath {
    fn from(path: String) -> Self {
        //TODO:
        Self::new(PathBuf::from(path)).unwrap()
    }
}

/// A wrapper for a path that is a folder.
pub struct AnimtaedFolder {
    path: PathBuf,
    name: String,
}

impl AnimtaedFolder {
    fn new(path: PathBuf) -> Self {
        let path_str = path.file_name().unwrap().to_str().unwrap().to_owned();
        Self {
            path,
            name: path_str,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ToString for AnimtaedFolder {
    fn to_string(&self) -> String {
        self.path.to_str().unwrap().to_owned()
    }
}

struct AnimatedInfo {
    animated_folder: String,
    animated_number: u32,
}

impl AnimatedInfo {
    fn new(path: &Path) -> Self {
        let animated_folder = Self::update_animated_folder(path);
        let animated_number = Self::update_animated_number(path);

        Self {
            animated_folder,
            animated_number,
        }
    }

    fn update_animated_folder(path: &Path) -> String {
        path.parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    fn update_animated_number(path: &Path) -> u32 {
        path.file_stem()
            .expect("failed to get file name")
            .to_str()
            .expect("failed to convert file name to str")
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
            .parse::<u32>()
            .expect("failed to parse animated number")
    }
}
