use crate::{domain::devices::KeyboardModel, services::QmkMetadataRepository};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalQmkCatalog {
    pub schema_version: u32,
    pub qmk_revision: String,
    models: Vec<KeyboardModel>,
}

#[derive(Debug, thiserror::Error)]
#[error("Catálogo no válido: {0}")]
pub struct CatalogError(pub String);

impl LocalQmkCatalog {
    pub fn bundled() -> Result<Self, CatalogError> {
        Self::parse(include_str!("../../catalog/qmk.json"))
    }
    pub fn parse(json: &str) -> Result<Self, CatalogError> {
        if json.len() > 2 * 1024 * 1024 {
            return Err(CatalogError("archivo demasiado grande".into()));
        }
        let catalog: Self = serde_json::from_str(json).map_err(|e| CatalogError(e.to_string()))?;
        if catalog.schema_version != 1
            || catalog.qmk_revision.len() != 40
            || !catalog.qmk_revision.bytes().all(|b| b.is_ascii_hexdigit())
            || catalog.models.is_empty()
        {
            return Err(CatalogError(
                "versión, revisión o lista de modelos incorrecta".into(),
            ));
        }
        let mut ids = HashSet::new();
        for model in &catalog.models {
            if model.id.trim().is_empty()
                || model.name.trim().is_empty()
                || model.qmk_target.trim().is_empty()
                || !ids.insert(&model.id)
                || model.layouts.is_empty()
                || model.source_urls.is_empty()
            {
                return Err(CatalogError("modelo duplicado o incompleto".into()));
            }
            if model.displays.count == Some(0)
                || model.displays.resolution.is_some_and(|r| r.contains(&0))
            {
                return Err(CatalogError("metadatos de pantalla incorrectos".into()));
            }
            let mut layouts = HashSet::new();
            for layout in &model.layouts {
                layout.validate().map_err(|e| CatalogError(e.to_string()))?;
                if !layouts.insert(&layout.id) {
                    return Err(CatalogError("layout repetido".into()));
                }
            }
        }
        Ok(catalog)
    }
}
impl QmkMetadataRepository for LocalQmkCatalog {
    fn models(&self) -> &[KeyboardModel] {
        &self.models
    }
}
