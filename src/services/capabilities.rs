use crate::domain::devices::*;

pub struct DisplayCapabilitiesService;
impl DisplayCapabilitiesService {
    pub fn assess(model: Option<&KeyboardModel>) -> (CapabilityState, String) {
        match model.and_then(|m| m.displays.graphical) {
            Some(true) => (CapabilityState::Metadata, "La definición menciona pantallas gráficas. No se ha verificado su presencia, número ni resolución en tu teclado.".into()),
            Some(false) => (CapabilityState::Metadata, "Esta definición declara que no incorpora pantallas; tu firmware puede ser distinto.".into()),
            None => (CapabilityState::Unknown, "No hay información suficiente sobre las pantallas. No asumimos que existan.".into()),
        }
    }
}

pub struct DeviceCapabilitiesService;
impl DeviceCapabilitiesService {
    pub fn assess(model: Option<&KeyboardModel>) -> DeviceCapabilities {
        DeviceCapabilities(Capability::ALL.into_iter().map(|capability| {
            let (state, explanation) = match capability {
                Capability::LocalKeys | Capability::LocalLayers => (CapabilityState::Unavailable, "La edición de perfiles llegará en la siguiente fase. Ya puedes explorar y seleccionar layouts.".into()),
                Capability::Compile | Capability::Flash => (CapabilityState::Unavailable, "La generación y actualización de firmware aún no están implementadas.".into()),
                Capability::Displays => DisplayCapabilitiesService::assess(model),
                Capability::Lighting if model.and_then(|m| m.lighting) == Some(true) => (CapabilityState::Metadata, "El catálogo declara iluminación; esta aplicación todavía no la controla.".into()),
                _ => (CapabilityState::Unknown, "Necesita un protocolo compatible y comprobado. Todavía no hay un adaptador físico implementado.".into()),
            };
            CapabilityAssessment { capability, state, explanation, actionable: false }
        }).collect())
    }
}

pub struct FirmwareCompatibilityService;
impl FirmwareCompatibilityService {
    /// Fail closed until artifact, target, revision, bootloader and transport checks exist.
    pub fn flash_block_reason(identification: Option<&Identification>) -> &'static str {
        if !identification.is_some_and(|i| i.state == IdentificationState::Confirmed) {
            "El modelo exacto no está confirmado. La selección manual y los datos USB no autorizan una actualización."
        } else {
            "La identidad no demuestra compatibilidad de firmware. Faltan validación del archivo, destino y adaptador de flasheo."
        }
    }
}
