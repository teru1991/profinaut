use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub struct PidLock {
    path: std::path::PathBuf,
}

impl PidLock {
    pub fn acquire(path: &Path) -> Result<Self, String> {
        if path.exists() {
            return Err(format!("pid lock exists: {}", path.display()));
        }
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|e| format!("create lock {}: {e}", path.display()))?;

        let pid = std::process::id();
        writeln!(f, "{pid}").map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

impl Drop for PidLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
