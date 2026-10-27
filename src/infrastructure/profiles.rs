use crate::domain::keymap::{EditError, Profile};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

const MAX_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("No se pudo acceder al perfil: {0}")]
    Io(#[from] std::io::Error),
    #[error("El archivo no es un perfil JSON válido: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Invalid(#[from] EditError),
    #[error("El perfil supera el límite de 8 MiB.")]
    TooLarge,
}
pub fn parse(json: &str) -> Result<Profile, ProfileError> {
    if json.len() as u64 > MAX_BYTES {
        return Err(ProfileError::TooLarge);
    }
    let profile: Profile = serde_json::from_str(json)?;
    profile.validate()?;
    Ok(profile)
}
pub fn load(path: &Path) -> Result<Profile, ProfileError> {
    let mut json = String::new();
    File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_string(&mut json)?;
    parse(&json)
}
pub fn save(path: &Path, profile: &Profile) -> Result<(), ProfileError> {
    profile.validate()?;
    let data = serde_json::to_vec_pretty(profile)?;
    if data.len() as u64 > MAX_BYTES {
        return Err(ProfileError::TooLarge);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(&data)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}
