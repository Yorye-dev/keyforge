use crate::{
    domain::{devices::*, layout::Layout},
    infrastructure::layouts,
    services::{
        QmkMetadataRepository,
        capabilities::{DeviceCapabilitiesService, FirmwareCompatibilityService},
        discovery::DemoScenario,
        session::DiscoverySession,
    },
    ui::keyboard,
};
use eframe::egui;
use std::{path::Path, sync::mpsc};

pub struct KeyboardApp {
    layout: Option<Layout>,
    path: String,
    error: Option<String>,
    pending: Option<mpsc::Receiver<Result<Layout, layouts::LoadError>>>,
    selected: Option<String>,
    zoom: f32,
    pan: egui::Vec2,
    discovery: DiscoverySession,
    selected_model: Option<String>,
    show_models: bool,
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
        let layout = first.and_then(|m| m.layouts.first()).cloned();
        Self {
            layout,
            path: layouts::example_path().to_string_lossy().into_owned(),
            error: None,
            pending: None,
            selected: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            discovery,
            selected_model,
            show_models: true,
        }
    }
    fn reset_view(&mut self) {
        self.selected = None;
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
    }

    fn poll_load(&mut self, ctx: &egui::Context) {
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    match result {
                        Ok(layout) => {
                            self.layout = Some(layout);
                            self.selected_model = None;
                            self.error = None;
                            self.reset_view();
                        }
                        Err(error) => {
                            self.error = Some(format!(
                                "{error}. Revisa la ruta y el formato del archivo; tu layout anterior sigue disponible."
                            ))
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.error =
                        Some("La carga se interrumpió. Puedes volver a intentarlo.".into());
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(50))
                }
            }
        }
    }

    fn advanced(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Información avanzada", |ui| {
            if let Some(device) = &self.discovery.device {
                ui.monospace(format!(
                    "USB {:04X}:{:04X}",
                    device.vendor_id, device.product_id
                ));
                ui.label(format!(
                    "Serie: {}",
                    device.serial.as_deref().unwrap_or("No disponible")
                ));
                for interface in &device.interfaces {
                    ui.label(format!(
                        "Interfaz {} · Usage {:04X}:{:04X}",
                        interface.number, interface.usage_page, interface.usage_id
                    ));
                }
            }
            if let Some(model) = self.discovery.model(self.selected_model.as_deref()) {
                ui.label("Metadatos de la ficha seleccionada, no del hardware:");
                ui.label(format!("Destino QMK: {}", model.qmk_target));
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
            ui.separator();
            ui.label("Cargar una distribución JSON propia");
            ui.text_edit_singleline(&mut self.path);
            if ui
                .add_enabled(self.pending.is_none(), egui::Button::new("Cargar archivo"))
                .clicked()
            {
                let path = self.path.clone();
                let ctx = ui.ctx().clone();
                let (sender, receiver) = mpsc::channel();
                match std::thread::Builder::new()
                    .name("layout-loader".into())
                    .spawn(move || {
                        let _ = sender.send(layouts::load(Path::new(&path)));
                        ctx.request_repaint();
                    }) {
                    Ok(_) => self.pending = Some(receiver),
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            if self.pending.is_some() {
                ui.spinner();
            }
            if ui
                .add_enabled(
                    self.pending.is_none(),
                    egui::Button::new("Ejemplo con dos IPS ilustrativas"),
                )
                .clicked()
            {
                match layouts::demo() {
                    Ok(layout) => {
                        self.layout = Some(layout);
                        self.selected_model = None;
                        self.error = None;
                        self.reset_view();
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
        });
    }
}

impl eframe::App for KeyboardApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_load(root.ctx());
        egui::Panel::top("header").show(root, |ui| {
            ui.horizontal(|ui| {
                ui.heading("KEYFORGE");
                ui.separator();
                ui.label("Tu teclado, a tu manera");
                ui.label(
                    egui::RichText::new("DEMOSTRACIÓN · SIN USB REAL")
                        .color(egui::Color32::LIGHT_GREEN),
                );
            });
        });
        egui::Panel::left("inspector").resizable(false).default_size(290.0).show(root, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Empezar");
                ui.label("Prueba la detección con un teclado simulado o elige su forma en el catálogo.");
                let old = self.discovery.scenario;
                egui::ComboBox::from_id_salt("demo_scenario").selected_text(self.discovery.scenario.label()).show_ui(ui, |ui| {
                    for scenario in DemoScenario::ALL { ui.selectable_value(&mut self.discovery.scenario, scenario, scenario.label()); }
                });
                if old != self.discovery.scenario {
                    self.discovery.refresh(); self.show_models = true;
                    self.selected_model = None;
                }
                ui.separator();
                if let Some(identity) = &self.discovery.identification {
                    ui.strong(match identity.state {
                        IdentificationState::Confirmed => "Modelo confirmado en el simulador",
                        IdentificationState::HighConfidence => "Coincidencia USB de alta confianza",
                        IdentificationState::Probable => "Modelo por confirmar",
                        IdentificationState::Unknown => "Teclado sin identificar",
                    });
                    ui.label(&identity.explanation);
                    if let Some(catalog) = &self.discovery.catalog {
                        for id in &identity.candidates {
                            if let Some(model) = catalog.find(id) { ui.label(format!("Coincidencia: {} · {}", model.name, model.revision)); }
                        }
                    }
                } else { ui.strong("Puedes explorar sin conectar un teclado"); }
                if ui.button("Elegir distribución visualmente").clicked() { self.show_models = !self.show_models; }
                ui.separator();
                ui.collapsing("Funciones disponibles", |ui| {
                    ui.label("Estas etiquetas describen la ficha seleccionada. El catálogo no confirma tu hardware.");
                    for assessment in DeviceCapabilitiesService::assess(self.discovery.model(self.selected_model.as_deref())).0 {
                        ui.strong(assessment.capability.label());
                        ui.label(assessment.state.label()).on_hover_text(assessment.explanation);
                        ui.add_space(4.0);
                    }
                    ui.label("La edición local de teclas y capas está pendiente. Puedes inspeccionar cualquier layout sin identificar el teclado.");
                });
                ui.collapsing("Actualización de firmware", |ui| {
                    ui.label(FirmwareCompatibilityService::flash_block_reason(self.discovery.identification.as_ref()));
                    ui.label("El asistente de herramientas QMK se añadirá en una fase posterior.");
                });
                self.advanced(ui);
                if let Some(error) = &self.discovery.error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
                if let Some(error) = &self.error { ui.colored_label(egui::Color32::LIGHT_RED, error); }
                ui.separator();
                if let Some(layout) = &self.layout {
                    ui.strong(&layout.name);
                    ui.label(format!("{} teclas · vista local", layout.keys.len()));
                    if self.selected_model.is_none() { ui.label("Selección local independiente de la detección."); }
                    if !layout.displays.is_empty() { ui.label("Las pantallas de este layout son una representación visual, no hardware detectado."); }
                    if let Some(key) = self.selected.as_ref().and_then(|id| layout.keys.iter().find(|k| &k.id == id)) {
                        ui.label(format!("Tecla {} · {}", key.label, if key.thumb {"pulgar"} else {"estándar"}));
                        ui.collapsing("Datos de la tecla", |ui| { ui.label(format!("ID: {} · matriz {:?} · {}°", key.id, key.matrix, key.geometry.rotation)); });
                    }
                }
            });
        });
        egui::CentralPanel::default().show(root, |ui| {
            if self.show_models {
                ui.heading("Elige la forma de tu teclado");
                ui.label("Seleccionar una vista local no confirma el modelo conectado ni autoriza actualizarlo.");
                let mut choice = None;
                egui::ScrollArea::horizontal().id_salt("models").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(catalog) = &self.discovery.catalog {
                            for model in catalog.models() { for layout in &model.layouts {
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    ui.vertical(|ui| {
                                    ui.set_width(210.0);
                                    ui.strong(&layout.name);
                                    keyboard::thumbnail(ui, layout);
                                    if ui.add_enabled(self.pending.is_none(), egui::Button::new("Ver este teclado")).clicked() { choice = Some((model.id.clone(), layout.clone())); }
                                    });
                                });
                            } }
                        }
                    });
                });
                if let Some((id, layout)) = choice { self.layout = Some(layout); self.selected_model = Some(id); self.error = None; self.reset_view(); }
                ui.separator();
            }
            ui.horizontal(|ui| {
                ui.heading("Vista del teclado");
                ui.add(egui::Slider::new(&mut self.zoom, 0.5..=3.0).text("Zoom"));
                if ui.button("Centrar").clicked() { self.zoom = 1.0; self.pan = egui::Vec2::ZERO; }
            });
            ui.label("Arrastra para desplazar · pulsa una tecla para inspeccionarla · etiquetas de posición, no asignaciones");
            if let Some(layout) = &self.layout { keyboard::draw(ui, layout, self.zoom, &mut self.pan, &mut self.selected); }
        });
    }
}
