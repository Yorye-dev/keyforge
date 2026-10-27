use super::layout::Layout;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyboardModel {
    pub id: String,
    pub name: String,
    pub revision: String,
    pub manufacturer: String,
    pub usb_product: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub qmk_target: String,
    pub development_board: Option<String>,
    pub processor: Option<String>,
    pub bootloader: Option<String>,
    pub lighting: Option<bool>,
    pub displays: DisplayMetadata,
    pub source_urls: Vec<String>,
    pub layouts: Vec<Layout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayMetadata {
    pub graphical: Option<bool>,
    pub technology: Option<String>,
    pub count: Option<u8>,
    pub resolution: Option<[u16; 2]>,
    pub controller: Option<String>,
    pub orientation: Option<u16>,
    pub quantum_painter: Option<bool>,
    pub dynamic_update: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct HidInterface {
    pub number: i32,
    pub usage_page: u16,
    pub usage_id: u16,
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial: Option<String>,
    pub interfaces: Vec<HidInterface>,
    pub bootloader_mode: Option<String>,
    pub simulated: bool,
    /// Only supplied by a protocol adapter, never deserialized from a layout/catalog.
    pub protocol_identity: Option<ProtocolIdentity>,
}

#[derive(Debug, Clone)]
pub struct ProtocolIdentity {
    pub model_id: String,
    pub device_id: String,
    pub protocol: String,
    pub simulated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentificationState {
    Confirmed,
    HighConfidence,
    Probable,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Identification {
    pub state: IdentificationState,
    pub candidates: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityState {
    Verified,
    Metadata,
    Unavailable,
    Unknown,
}

impl CapabilityState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Verified => "Comprobado",
            Self::Metadata => "Según catálogo · sin comprobar",
            Self::Unavailable => "No disponible",
            Self::Unknown => "Por comprobar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    LocalKeys,
    LocalLayers,
    DynamicKeys,
    Read,
    Write,
    Compile,
    Flash,
    Lighting,
    Macros,
    Displays,
    DynamicDisplays,
}

impl Capability {
    pub const ALL: [Self; 11] = [
        Self::LocalKeys,
        Self::LocalLayers,
        Self::DynamicKeys,
        Self::Read,
        Self::Write,
        Self::Compile,
        Self::Flash,
        Self::Lighting,
        Self::Macros,
        Self::Displays,
        Self::DynamicDisplays,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalKeys => "Editar teclas en un perfil",
            Self::LocalLayers => "Editar capas en un perfil",
            Self::DynamicKeys => "Configurar teclas al instante",
            Self::Read => "Leer el teclado",
            Self::Write => "Guardar en el teclado",
            Self::Compile => "Generar firmware",
            Self::Flash => "Actualizar firmware",
            Self::Lighting => "Iluminación",
            Self::Macros => "Macros",
            Self::Displays => "Pantallas gráficas",
            Self::DynamicDisplays => "Enviar diseños a las pantallas",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityAssessment {
    pub capability: Capability,
    pub state: CapabilityState,
    pub explanation: String,
    /// Application support is separate from a hardware metadata claim.
    pub actionable: bool,
}

#[derive(Debug, Clone)]
pub struct DeviceCapabilities(pub Vec<CapabilityAssessment>);
