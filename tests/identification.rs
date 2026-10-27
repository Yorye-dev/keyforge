use keyforge::{
    domain::devices::*,
    infrastructure::catalog::LocalQmkCatalog,
    services::{
        QmkMetadataRepository, capabilities::*, discovery::*,
        identification::KeyboardIdentificationService, session::DiscoverySession,
    },
};

fn device(scenario: DemoScenario) -> DiscoveredDevice {
    SimulatedDiscovery(scenario).discover().unwrap().remove(0)
}

#[test]
fn simulated_states_do_not_imply_write_access() {
    let catalog = LocalQmkCatalog::bundled().unwrap();
    for (scenario, expected) in [
        (DemoScenario::Confirmed, IdentificationState::Confirmed),
        (DemoScenario::UsbMatch, IdentificationState::HighConfidence),
        (DemoScenario::Incomplete, IdentificationState::Probable),
        (DemoScenario::Unknown, IdentificationState::Unknown),
    ] {
        let identity = KeyboardIdentificationService::identify(&device(scenario), &catalog);
        assert_eq!(identity.state, expected);
        assert!(!FirmwareCompatibilityService::flash_block_reason(Some(&identity)).is_empty());
        let model = identity.candidates.first().and_then(|id| catalog.find(id));
        assert!(
            DeviceCapabilitiesService::assess(model)
                .0
                .iter()
                .filter(|c| !matches!(
                    c.capability,
                    Capability::LocalKeys | Capability::LocalLayers
                ))
                .all(|c| !c.actionable && c.state != CapabilityState::Verified)
        );
    }
}

struct TestCatalog(Vec<KeyboardModel>);
impl QmkMetadataRepository for TestCatalog {
    fn models(&self) -> &[KeyboardModel] {
        &self.0
    }
}

#[test]
fn shared_usb_ids_remain_ambiguous_even_with_matching_names() {
    let source = LocalQmkCatalog::bundled().unwrap();
    let first = source.models()[0].clone();
    let mut second = first.clone();
    second.id = "test/other-revision".into();
    let catalog = TestCatalog(vec![first, second]);
    let identity =
        KeyboardIdentificationService::identify(&device(DemoScenario::UsbMatch), &catalog);
    assert_eq!(identity.state, IdentificationState::Probable);
    assert_eq!(identity.candidates.len(), 2);
}

#[test]
fn mismatched_protocol_identity_is_not_trusted() {
    let catalog = LocalQmkCatalog::bundled().unwrap();
    let mut dev = device(DemoScenario::Confirmed);
    dev.protocol_identity.as_mut().unwrap().device_id = "another-device".into();
    assert_eq!(
        KeyboardIdentificationService::identify(&dev, &catalog).state,
        IdentificationState::Unknown
    );
    let mut dev = device(DemoScenario::Confirmed);
    dev.simulated = false;
    assert_eq!(
        KeyboardIdentificationService::identify(&dev, &catalog).state,
        IdentificationState::Unknown
    );
}

#[test]
fn bootloader_and_name_mismatch_cannot_be_high_confidence() {
    let catalog = LocalQmkCatalog::bundled().unwrap();
    let mut dev = device(DemoScenario::UsbMatch);
    dev.bootloader_mode = Some("test bootloader".into());
    assert_eq!(
        KeyboardIdentificationService::identify(&dev, &catalog).state,
        IdentificationState::Probable
    );
    dev.bootloader_mode = None;
    dev.product = Some("Different keyboard".into());
    assert_eq!(
        KeyboardIdentificationService::identify(&dev, &catalog).state,
        IdentificationState::Probable
    );
}

#[test]
fn disconnect_clears_identity_and_keeps_catalog_for_local_selection() {
    let mut session = DiscoverySession::default();
    assert!(session.identification.is_some());
    session.scenario = DemoScenario::Disconnected;
    session.refresh();
    assert!(session.device.is_none() && session.identification.is_none());
    assert!(!session.catalog.as_ref().unwrap().models().is_empty());
    assert!(session.model(None).is_none());
}

#[test]
fn catalog_is_valid_and_display_metadata_never_detects_ips() {
    let catalog = LocalQmkCatalog::bundled().unwrap();
    assert_eq!(catalog.models().len(), 2);
    assert_eq!(
        catalog.models()[0]
            .layouts
            .iter()
            .map(|l| l.key_count)
            .collect::<Vec<_>>(),
        vec![42, 36]
    );
    for model in catalog.models() {
        for layout in &model.layouts {
            layout.validate().unwrap();
            assert!(layout.displays.is_empty());
        }
    }
    assert_eq!(
        DisplayCapabilitiesService::assess(catalog.find("crkbd/rev1")).0,
        CapabilityState::Metadata
    );
    assert_eq!(
        DisplayCapabilitiesService::assess(catalog.find("planck/rev6")).0,
        CapabilityState::Unknown
    );
    assert_eq!(
        DisplayCapabilitiesService::assess(None).0,
        CapabilityState::Unknown
    );
}

#[test]
fn rejects_invalid_catalogs() {
    let json = include_str!("../catalog/qmk.json");
    let original: serde_json::Value = serde_json::from_str(json).unwrap();
    for field in ["schema_version", "qmk_revision", "models"] {
        let mut value = original.clone();
        value[field] = serde_json::Value::Null;
        assert!(LocalQmkCatalog::parse(&value.to_string()).is_err());
    }
    let mut value = original;
    value["models"][1]["id"] = value["models"][0]["id"].clone();
    assert!(LocalQmkCatalog::parse(&value.to_string()).is_err());
}
