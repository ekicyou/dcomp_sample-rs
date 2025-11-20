use bevy_ecs::prelude::*;
use windows_numerics::Vector2;
use wintf::ecs::visual_resource_management_system;
use wintf::ecs::Visual;
use wintf::ecs::{GraphicsCore, SurfaceGraphics, VisualGraphics};

#[test]
fn test_visual_component_definition() {
    let mut world = World::new();

    // Test Default
    let entity = world.spawn(Visual::default()).id();

    let visual = world.get::<Visual>(entity).unwrap();

    // Check default values
    assert_eq!(visual.is_visible, true);
    assert_eq!(visual.opacity, 1.0);
    assert_eq!(visual.transform_origin.X, 0.0);
    assert_eq!(visual.transform_origin.Y, 0.0);
}

#[test]
fn test_visual_component_properties() {
    let visual = Visual {
        is_visible: false,
        opacity: 0.5,
        transform_origin: Vector2::new(10.0, 20.0),
        size: Vector2::new(100.0, 100.0),
    };

    assert_eq!(visual.is_visible, false);
    assert_eq!(visual.opacity, 0.5);
    assert_eq!(visual.transform_origin.X, 10.0);
    assert_eq!(visual.transform_origin.Y, 20.0);
}

#[test]
fn test_visual_resource_creation() {
    let mut world = World::new();

    // Setup GraphicsCore
    // Note: This requires a valid Windows environment with DComp support.
    // If running in CI without GPU, this might fail.
    // But local environment seems to have it.
    let graphics = GraphicsCore::new().expect("Failed to create GraphicsCore");
    world.insert_resource(graphics);

    // Setup Schedule
    let mut schedule = Schedule::default();
    schedule.add_systems(visual_resource_management_system);

    // Spawn entity with Visual
    let entity = world.spawn(Visual::default()).id();

    // Run schedule
    schedule.run(&mut world);

    // Check if VisualGraphics and SurfaceGraphics are added
    assert!(world.get::<VisualGraphics>(entity).is_some());
    assert!(world.get::<SurfaceGraphics>(entity).is_some());
}
