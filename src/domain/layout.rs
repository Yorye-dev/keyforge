use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, thiserror::Error)]
#[error("Layout no válido: {0}")]
pub struct ValidationError(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub variant: String,
    pub key_count: usize,
    pub matrix: [u16; 2],
    pub halves: Vec<String>,
    pub keys: Vec<Key>,
    #[serde(default)]
    pub displays: Vec<Display>,
    #[serde(default)]
    pub usb: Vec<UsbId>,
    #[serde(default)]
    pub firmware: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsbId {
    pub vendor_id: u16,
    pub product_id: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub rotation: f32,
}

impl Geometry {
    /// Clockwise degrees around the rectangle's center; coordinates are key units.
    pub fn corners(&self) -> [[f32; 2]; 4] {
        let (s, c) = self.rotation.to_radians().sin_cos();
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;
        [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]].map(|[x, y]| {
            let (x, y) = (x * self.width, y * self.height);
            [cx + x * c - y * s, cy + x * s + y * c]
        })
    }

    fn valid(&self) -> bool {
        [self.x, self.y, self.width, self.height, self.rotation]
            .iter()
            .all(|v| v.is_finite() && v.abs() <= 10000.0)
            && self.width >= 0.1
            && self.height >= 0.1
            && self.width <= 100.0
            && self.height <= 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub id: String,
    pub label: String,
    pub matrix: [u16; 2],
    pub half: String,
    pub geometry: Geometry,
    #[serde(default)]
    pub thumb: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Display {
    pub id: String,
    pub half: String,
    pub geometry: Geometry,
    /// None means the real hardware resolution is unknown.
    pub resolution: Option<[u16; 2]>,
    pub technology: String,
}

impl Layout {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let fail = |s: &str| Err(ValidationError(s.into()));
        if self.schema_version != 1 {
            return fail("version de formato no soportada");
        }
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return fail("identificador y nombre obligatorios");
        }
        if self.keys.is_empty() || self.keys.len() > 1024 || self.key_count != self.keys.len() {
            return fail("cantidad de teclas incorrecta (1..1024)");
        }
        if self.matrix.contains(&0) {
            return fail("matriz vacía");
        }
        let halves: HashSet<_> = self.halves.iter().collect();
        if halves.is_empty()
            || halves.len() != self.halves.len()
            || self.halves.iter().any(|s| s.trim().is_empty())
        {
            return fail("mitades vacías o duplicadas");
        }
        let mut ids = HashSet::new();
        let mut cells = HashSet::new();
        for key in &self.keys {
            if key.id.trim().is_empty() || !ids.insert(&key.id) {
                return fail("identificador de tecla vacío o duplicado");
            }
            if !cells.insert(key.matrix)
                || key.matrix[0] >= self.matrix[0]
                || key.matrix[1] >= self.matrix[1]
            {
                return fail("posición de matriz duplicada o fuera de rango");
            }
            if !halves.contains(&key.half) || !key.geometry.valid() {
                return fail("mitad o geometría de tecla incorrecta");
            }
        }
        let mut displays = HashSet::new();
        if self.displays.len() > 16 {
            return fail("demasiadas pantallas");
        }
        for display in &self.displays {
            if display.id.trim().is_empty()
                || !displays.insert(&display.id)
                || !halves.contains(&display.half)
                || !display.geometry.valid()
                || display.resolution.is_some_and(|r| r.contains(&0))
            {
                return fail("pantalla inválida");
            }
        }
        Ok(())
    }

    pub fn bounds(&self) -> ([f32; 2], [f32; 2]) {
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        for g in self
            .keys
            .iter()
            .map(|k| &k.geometry)
            .chain(self.displays.iter().map(|d| &d.geometry))
        {
            for p in g.corners() {
                for i in 0..2 {
                    min[i] = min[i].min(p[i]);
                    max[i] = max[i].max(p[i]);
                }
            }
        }
        (min, max)
    }
}
