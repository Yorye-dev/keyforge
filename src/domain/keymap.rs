use super::layout::Layout;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_LAYERS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamedKey {
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Quote,
    Grave,
    Comma,
    Period,
    Slash,
    CapsLock,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PageUp,
    Delete,
    End,
    PageDown,
    Right,
    Left,
    Down,
    Up,
}
impl NamedKey {
    pub const ALL: [Self; 30] = [
        Self::Enter,
        Self::Escape,
        Self::Backspace,
        Self::Tab,
        Self::Space,
        Self::Minus,
        Self::Equal,
        Self::LeftBracket,
        Self::RightBracket,
        Self::Backslash,
        Self::Semicolon,
        Self::Quote,
        Self::Grave,
        Self::Comma,
        Self::Period,
        Self::Slash,
        Self::CapsLock,
        Self::PrintScreen,
        Self::ScrollLock,
        Self::Pause,
        Self::Insert,
        Self::Home,
        Self::PageUp,
        Self::Delete,
        Self::End,
        Self::PageDown,
        Self::Right,
        Self::Left,
        Self::Down,
        Self::Up,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Enter => "Enter",
            Self::Escape => "Esc",
            Self::Backspace => "Borrar",
            Self::Tab => "Tab",
            Self::Space => "Espacio",
            Self::Minus => "-",
            Self::Equal => "=",
            Self::LeftBracket => "[",
            Self::RightBracket => "]",
            Self::Backslash => "\\",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::Grave => "`",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::CapsLock => "Bloq Mayús",
            Self::PrintScreen => "Impr Pant",
            Self::ScrollLock => "Bloq Despl",
            Self::Pause => "Pausa",
            Self::Insert => "Insert",
            Self::Home => "Inicio",
            Self::PageUp => "Re Pág",
            Self::Delete => "Supr",
            Self::End => "Fin",
            Self::PageDown => "Av Pág",
            Self::Right => "→",
            Self::Left => "←",
            Self::Down => "↓",
            Self::Up => "↑",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCode {
    Letter(char),
    Digit(u8),
    Function(u8),
    Named(NamedKey),
}
impl KeyCode {
    pub fn valid(self) -> bool {
        match self {
            Self::Letter(c) => c.is_ascii_uppercase(),
            Self::Digit(n) => n <= 9,
            Self::Function(n) => (1..=24).contains(&n),
            Self::Named(_) => true,
        }
    }
    pub fn label(self) -> String {
        match self {
            Self::Letter(c) => c.to_string(),
            Self::Digit(n) => n.to_string(),
            Self::Function(n) => format!("F{n}"),
            Self::Named(k) => k.label().into(),
        }
    }
    pub fn palette() -> Vec<Self> {
        ('A'..='Z')
            .map(Self::Letter)
            .chain((0..=9).map(Self::Digit))
            .chain((1..=24).map(Self::Function))
            .chain(NamedKey::ALL.map(Self::Named))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Modifier {
    LeftCtrl,
    LeftShift,
    LeftAlt,
    LeftGui,
    RightCtrl,
    RightShift,
    RightAlt,
    RightGui,
}
impl Modifier {
    pub const ALL: [Self; 8] = [
        Self::LeftCtrl,
        Self::LeftShift,
        Self::LeftAlt,
        Self::LeftGui,
        Self::RightCtrl,
        Self::RightShift,
        Self::RightAlt,
        Self::RightGui,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::LeftCtrl => "Ctrl",
            Self::LeftShift => "Shift",
            Self::LeftAlt => "Alt",
            Self::LeftGui => "GUI",
            Self::RightCtrl => "Ctrl der.",
            Self::RightShift => "Shift der.",
            Self::RightAlt => "Alt der.",
            Self::RightGui => "GUI der.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Media {
    PlayPause,
    Next,
    Previous,
    Stop,
    Mute,
    VolumeUp,
    VolumeDown,
}
impl Media {
    pub const ALL: [Self; 7] = [
        Self::PlayPause,
        Self::Next,
        Self::Previous,
        Self::Stop,
        Self::Mute,
        Self::VolumeUp,
        Self::VolumeDown,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::PlayPause => "Reproducir / pausar",
            Self::Next => "Siguiente",
            Self::Previous => "Anterior",
            Self::Stop => "Detener",
            Self::Mute => "Silencio",
            Self::VolumeUp => "Volumen +",
            Self::VolumeDown => "Volumen −",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Behavior {
    None,
    Transparent,
    Key {
        key: KeyCode,
    },
    Modifier {
        modifier: Modifier,
    },
    Modified {
        key: KeyCode,
        modifiers: BTreeSet<Modifier>,
    },
    Momentary {
        layer: u32,
    },
    Toggle {
        layer: u32,
    },
    ModTap {
        key: KeyCode,
        modifier: Modifier,
    },
    LayerTap {
        key: KeyCode,
        layer: u32,
    },
    Media {
        action: Media,
    },
}
impl Behavior {
    pub fn target_layer(&self) -> Option<u32> {
        match self {
            Self::Momentary { layer } | Self::Toggle { layer } | Self::LayerTap { layer, .. } => {
                Some(*layer)
            }
            _ => None,
        }
    }
    pub fn valid(&self) -> bool {
        match self {
            Self::Key { key } | Self::ModTap { key, .. } | Self::LayerTap { key, .. } => {
                key.valid()
            }
            Self::Modified { key, modifiers } => key.valid() && !modifiers.is_empty(),
            _ => true,
        }
    }
    pub fn label(&self, layers: &[Layer]) -> String {
        let layer_name = |id: &u32| {
            layers
                .iter()
                .find(|l| l.id == *id)
                .map_or("?".into(), |l| l.name.clone())
        };
        match self {
            Self::None => "—".into(),
            Self::Transparent => "TRNS".into(),
            Self::Key { key } => key.label(),
            Self::Modifier { modifier } => modifier.label().into(),
            Self::Modified { key, modifiers } => format!(
                "{}+{}",
                modifiers
                    .iter()
                    .map(|m| m.label())
                    .collect::<Vec<_>>()
                    .join("+"),
                key.label()
            ),
            Self::Momentary { layer } => format!("MO\n{}", layer_name(layer)),
            Self::Toggle { layer } => format!("TG\n{}", layer_name(layer)),
            Self::ModTap { key, modifier } => format!("{} / {}", key.label(), modifier.label()),
            Self::LayerTap { key, layer } => format!("{} / {}", key.label(), layer_name(layer)),
            Self::Media { action } => action.label().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub bindings: BTreeMap<String, Behavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: u32,
    pub name: String,
    pub layout: Layout,
    pub layers: Vec<Layer>,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct EditError(pub String);

pub fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().count() <= 48 && !name.chars().any(char::is_control)
}

impl Profile {
    pub fn blank(layout: Layout) -> Result<Self, EditError> {
        let bindings = layout
            .keys
            .iter()
            .map(|k| (k.id.clone(), Behavior::None))
            .collect();
        let profile = Self {
            schema_version: 1,
            name: format!("Mi {}", layout.name).chars().take(48).collect(),
            layout,
            layers: vec![Layer {
                id: 0,
                name: "Base".into(),
                bindings,
            }],
        };
        profile.validate()?;
        Ok(profile)
    }
    pub fn validate(&self) -> Result<(), EditError> {
        let fail = |message: &str| Err(EditError(message.into()));
        if self.schema_version != 1 {
            return fail("Versión de perfil no compatible.");
        }
        if !valid_name(&self.name) {
            return fail("El nombre del perfil debe tener entre 1 y 48 caracteres.");
        }
        self.layout
            .validate()
            .map_err(|e| EditError(e.to_string()))?;
        if self.layers.is_empty() || self.layers.len() > MAX_LAYERS {
            return fail("El perfil necesita entre 1 y 32 capas locales.");
        }
        let ids: BTreeSet<_> = self.layers.iter().map(|l| l.id).collect();
        if ids.len() != self.layers.len() {
            return fail("Hay identificadores de capa repetidos.");
        }
        let keys: BTreeSet<_> = self.layout.keys.iter().map(|k| &k.id).collect();
        for layer in &self.layers {
            if !valid_name(&layer.name) {
                return fail("El nombre de cada capa debe tener entre 1 y 48 caracteres.");
            }
            if layer.bindings.keys().collect::<BTreeSet<_>>() != keys {
                return fail("Cada capa debe contener exactamente las teclas del layout.");
            }
            for behavior in layer.bindings.values() {
                if !behavior.valid() {
                    return fail("Asignación de tecla no válida.");
                }
                if behavior.target_layer().is_some_and(|id| !ids.contains(&id)) {
                    return fail("Una tecla hace referencia a una capa que no existe.");
                }
            }
        }
        Ok(())
    }
}
