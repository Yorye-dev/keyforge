use keyforge::{domain::layout::Geometry, infrastructure::layouts};

#[test]
fn corne_has_two_halves_and_six_thumbs() {
    let layout = layouts::demo().expect("bundled layout");
    assert_eq!(layout.keys.len(), 42);
    for half in &layout.halves {
        assert_eq!(layout.keys.iter().filter(|k| &k.half == half).count(), 21);
    }
    assert_eq!(layout.keys.iter().filter(|k| k.thumb).count(), 6);
    assert_eq!(layout.displays.len(), 2);
    let roundtrip = serde_json::to_string(&layout).unwrap();
    assert_eq!(layouts::parse(&roundtrip).unwrap().id, layout.id);
}

#[test]
fn rejects_invalid_layouts() {
    let original = layouts::demo().unwrap();
    let mutations: Vec<fn(&mut keyforge::domain::layout::Layout)> = vec![
        |l| l.schema_version = 999,
        |l| l.key_count = 1,
        |l| l.keys[1].id = l.keys[0].id.clone(),
        |l| l.keys[1].matrix = l.keys[0].matrix,
        |l| l.keys[0].matrix = [8, 0],
        |l| l.keys[0].half = "unknown".into(),
        |l| l.keys[0].geometry.width = 0.0,
        |l| l.keys[0].geometry.x = f32::NAN,
        |l| l.displays[0].resolution = Some([0, 240]),
        |l| l.displays[1].id = l.displays[0].id.clone(),
    ];
    for mutate in mutations {
        let mut layout = original.clone();
        mutate(&mut layout);
        assert!(layout.validate().is_err());
    }
    assert!(layouts::parse("{}").is_err());
    assert!(layouts::parse(&" ".repeat(2 * 1024 * 1024 + 1)).is_err());
    assert!(layouts::load(std::path::Path::new("nonexistent-layout.json")).is_err());
}

#[test]
fn geometry_rotates_around_center() {
    let g = Geometry {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 1.0,
        rotation: 90.0,
    };
    let p = g.corners()[0];
    assert!((p[0] - 1.5).abs() < 0.0001);
    assert!((p[1] + 0.5).abs() < 0.0001);
}

#[test]
fn accepts_non_split_single_key_layout() {
    let mut layout = layouts::demo().unwrap();
    layout.keys.truncate(1);
    layout.key_count = 1;
    layout.halves = vec!["left".into()];
    layout.displays.clear();
    layout.validate().unwrap();
    let (min, max) = layout.bounds();
    assert!(max[0] > min[0] && max[1] > min[1]);
}

#[test]
fn disk_layout_matches_embedded_demo() {
    let layout =
        layouts::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("layouts/corne.json"))
            .unwrap();
    assert_eq!(layout.id, layouts::demo().unwrap().id);
}
