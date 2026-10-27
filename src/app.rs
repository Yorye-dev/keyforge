use crate::{
    domain::{devices::*, keymap::Profile, layout::Layout},
    infrastructure::layouts,
    services::{
        QmkMetadataRepository,
        capabilities::{DeviceCapabilitiesService, FirmwareCompatibilityService},
        discovery::DemoScenario,
        editor::KeymapEditor,
        file_jobs::{FileJob, FileOutcome, FileRequest},
        session::DiscoverySession,
    },
    ui::{editor::EditorPanel, keyboard},
};
use eframe::egui;
use std::path::PathBuf;

enum Transition {
    Layout(Box<Layout>, Option<String>),
    OpenProfile,
    OpenLayout,
    Exit,
}

pub struct KeyboardApp {
    editor: Option<KeymapEditor>,
    editor_panel: EditorPanel,
    selected: Option<String>,
    zoom: f32,
    pan: egui::Vec2,
    discovery: DiscoverySession,
    selected_model: Option<String>,
    show_models: bool,
    job: Option<FileJob>,
    pending: Option<Transition>,
    closing: bool,
    error: Option<String>,
    file_path: Option<PathBuf>,
    status: String,
}

impl Default for KeyboardApp {
    fn default() -> Self {
        Self::new()
    }
}
impl KeyboardApp {
    pub fn new() -> Self {
        let discovery = DiscoverySession::default();
        let first = discovery.catalog.as_ref().and_then(|c| c.models().first());
        let selected_model = first.map(|m| m.id.clone());
        let result = first
            .and_then(|m| m.layouts.first())
            .cloned()
            .ok_or_else(|| "No se pudo cargar el catálogo. Puedes abrir un perfil o layout.".into())
            .and_then(|layout| {
                Profile::blank(layout)
                    .and_then(KeymapEditor::new)
                    .map_err(|e| e.to_string())
            });
        let error = result.as_ref().err().cloned();
        Self {
            editor: result.ok(),
            editor_panel: EditorPanel::default(),
            selected: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            discovery,
            selected_model,
            show_models: false,
            job: None,
            pending: None,
            closing: false,
            error,
            file_path: None,
            status: "Perfil nuevo: todas las teclas empiezan sin asignación.".into(),
        }
    }
    fn dirty(&self) -> bool {
        self.editor.as_ref().is_some_and(KeymapEditor::dirty)
    }
    fn reset_view(&mut self) {
        self.selected = None;
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
        self.editor_panel.invalidate();
    }
    fn start(&mut self, request: FileRequest) {
        match FileJob::start(request) {
            Ok(job) => {
                self.job = Some(job);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
    fn save(&mut self) {
        if let Some(editor) = &self.editor {
            self.start(FileRequest::SaveProfile(Box::new(editor.profile().clone())));
        }
    }
    fn request(&mut self, action: Transition, ctx: &egui::Context) {
        if self.dirty() {
            self.pending = Some(action);
        } else {
            self.execute(action, ctx);
        }
    }
    fn execute(&mut self, action: Transition, ctx: &egui::Context) {
        match action {
            Transition::Layout(layout, model) => {
                match Profile::blank(*layout).and_then(KeymapEditor::new) {
                    Ok(editor) => {
                        self.editor = Some(editor);
                        self.selected_model = model;
                        self.file_path = None;
                        self.error = None;
                        self.reset_view();
                        self.status = "Perfil nuevo: asigna las teclas antes de guardarlo.".into();
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            Transition::OpenProfile => self.start(FileRequest::OpenProfile),
            Transition::OpenLayout => self.start(FileRequest::OpenLayout),
            Transition::Exit => {
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
    fn poll(&mut self, ctx: &egui::Context) {
        let Some(job) = &self.job else {
            return;
        };
        let Some(result) = job.poll() else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            return;
        };
        self.job = None;
        match result {
            Ok(FileOutcome::Saved(path)) => {
                if let Some(editor) = &mut self.editor {
                    editor.mark_saved();
                }
                self.status =
                    "Perfil guardado en el ordenador. No se ha enviado al teclado.".into();
                self.file_path = Some(path);
                if let Some(action) = self.pending.take() {
                    self.execute(action, ctx);
                }
            }
            Ok(FileOutcome::Profile(profile, path)) => match KeymapEditor::new(*profile) {
                Ok(editor) => {
                    self.editor = Some(editor);
                    self.selected_model = None;
                    self.file_path = Some(path);
                    self.reset_view();
                    self.error = None;
                    self.status = "Perfil local abierto y validado.".into();
                }
                Err(error) => self.error = Some(error.to_string()),
            },
            Ok(FileOutcome::Layout(layout)) => {
                self.execute(Transition::Layout(Box::new(layout), None), ctx)
            }
            Ok(FileOutcome::Cancelled) => {
                self.status = "Operación cancelada; tu trabajo sigue disponible.".into();
            }
            Err(error) => {
                self.error = Some(format!(
                    "{error} Tu trabajo sigue disponible; puedes elegir otro archivo."
                ))
            }
        }
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.job.is_some() || self.pending.is_some() || ctx.egui_wants_keyboard_input() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
            self.save();
            return;
        }
        if let Some(editor) = &mut self.editor {
            if ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z)
                    || i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)
            }) {
                editor.redo();
                self.editor_panel.invalidate();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Z)) {
                editor.undo();
                self.editor_panel.invalidate();
            }
        }
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("KEYFORGE");
            ui.separator();
            if ui.button("Abrir perfil").clicked() {
                self.request(Transition::OpenProfile, ui.ctx());
            }
            if ui
                .add_enabled(self.editor.is_some(), egui::Button::new("Guardar perfil…"))
                .clicked()
            {
                self.save();
            }
            if ui.button("Cambiar teclado").clicked() {
                self.show_models = true;
            }
            if let Some(editor) = &mut self.editor {
                if ui
                    .add_enabled(editor.can_undo(), egui::Button::new("Deshacer"))
                    .clicked()
                {
                    editor.undo();
                    self.editor_panel.invalidate();
                }
                if ui
                    .add_enabled(editor.can_redo(), egui::Button::new("Rehacer"))
                    .clicked()
                {
                    editor.redo();
                    self.editor_panel.invalidate();
                }
            }
            ui.separator();
            ui.colored_label(egui::Color32::LIGHT_GREEN, "EDICIÓN LOCAL");
            if self.dirty() {
                ui.colored_label(egui::Color32::YELLOW, "* Cambios sin guardar");
            }
        });
    }
    fn device_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tu teclado");
        if let Some(editor) = &self.editor {
            ui.strong(&editor.profile().layout.name);
            ui.label(format!("{} teclas", editor.profile().layout.keys.len()));
            ui.label(&editor.profile().name);
        }
        ui.label("Puedes editar sin conectar ni identificar un dispositivo.");
        ui.separator();
        ui.collapsing("Demostración de detección", |ui| {
            ui.label("Sin conexión USB real");
            let old = self.discovery.scenario;
            egui::ComboBox::from_id_salt("demo_scenario")
                .selected_text(self.discovery.scenario.label())
                .show_ui(ui, |ui| {
                    for scenario in DemoScenario::ALL {
                        ui.selectable_value(
                            &mut self.discovery.scenario,
                            scenario,
                            scenario.label(),
                        );
                    }
                });
            if old != self.discovery.scenario {
                self.discovery.refresh();
                self.selected_model = None;
            }
            if let Some(identity) = &self.discovery.identification {
                ui.strong(match identity.state {
                    IdentificationState::Confirmed => "Confirmado en el simulador",
                    IdentificationState::HighConfidence => "Coincidencia USB de alta confianza",
                    IdentificationState::Probable => "Modelo por confirmar",
                    IdentificationState::Unknown => "Teclado sin identificar",
                });
                ui.label(&identity.explanation);
            } else {
                ui.label("Sin dispositivo. La edición local sigue disponible.");
            }
        });
        ui.collapsing("Funciones disponibles", |ui| {
            for assessment in DeviceCapabilitiesService::assess(
                self.discovery.model(self.selected_model.as_deref()),
            )
            .0
            {
                ui.strong(assessment.capability.label());
                ui.label(assessment.state.label())
                    .on_hover_text(assessment.explanation);
                ui.add_space(4.0);
            }
        });
        ui.collapsing("Información avanzada", |ui| {
            if let Some(device) = &self.discovery.device {
                ui.monospace(format!(
                    "USB {:04X}:{:04X}",
                    device.vendor_id, device.product_id
                ));
                ui.label(format!(
                    "Serie: {}",
                    device.serial.as_deref().unwrap_or("Sin datos")
                ));
            }
            if let Some(model) = self.discovery.model(self.selected_model.as_deref()) {
                ui.label("Ficha seleccionada; hardware sin verificar.");
                ui.label(format!("QMK: {}", model.qmk_target));
                ui.label(format!(
                    "Placa: {}",
                    model.development_board.as_deref().unwrap_or("Sin datos")
                ));
                ui.label(format!(
                    "Procesador: {}",
                    model.processor.as_deref().unwrap_or("Sin confirmar")
                ));
                ui.label(format!(
                    "Bootloader: {}",
                    model.bootloader.as_deref().unwrap_or("Sin confirmar")
                ));
                for source in &model.source_urls {
                    ui.hyperlink_to("Fuente QMK", source);
                }
            }
            if ui.button("Abrir layout JSON…").clicked() {
                self.request(Transition::OpenLayout, ui.ctx());
            }
            if ui.button("Ejemplo de dos IPS").clicked() {
                match layouts::demo() {
                    Ok(layout) => {
                        self.request(Transition::Layout(Box::new(layout), None), ui.ctx())
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            ui.label(FirmwareCompatibilityService::flash_block_reason(
                self.discovery.identification.as_ref(),
            ));
        });
        ui.separator();
        ui.small("Las asignaciones son locales y no cambian el teclado conectado. Las pantallas dibujadas son ilustrativas.");
        if let Some(error) = &self.discovery.error {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }
    }
    fn gallery(&mut self, ctx: &egui::Context) {
        if !self.show_models {
            return;
        }
        let mut open = true;
        let mut choice = None;
        egui::Window::new("Elige la forma de tu teclado")
            .open(&mut open)
            .resizable(true)
            .default_width(740.0)
            .show(ctx, |ui| {
                ui.label("Se creará un perfil vacío. La elección no confirma el modelo conectado.");
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(catalog) = &self.discovery.catalog {
                            for model in catalog.models() {
                                for layout in &model.layouts {
                                    egui::Frame::group(ui.style()).show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.set_width(210.0);
                                            ui.strong(&layout.name);
                                            keyboard::thumbnail(ui, layout);
                                            if ui.button("Crear perfil").clicked() {
                                                choice = Some((layout.clone(), model.id.clone()));
                                            }
                                        });
                                    });
                                }
                            }
                        }
                    });
                });
            });
        self.show_models = open;
        if let Some((layout, id)) = choice {
            self.show_models = false;
            self.request(Transition::Layout(Box::new(layout), Some(id)), ctx);
        }
    }
    fn confirm_pending(&mut self, ctx: &egui::Context) {
        if self.pending.is_none() || self.job.is_some() {
            return;
        }
        let mut decision = 0;
        egui::Window::new("Cambios sin guardar")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Guarda tu perfil antes de continuar para conservar los cambios.");
                ui.horizontal(|ui| {
                    if ui.button("Guardar y continuar…").clicked() {
                        decision = 1;
                    }
                    if ui.button("Descartar y continuar").clicked() {
                        decision = 2;
                    }
                    if ui.button("Cancelar").clicked() {
                        decision = 3;
                    }
                });
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::LIGHT_RED, error);
                }
            });
        match decision {
            1 => self.save(),
            2 => {
                if let Some(action) = self.pending.take() {
                    self.execute(action, ctx);
                }
            }
            3 => self.pending = None,
            _ => {}
        }
    }
}
impl eframe::App for KeyboardApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        self.poll(&ctx);
        if !self.closing && ctx.input(|i| i.viewport().close_requested()) {
            if self.job.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            } else if self.dirty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.pending = Some(Transition::Exit);
            }
        }
        self.shortcuts(&ctx);
        let enabled = self.job.is_none() && self.pending.is_none();
        egui::Panel::top("header").show(root, |ui| {
            ui.add_enabled_ui(enabled, |ui| self.toolbar(ui));
        });
        egui::Panel::bottom("status").show(root, |ui| {
            if self.job.is_some() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Selecciona un archivo en el diálogo del sistema…");
                });
            } else if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            } else {
                ui.label(&self.status);
            }
            if let Some(path) = &self.file_path {
                ui.small(path.display().to_string());
            }
        });
        egui::Panel::left("device")
            .default_size(225.0)
            .resizable(true)
            .show(root, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_enabled_ui(enabled, |ui| self.device_panel(ui));
                });
            });
        egui::Panel::right("editor")
            .default_size(275.0)
            .resizable(true)
            .show(root, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_enabled_ui(enabled, |ui| {
                        if let Some(editor) = &mut self.editor
                            && let Some(error) =
                                self.editor_panel.show(ui, editor, self.selected.as_deref())
                        {
                            self.error = Some(error);
                        }
                    });
                });
            });
        egui::CentralPanel::default().show(root, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if let Some(editor) = &self.editor {
                        ui.heading(&editor.layer().name);
                    }
                    ui.add(egui::Slider::new(&mut self.zoom, 0.5..=3.0).text("Zoom"));
                    if ui.button("Centrar").clicked() {
                        self.zoom = 1.0;
                        self.pan = egui::Vec2::ZERO;
                    }
                });
                ui.label("Pulsa una tecla para asignarla · arrastra para desplazar");
                ui.small("TRNS: transparente · — Sin acción · borde ámbar: cambio sin guardar");
                if let Some(editor) = &self.editor {
                    let labels = editor
                        .layer()
                        .bindings
                        .iter()
                        .map(|(id, b)| (id.clone(), b.label(&editor.profile().layers)))
                        .collect();
                    keyboard::draw(
                        ui,
                        &editor.profile().layout,
                        self.zoom,
                        &mut self.pan,
                        &mut self.selected,
                        &labels,
                        &editor.changed_keys(),
                    );
                }
            });
        });
        if enabled {
            self.gallery(&ctx);
        }
        self.confirm_pending(&ctx);
    }
}
