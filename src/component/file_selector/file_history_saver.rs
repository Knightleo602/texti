use crate::component::file_selector::file_history::HISTORY_FILE_NAME;
use crate::config::Config;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Utility for adding newly opened or saved files to the file history
/// Files already present are re-added to the top of the list.
/// Stores newly added files and only writes to the file when this struct is dropped.
#[derive(Default)]
pub struct FileHistorySaver {
    data_file_dir: PathBuf,
    current: HashSet<PathBuf>,
    new: HashSet<PathBuf>,
}

impl From<&Config> for FileHistorySaver {
    fn from(value: &Config) -> Self {
        Self::new(value.config.data_dir.clone())
    }
}

impl FileHistorySaver {
    pub fn new(data_dir: PathBuf) -> FileHistorySaver {
        let mut saver = Self::default();
        saver.load_from_data_dir(&data_dir);
        saver
    }
    pub fn load_from_config(&mut self, config: &Config) {
        self.load_from_data_dir(&config.config.data_dir);
    }
    pub fn load_from_data_dir(&mut self, data_dir: &Path) {
        let file = data_dir.join(HISTORY_FILE_NAME);
        if let Ok(file_content) = fs::read_to_string(&file) {
            let file_lines = file_content.lines();
            for line in file_lines {
                let path = PathBuf::from(line);
                if path.is_file() {
                    self.current.insert(path);
                }
            }
        }
        self.data_file_dir = file;
        self.new = HashSet::new();
    }
    pub fn push_to_history<P: AsRef<Path>>(&mut self, file: P) {
        let file = file.as_ref().to_path_buf();
        self.current.remove(&file);
        self.new.insert(file);
    }
    pub fn awaiting_write(&self) -> bool {
        !self.new.is_empty()
    }
    fn save_new_files(&mut self) -> color_eyre::Result<()> {
        if !self.awaiting_write() {
            return Ok(());
        }
        let f = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.data_file_dir)?;
        let mut buf = BufWriter::new(&f);
        for new in self.new.iter() {
            let path = new.to_string_lossy().to_string();
            writeln!(buf, "{path}")?;
        }
        for current in self.current.iter() {
            let path = current.to_string_lossy().to_string();
            writeln!(buf, "{path}")?;
        }
        buf.flush()?;
        self.current.extend(self.new.drain());
        Ok(())
    }
}

impl Drop for FileHistorySaver {
    fn drop(&mut self) {
        let _ = self.save_new_files();
    }
}
