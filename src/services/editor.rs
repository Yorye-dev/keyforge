use crate::domain::keymap::*;
use std::collections::BTreeSet;

const HISTORY_LIMIT: usize = 100;

#[derive(Clone)]
struct Snapshot {
    layers: Vec<Layer>,
    active: u32,
    name: String,
}

pub struct KeymapEditor {
    profile: Profile,
    active: u32,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    saved_layers: Vec<Layer>,
    saved_name: String,
    clipboard: Option<Behavior>,
    next_layer_id: Option<u32>,
}
impl KeymapEditor {
    pub fn new(profile: Profile) -> Result<Self, EditError> {
        profile.validate()?;
        let active = profile.layers[0].id;
        let next_layer_id = profile
            .layers
            .iter()
            .map(|l| l.id)
            .max()
            .and_then(|id| id.checked_add(1));
        Ok(Self {
            saved_layers: profile.layers.clone(),
            saved_name: profile.name.clone(),
            profile,
            active,
            undo: vec![],
            redo: vec![],
            clipboard: None,
            next_layer_id,
        })
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn active(&self) -> u32 {
        self.active
    }
    pub fn layer(&self) -> &Layer {
        self.profile
            .layers
            .iter()
            .find(|l| l.id == self.active)
            .unwrap_or(&self.profile.layers[0])
    }
    pub fn select_layer(&mut self, id: u32) -> Result<(), EditError> {
        if !self.profile.layers.iter().any(|l| l.id == id) {
            return Err(EditError("La capa ya no existe.".into()));
        }
        self.active = id;
        Ok(())
    }
    pub fn dirty(&self) -> bool {
        self.saved_layers != self.profile.layers || self.saved_name != self.profile.name
    }
    pub fn mark_saved(&mut self) {
        self.saved_layers = self.profile.layers.clone();
        self.saved_name = self.profile.name.clone();
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn has_clipboard(&self) -> bool {
        self.clipboard.is_some()
    }
    pub fn changed_keys(&self) -> BTreeSet<String> {
        let saved = self.saved_layers.iter().find(|l| l.id == self.active);
        self.layer()
            .bindings
            .iter()
            .filter(|(key, value)| saved.and_then(|l| l.bindings.get(*key)) != Some(*value))
            .map(|(key, _)| key.clone())
            .collect()
    }
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            layers: self.profile.layers.clone(),
            active: self.active,
            name: self.profile.name.clone(),
        }
    }
    fn restore(&mut self, snapshot: Snapshot) {
        self.profile.layers = snapshot.layers;
        self.active = snapshot.active;
        self.profile.name = snapshot.name;
    }
    fn change(
        &mut self,
        edit: impl FnOnce(&mut Profile, &mut u32) -> Result<(), EditError>,
    ) -> Result<(), EditError> {
        let before = self.snapshot();
        if let Err(error) =
            edit(&mut self.profile, &mut self.active).and_then(|()| self.profile.validate())
        {
            self.restore(before);
            return Err(error);
        }
        if before.layers != self.profile.layers || before.name != self.profile.name {
            self.undo.push(before);
            if self.undo.len() > HISTORY_LIMIT {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        Ok(())
    }
    pub fn assign(&mut self, key_id: &str, behavior: Behavior) -> Result<(), EditError> {
        self.change(|p, active| {
            let binding = p
                .layers
                .iter_mut()
                .find(|l| l.id == *active)
                .and_then(|l| l.bindings.get_mut(key_id))
                .ok_or_else(|| EditError("La tecla no existe en esta capa.".into()))?;
            *binding = behavior;
            Ok(())
        })
    }
    pub fn rename_profile(&mut self, name: String) -> Result<(), EditError> {
        self.change(|p, _| {
            p.name = name.trim().into();
            Ok(())
        })
    }
    pub fn rename_layer(&mut self, name: String) -> Result<(), EditError> {
        self.change(|p, active| {
            if let Some(layer) = p.layers.iter_mut().find(|l| l.id == *active) {
                layer.name = name.trim().into();
            }
            Ok(())
        })
    }
    pub fn add_layer(&mut self, duplicate: bool) -> Result<(), EditError> {
        let id = self
            .next_layer_id
            .ok_or_else(|| EditError("No quedan identificadores de capa disponibles.".into()))?;
        let result = self.change(|p, active| {
            if p.layers.len() == MAX_LAYERS {
                return Err(EditError(
                    "Has alcanzado el límite local de 32 capas.".into(),
                ));
            }
            let bindings = if duplicate {
                p.layers
                    .iter()
                    .find(|l| l.id == *active)
                    .ok_or_else(|| EditError("Capa no encontrada.".into()))?
                    .bindings
                    .clone()
            } else {
                p.layout
                    .keys
                    .iter()
                    .map(|k| (k.id.clone(), Behavior::Transparent))
                    .collect()
            };
            p.layers.push(Layer {
                id,
                name: format!("Capa {id}"),
                bindings,
            });
            *active = id;
            Ok(())
        });
        if result.is_ok() {
            self.next_layer_id = id.checked_add(1);
        }
        result
    }
    pub fn delete_layer(&mut self) -> Result<(), EditError> {
        self.change(|p, active| {
            if p.layers.len() == 1 {
                return Err(EditError("Debes conservar al menos una capa.".into()));
            }
            if p.layers
                .iter()
                .filter(|l| l.id != *active)
                .flat_map(|l| l.bindings.values())
                .any(|b| b.target_layer() == Some(*active))
            {
                return Err(EditError(
                    "Otra capa utiliza esta capa. Cambia esas asignaciones antes de eliminarla."
                        .into(),
                ));
            }
            p.layers.retain(|l| l.id != *active);
            *active = p.layers[0].id;
            Ok(())
        })
    }
    pub fn copy(&mut self, key_id: &str) -> Result<(), EditError> {
        self.clipboard = Some(
            self.layer()
                .bindings
                .get(key_id)
                .ok_or_else(|| EditError("Selecciona una tecla.".into()))?
                .clone(),
        );
        Ok(())
    }
    pub fn paste(&mut self, key_id: &str) -> Result<(), EditError> {
        self.assign(
            key_id,
            self.clipboard
                .clone()
                .ok_or_else(|| EditError("Primero copia una asignación.".into()))?,
        )
    }
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo.pop() {
            self.redo.push(self.snapshot());
            self.restore(snapshot);
        }
    }
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo.pop() {
            self.undo.push(self.snapshot());
            self.restore(snapshot);
        }
    }
}
