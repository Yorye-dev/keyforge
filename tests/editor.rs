use keyforge::{
    domain::devices::{Capability, CapabilityState},
    domain::keymap::*,
    infrastructure::{layouts, profiles},
    services::{capabilities::DeviceCapabilitiesService, editor::KeymapEditor},
};
use std::collections::BTreeSet;

fn editor() -> KeymapEditor {
    KeymapEditor::new(Profile::blank(layouts::demo().unwrap()).unwrap()).unwrap()
}
fn key(editor: &KeymapEditor) -> String {
    editor.profile().layout.keys[0].id.clone()
}
fn letter(c: char) -> Behavior {
    Behavior::Key {
        key: KeyCode::Letter(c),
    }
}

#[test]
fn blank_profile_is_complete_and_does_not_claim_device_mapping() {
    let e = editor();
    assert_eq!(e.layer().bindings.len(), 42);
    assert!(e.layer().bindings.values().all(|b| *b == Behavior::None));
    assert!(!e.dirty());
}
#[test]
fn assignment_undo_redo_and_saved_baseline() {
    let mut e = editor();
    let k = key(&e);
    e.assign(&k, letter('A')).unwrap();
    assert!(e.dirty() && e.changed_keys().contains(&k));
    e.mark_saved();
    assert!(!e.dirty());
    e.assign(&k, letter('B')).unwrap();
    e.undo();
    assert_eq!(e.layer().bindings[&k], letter('A'));
    assert!(!e.dirty());
    e.undo();
    assert_eq!(e.layer().bindings[&k], Behavior::None);
    assert!(e.dirty());
    e.redo();
    assert!(!e.dirty());
    e.assign(&k, letter('C')).unwrap();
    assert!(!e.can_redo());
}
#[test]
fn invalid_assignment_rolls_back_without_history_or_dirty_state() {
    let mut e = editor();
    let k = key(&e);
    for b in [
        Behavior::Momentary { layer: 123 },
        letter('ñ'),
        Behavior::Key {
            key: KeyCode::Function(0),
        },
        Behavior::Modified {
            key: KeyCode::Letter('A'),
            modifiers: BTreeSet::new(),
        },
    ] {
        assert!(e.assign(&k, b).is_err());
        assert!(!e.dirty() && !e.can_undo());
    }
    assert!(e.assign("not-a-key", letter('A')).is_err());
}
#[test]
fn duplicate_layers_are_independent_and_rename_keeps_references() {
    let mut e = editor();
    let k = key(&e);
    e.assign(&k, letter('A')).unwrap();
    e.add_layer(true).unwrap();
    let new = e.active();
    assert_eq!(e.layer().bindings[&k], letter('A'));
    e.assign(&k, letter('B')).unwrap();
    e.select_layer(0).unwrap();
    assert_eq!(e.layer().bindings[&k], letter('A'));
    e.assign(&k, Behavior::Momentary { layer: new }).unwrap();
    e.select_layer(new).unwrap();
    e.rename_layer("Símbolos".into()).unwrap();
    assert!(e.delete_layer().is_err());
    e.select_layer(0).unwrap();
    assert!(
        e.layer().bindings[&k]
            .label(&e.profile().layers)
            .contains("Símbolos")
    );
}
#[test]
fn removing_layer_and_undo_restores_active_layer_and_contents() {
    let mut e = editor();
    let k = key(&e);
    assert!(e.delete_layer().is_err());
    e.add_layer(false).unwrap();
    let id = e.active();
    assert_eq!(e.layer().bindings[&k], Behavior::Transparent);
    e.assign(&k, letter('Z')).unwrap();
    e.delete_layer().unwrap();
    assert_eq!(e.active(), 0);
    e.undo();
    assert_eq!(e.active(), id);
    assert_eq!(e.layer().bindings[&k], letter('Z'));
}
#[test]
fn clipboard_cannot_retarget_a_deleted_layer_to_a_new_layer() {
    let mut e = editor();
    let k = key(&e);
    e.add_layer(false).unwrap();
    let deleted = e.active();
    e.assign(&k, Behavior::Toggle { layer: deleted }).unwrap();
    e.copy(&k).unwrap();
    e.delete_layer().unwrap();
    e.add_layer(false).unwrap();
    assert_ne!(e.active(), deleted);
    assert!(e.paste(&k).is_err());
    assert_eq!(e.layer().bindings[&k], Behavior::Transparent);
}
#[test]
fn copy_paste_between_layers_and_undo_is_one_edit() {
    let mut e = editor();
    let k = key(&e);
    let b = Behavior::ModTap {
        key: KeyCode::Named(NamedKey::Space),
        modifier: Modifier::LeftShift,
    };
    e.assign(&k, b.clone()).unwrap();
    e.copy(&k).unwrap();
    e.add_layer(false).unwrap();
    e.paste(&k).unwrap();
    assert_eq!(e.layer().bindings[&k], b);
    e.undo();
    assert_eq!(e.layer().bindings[&k], Behavior::Transparent);
}
#[test]
fn local_layer_limit_and_bad_names_are_validated() {
    let mut e = editor();
    assert!(e.rename_layer("  ".into()).is_err());
    assert!(e.rename_profile("x".repeat(49)).is_err());
    for _ in 1..MAX_LAYERS {
        e.add_layer(false).unwrap();
    }
    assert!(e.add_layer(false).is_err());
    assert_eq!(e.profile().layers.len(), MAX_LAYERS);
}
#[test]
fn profiles_roundtrip_all_behaviors_and_replace_existing_file() {
    let mut e = editor();
    e.add_layer(false).unwrap();
    let id = e.active();
    e.select_layer(0).unwrap();
    let behaviors = vec![
        letter('A'),
        Behavior::Transparent,
        Behavior::None,
        Behavior::Modifier {
            modifier: Modifier::RightAlt,
        },
        Behavior::Modified {
            key: KeyCode::Digit(1),
            modifiers: BTreeSet::from([Modifier::LeftCtrl, Modifier::LeftShift]),
        },
        Behavior::Momentary { layer: id },
        Behavior::Toggle { layer: id },
        Behavior::ModTap {
            key: KeyCode::Named(NamedKey::Enter),
            modifier: Modifier::LeftCtrl,
        },
        Behavior::LayerTap {
            key: KeyCode::Function(12),
            layer: id,
        },
        Behavior::Media {
            action: Media::VolumeUp,
        },
    ];
    let ids: Vec<_> = e
        .profile()
        .layout
        .keys
        .iter()
        .map(|k| k.id.clone())
        .collect();
    for (id, behavior) in ids.iter().zip(behaviors) {
        e.assign(id, behavior).unwrap();
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    profiles::save(&path, e.profile()).unwrap();
    let loaded = profiles::load(&path).unwrap();
    assert_eq!(loaded.layers, e.profile().layers);
    e.rename_profile("Trabajo".into()).unwrap();
    profiles::save(&path, e.profile()).unwrap();
    assert_eq!(profiles::load(&path).unwrap().name, "Trabajo");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}
#[test]
fn invalid_profile_does_not_overwrite_previous_save() {
    let e = editor();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    profiles::save(&path, e.profile()).unwrap();
    let before = std::fs::read(&path).unwrap();
    let mut invalid = e.profile().clone();
    invalid.layers.clear();
    assert!(profiles::save(&path, &invalid).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(profiles::save(&dir.path().join("missing/file.json"), e.profile()).is_err());
}
#[test]
fn imported_profiles_require_exact_keys_and_existing_layer_targets() {
    let base = editor().profile().clone();
    let mutate: Vec<fn(&mut Profile)> = vec![
        |p| p.schema_version = 5,
        |p| {
            p.layers[0].bindings.pop_first();
        },
        |p| {
            p.layers[0].bindings.insert("extra".into(), Behavior::None);
        },
        |p| p.layers.push(p.layers[0].clone()),
        |p| {
            *p.layers[0].bindings.values_mut().next().unwrap() = Behavior::LayerTap {
                key: KeyCode::Letter('A'),
                layer: 999,
            };
        },
    ];
    for change in mutate {
        let mut p = base.clone();
        change(&mut p);
        assert!(profiles::parse(&serde_json::to_string(&p).unwrap()).is_err());
    }
    assert!(profiles::parse("{}").is_err());
    assert!(profiles::parse(&" ".repeat(8 * 1024 * 1024 + 1)).is_err());
}
#[test]
fn local_capabilities_are_available_even_without_device_metadata() {
    for c in DeviceCapabilitiesService::assess(None).0 {
        if matches!(
            c.capability,
            Capability::LocalKeys | Capability::LocalLayers
        ) {
            assert!(c.actionable);
            assert_eq!(c.state, CapabilityState::Verified);
        } else {
            assert!(!c.actionable);
        }
    }
}
