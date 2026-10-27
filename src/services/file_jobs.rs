use crate::{
    domain::{keymap::Profile, layout::Layout},
    infrastructure::{layouts, profiles},
};
use std::{path::PathBuf, sync::mpsc, thread};

pub enum FileRequest {
    OpenProfile,
    SaveProfile(Box<Profile>),
    OpenLayout,
}
pub enum FileOutcome {
    Profile(Box<Profile>, PathBuf),
    Layout(Layout),
    Saved(PathBuf),
    Cancelled,
}
pub struct FileJob {
    receiver: mpsc::Receiver<Result<FileOutcome, String>>,
}
impl FileJob {
    pub fn start(request: FileRequest) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .name("profile-files".into())
            .spawn(move || {
                let result = run(request);
                let _ = sender.send(result);
            })
            .map_err(|e| e.to_string())?;
        Ok(Self { receiver })
    }
    pub fn poll(&self) -> Option<Result<FileOutcome, String>> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(
                "La operación de archivo se interrumpió. Tu perfil sigue en memoria.".into(),
            )),
        }
    }
}
fn run(request: FileRequest) -> Result<FileOutcome, String> {
    let dialog = rfd::FileDialog::new().add_filter("JSON", &["json"]);
    match request {
        FileRequest::OpenProfile => {
            match dialog.set_title("Abrir perfil de Keyforge").pick_file() {
                Some(path) => profiles::load(&path)
                    .map(|p| FileOutcome::Profile(Box::new(p), path))
                    .map_err(|e| e.to_string()),
                None => Ok(FileOutcome::Cancelled),
            }
        }
        FileRequest::OpenLayout => match dialog.set_title("Abrir layout de Keyforge").pick_file() {
            Some(path) => layouts::load(&path)
                .map(FileOutcome::Layout)
                .map_err(|e| e.to_string()),
            None => Ok(FileOutcome::Cancelled),
        },
        FileRequest::SaveProfile(profile) => match dialog
            .set_title("Guardar perfil local")
            .set_file_name("perfil-keyforge.json")
            .save_file()
        {
            Some(mut path) => {
                if path.extension().is_none() {
                    path.set_extension("json");
                }
                profiles::save(&path, &profile).map_err(|e| e.to_string())?;
                Ok(FileOutcome::Saved(path))
            }
            None => Ok(FileOutcome::Cancelled),
        },
    }
}
