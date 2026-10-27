use crate::{domain::keymap::*, services::editor::KeymapEditor};
use eframe::egui;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct EditorPanel {
    anchor: Option<(u32, String)>,
    draft: Option<Behavior>,
    layer_name: String,
    profile_name: String,
    names_anchor: Option<(u32, String, String)>,
    search: String,
}
impl EditorPanel {
    pub fn invalidate(&mut self) {
        self.anchor = None;
        self.names_anchor = None;
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        editor: &mut KeymapEditor,
        key: Option<&str>,
    ) -> Option<String> {
        let mut error = None;
        let names_anchor = (
            editor.active(),
            editor.layer().name.clone(),
            editor.profile().name.clone(),
        );
        if self.names_anchor.as_ref() != Some(&names_anchor) {
            self.layer_name = names_anchor.1.clone();
            self.profile_name = names_anchor.2.clone();
            self.names_anchor = Some(names_anchor);
        }
        ui.heading("Capas");
        let mut active = editor.active();
        egui::ComboBox::from_id_salt("active_layer")
            .selected_text(&editor.layer().name)
            .show_ui(ui, |ui| {
                for layer in &editor.profile().layers {
                    ui.selectable_value(&mut active, layer.id, &layer.name);
                }
            });
        if active != editor.active() {
            report(editor.select_layer(active), &mut error);
            self.invalidate();
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editor.profile().layers.len() < MAX_LAYERS,
                    egui::Button::new("Nueva"),
                )
                .clicked()
            {
                report(editor.add_layer(false), &mut error);
                self.invalidate();
            }
            if ui
                .add_enabled(
                    editor.profile().layers.len() < MAX_LAYERS,
                    egui::Button::new("Duplicar"),
                )
                .clicked()
            {
                report(editor.add_layer(true), &mut error);
                self.invalidate();
            }
            if ui.button("Eliminar").clicked() {
                report(editor.delete_layer(), &mut error);
                self.invalidate();
            }
        });
        ui.label("Nombre de la capa");
        ui.text_edit_singleline(&mut self.layer_name);
        if ui.button("Renombrar capa").clicked() {
            report(editor.rename_layer(self.layer_name.clone()), &mut error);
            self.invalidate();
        }
        ui.label(format!(
            "{} / 32 capas locales",
            editor.profile().layers.len()
        ));
        ui.separator();
        ui.heading("Asignación");
        let Some(key) = key else {
            ui.label("Pulsa una tecla del teclado para cambiar su comportamiento.");
            return error;
        };
        let anchor = (editor.active(), key.to_owned());
        if self.anchor.as_ref() != Some(&anchor) {
            self.draft = editor.layer().bindings.get(key).cloned();
            self.anchor = Some(anchor);
            self.search.clear();
        }
        ui.label(format!("Tecla: {key}"));
        ui.horizontal(|ui| {
            if ui.button("Copiar").clicked() {
                report(editor.copy(key), &mut error);
            }
            if ui
                .add_enabled(editor.has_clipboard(), egui::Button::new("Pegar"))
                .clicked()
            {
                report(editor.paste(key), &mut error);
                self.anchor = None;
            }
        });
        if let Some(draft) = &mut self.draft {
            behavior_form(ui, draft, &editor.profile().layers, &mut self.search);
            let changed = editor.layer().bindings.get(key) != Some(draft);
            if ui
                .add_enabled(changed, egui::Button::new("Asignar a esta tecla"))
                .clicked()
            {
                report(editor.assign(key, draft.clone()), &mut error);
            }
            if changed {
                ui.small("Pulsa Asignar para incorporar esta selección al perfil.");
            }
        }
        ui.separator();
        ui.collapsing("Nombre del perfil", |ui| {
            ui.text_edit_singleline(&mut self.profile_name);
            if ui.button("Renombrar perfil").clicked() {
                report(editor.rename_profile(self.profile_name.clone()), &mut error);
                self.names_anchor = None;
            }
        });
        ui.small("Los símbolos dependen de la distribución de idioma del sistema. GUI equivale a Windows / Super.");
        ui.small("Las funciones duales y de capa se guardan localmente; requieren un firmware compatible para aplicarlas.");
        error
    }
}
fn report(result: Result<(), EditError>, error: &mut Option<String>) {
    if let Err(e) = result {
        *error = Some(e.to_string());
    }
}
fn kind(behavior: &Behavior) -> usize {
    match behavior {
        Behavior::Key { .. } => 0,
        Behavior::Modifier { .. } => 1,
        Behavior::Modified { .. } => 2,
        Behavior::Momentary { .. } => 3,
        Behavior::Toggle { .. } => 4,
        Behavior::ModTap { .. } => 5,
        Behavior::LayerTap { .. } => 6,
        Behavior::Media { .. } => 7,
        Behavior::Transparent => 8,
        Behavior::None => 9,
    }
}
fn behavior_form(
    ui: &mut egui::Ui,
    behavior: &mut Behavior,
    layers: &[Layer],
    search: &mut String,
) {
    const NAMES: [&str; 10] = [
        "Tecla",
        "Modificador",
        "Combinación",
        "Capa mientras mantienes",
        "Activar / desactivar capa",
        "Toque / modificador",
        "Toque / capa",
        "Multimedia",
        "Transparente",
        "Sin acción",
    ];
    let previous = kind(behavior);
    let mut selected = previous;
    egui::ComboBox::from_id_salt("behavior_kind")
        .selected_text(NAMES[selected])
        .show_ui(ui, |ui| {
            for (i, name) in NAMES.iter().enumerate() {
                ui.selectable_value(&mut selected, i, *name);
            }
        });
    if selected != previous {
        let key = KeyCode::Letter('A');
        let modifier = Modifier::LeftCtrl;
        let layer = layers.first().map_or(0, |l| l.id);
        *behavior = match selected {
            0 => Behavior::Key { key },
            1 => Behavior::Modifier { modifier },
            2 => Behavior::Modified {
                key,
                modifiers: BTreeSet::from([modifier]),
            },
            3 => Behavior::Momentary { layer },
            4 => Behavior::Toggle { layer },
            5 => Behavior::ModTap { key, modifier },
            6 => Behavior::LayerTap { key, layer },
            7 => Behavior::Media {
                action: Media::PlayPause,
            },
            8 => Behavior::Transparent,
            _ => Behavior::None,
        };
    }
    match behavior {
        Behavior::Key { key } => key_picker(ui, key, search),
        Behavior::Modifier { modifier } => modifier_picker(ui, modifier),
        Behavior::Modified { key, modifiers } => {
            key_picker(ui, key, search);
            ui.label("Mantener también:");
            for modifier in Modifier::ALL {
                let mut checked = modifiers.contains(&modifier);
                if ui.checkbox(&mut checked, modifier.label()).changed() {
                    if checked {
                        modifiers.insert(modifier);
                    } else {
                        modifiers.remove(&modifier);
                    }
                }
            }
        }
        Behavior::Momentary { layer } | Behavior::Toggle { layer } => {
            layer_picker(ui, layer, layers)
        }
        Behavior::ModTap { key, modifier } => {
            ui.label("Al tocar:");
            key_picker(ui, key, search);
            ui.label("Al mantener:");
            modifier_picker(ui, modifier);
        }
        Behavior::LayerTap { key, layer } => {
            ui.label("Al tocar:");
            key_picker(ui, key, search);
            ui.label("Al mantener:");
            layer_picker(ui, layer, layers);
        }
        Behavior::Media { action } => {
            egui::ComboBox::from_id_salt("media")
                .selected_text(action.label())
                .show_ui(ui, |ui| {
                    for candidate in Media::ALL {
                        ui.selectable_value(action, candidate, candidate.label());
                    }
                });
        }
        Behavior::Transparent => {
            ui.label("Deja pasar la asignación de una capa inferior.");
        }
        Behavior::None => {
            ui.label("Esta tecla no realizará ninguna acción.");
        }
    }
}
fn key_picker(ui: &mut egui::Ui, key: &mut KeyCode, search: &mut String) {
    ui.add(egui::TextEdit::singleline(search).hint_text("Buscar tecla…"));
    egui::ComboBox::from_id_salt("keycode")
        .selected_text(key.label())
        .height(220.0)
        .show_ui(ui, |ui| {
            for candidate in KeyCode::palette()
                .into_iter()
                .filter(|k| k.label().to_lowercase().contains(&search.to_lowercase()))
            {
                ui.selectable_value(key, candidate, candidate.label());
            }
        });
}
fn modifier_picker(ui: &mut egui::Ui, modifier: &mut Modifier) {
    egui::ComboBox::from_id_salt("modifier")
        .selected_text(modifier.label())
        .show_ui(ui, |ui| {
            for candidate in Modifier::ALL {
                ui.selectable_value(modifier, candidate, candidate.label());
            }
        });
}
fn layer_picker(ui: &mut egui::Ui, layer: &mut u32, layers: &[Layer]) {
    let name = layers
        .iter()
        .find(|l| l.id == *layer)
        .map_or("Capa inexistente", |l| l.name.as_str());
    egui::ComboBox::from_id_salt("target_layer")
        .selected_text(name)
        .show_ui(ui, |ui| {
            for candidate in layers {
                ui.selectable_value(layer, candidate.id, &candidate.name);
            }
        });
}
