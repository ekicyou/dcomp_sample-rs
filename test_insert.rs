use bevy_ecs::prelude::*;

#[derive(Component, Default)]
struct TestComp(i32);

fn main() {
    let mut world = World::new();
    
    let entity = world.spawn(TestComp(10)).id();
    println!("初期値: {:?}", world.get::<TestComp>(entity).unwrap().0);
    
    // 既に存在するコンポーネントにinsert
    world.entity_mut(entity).insert(TestComp(20));
    println!("insert後: {:?}", world.get::<TestComp>(entity).unwrap().0);
    
    // もう一度insert
    world.entity_mut(entity).insert(TestComp(30));
    println!("2回目insert後: {:?}", world.get::<TestComp>(entity).unwrap().0);
}
