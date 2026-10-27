use crate::domain::layout::{Layout, ValidationError};
use std::{fs::File, io::Read, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("No se pudo leer el archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON incorrecto: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("El layout excede el límite de 2 MiB")]
    TooLarge,
}

pub fn parse(json: &str) -> Result<Layout, LoadError> {
    if json.len() > 2 * 1024 * 1024 {
        return Err(LoadError::TooLarge);
    }
    let layout: Layout = serde_json::from_str(json)?;
    layout.validate()?;
    Ok(layout)
}

pub fn load(path: &Path) -> Result<Layout, LoadError> {
    let mut json = String::new();
    File::open(path)?
        .take(2 * 1024 * 1024 + 1)
        .read_to_string(&mut json)?;
    parse(&json)
}

pub fn demo() -> Result<Layout, LoadError> {
    parse(include_str!("../../layouts/corne.json"))
}

/// Locate editable examples when launched from Explorer or a different working directory.
pub fn example_path() -> std::path::PathBuf {
    let relative = Path::new("layouts/corne.json");
    if relative.is_file() {
        return std::fs::canonicalize(relative).unwrap_or_else(|_| relative.into());
    }
    if let Ok(executable) = std::env::current_exe() {
        for directory in executable.ancestors().skip(1).take(4) {
            let candidate = directory.join(relative);
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    relative.into()
}
