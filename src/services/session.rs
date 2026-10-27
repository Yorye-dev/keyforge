use super::{
    QmkMetadataRepository,
    discovery::{DemoScenario, DeviceDiscoveryService, SimulatedDiscovery},
    identification::KeyboardIdentificationService,
};
use crate::{domain::devices::*, infrastructure::catalog::LocalQmkCatalog};

pub struct DiscoverySession {
    pub catalog: Option<LocalQmkCatalog>,
    pub scenario: DemoScenario,
    pub device: Option<DiscoveredDevice>,
    pub identification: Option<Identification>,
    pub error: Option<String>,
}

impl Default for DiscoverySession {
    fn default() -> Self {
        let result = LocalQmkCatalog::bundled();
        let error = result.as_ref().err().map(ToString::to_string);
        let mut session = Self {
            catalog: result.ok(),
            scenario: DemoScenario::UsbMatch,
            device: None,
            identification: None,
            error,
        };
        session.refresh();
        session
    }
}

impl DiscoverySession {
    pub fn refresh(&mut self) {
        self.device = None;
        self.identification = None;
        match SimulatedDiscovery(self.scenario).discover() {
            Ok(devices) => {
                self.device = devices.into_iter().next();
                if let (Some(device), Some(catalog)) = (&self.device, &self.catalog) {
                    self.identification =
                        Some(KeyboardIdentificationService::identify(device, catalog));
                    self.error = None;
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub fn model(&self, id: Option<&str>) -> Option<&KeyboardModel> {
        self.catalog.as_ref()?.find(id?)
    }
}
