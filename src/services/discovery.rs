use crate::domain::devices::*;

pub trait DeviceDiscoveryService {
    fn discover(&self) -> Result<Vec<DiscoveredDevice>, DiscoveryError>;
}

#[derive(Debug, thiserror::Error)]
#[error("No se pudieron buscar teclados: {0}")]
pub struct DiscoveryError(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoScenario {
    Confirmed,
    UsbMatch,
    Incomplete,
    Unknown,
    Disconnected,
}

impl DemoScenario {
    pub const ALL: [Self; 5] = [
        Self::UsbMatch,
        Self::Confirmed,
        Self::Incomplete,
        Self::Unknown,
        Self::Disconnected,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Confirmed => "Corne · confirmación simulada",
            Self::UsbMatch => "Corne · coincidencia USB simulada",
            Self::Incomplete => "Corne · datos incompletos",
            Self::Unknown => "Teclado desconocido",
            Self::Disconnected => "Sin teclado",
        }
    }
}

pub struct SimulatedDiscovery(pub DemoScenario);

impl DeviceDiscoveryService for SimulatedDiscovery {
    fn discover(&self) -> Result<Vec<DiscoveredDevice>, DiscoveryError> {
        if self.0 == DemoScenario::Disconnected {
            return Ok(vec![]);
        }
        let mut device = DiscoveredDevice {
            id: "demo-corne".into(),
            vendor_id: 0x4653,
            product_id: 1,
            manufacturer: Some("foostan".into()),
            product: Some("Corne".into()),
            serial: Some("SIMULATED-001".into()),
            interfaces: vec![HidInterface {
                number: 0,
                usage_page: 1,
                usage_id: 6,
            }],
            bootloader_mode: None,
            simulated: true,
            protocol_identity: None,
        };
        match self.0 {
            DemoScenario::Confirmed => {
                device.protocol_identity = Some(ProtocolIdentity {
                    model_id: "crkbd/rev1".into(),
                    device_id: device.id.clone(),
                    protocol: "Adaptador de pruebas (no es un protocolo USB)".into(),
                    simulated: true,
                })
            }
            DemoScenario::Incomplete => {
                device.manufacturer = None;
                device.product = None;
            }
            DemoScenario::Unknown => {
                device.vendor_id = 0xffff;
                device.product_id = 0xffff;
                device.product = Some("Teclado de ejemplo".into());
                device.manufacturer = None;
            }
            _ => {}
        }
        Ok(vec![device])
    }
}
