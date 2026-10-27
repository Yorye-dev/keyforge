use crate::{domain::devices::*, services::QmkMetadataRepository};

pub struct KeyboardIdentificationService;

impl KeyboardIdentificationService {
    pub fn identify(
        device: &DiscoveredDevice,
        catalog: &dyn QmkMetadataRepository,
    ) -> Identification {
        if let Some(proof) = &device.protocol_identity {
            if proof.device_id == device.id
                && proof.simulated == device.simulated
                && !proof.protocol.trim().is_empty()
                && catalog.find(&proof.model_id).is_some()
            {
                return Identification { state: IdentificationState::Confirmed, candidates: vec![proof.model_id.clone()], explanation: if device.simulated { "Identificación confirmada solo dentro del simulador. No se ha conectado hardware real." } else { "El adaptador ha confirmado el modelo mediante su protocolo." }.into() };
            }
            // Conflicting/invalid protocol evidence must not fall back to a confident USB match.
            return Identification { state: IdentificationState::Unknown, candidates: vec![], explanation: "La respuesta de identificación no corresponde al dispositivo o al catálogo. Puedes elegir un layout local.".into() };
        }
        let candidates: Vec<_> = catalog
            .models()
            .iter()
            .filter(|m| m.vendor_id == device.vendor_id && m.product_id == device.product_id)
            .collect();
        let exact_names = |m: &KeyboardModel| {
            device
                .manufacturer
                .as_deref()
                .is_some_and(|s| s.trim().eq_ignore_ascii_case(&m.manufacturer))
                && device
                    .product
                    .as_deref()
                    .is_some_and(|s| s.trim().eq_ignore_ascii_case(&m.usb_product))
        };
        let state = if candidates.len() == 1
            && exact_names(candidates[0])
            && device.bootloader_mode.is_none()
        {
            IdentificationState::HighConfidence
        } else if candidates.is_empty() {
            IdentificationState::Unknown
        } else {
            IdentificationState::Probable
        };
        Identification { state, candidates: candidates.iter().map(|m| m.id.clone()).collect(), explanation: match state {
            IdentificationState::HighConfidence => "Los datos USB coinciden con un modelo del catálogo. El firmware y sus funciones aún no están comprobados.",
            IdentificationState::Probable => "Hay coincidencias parciales. Elige visualmente la distribución; esto no confirma el hardware.",
            _ => "Todavía no reconocemos este teclado. Puedes explorar los modelos del catálogo sin conocer sus componentes.",
        }.into() }
    }
}
