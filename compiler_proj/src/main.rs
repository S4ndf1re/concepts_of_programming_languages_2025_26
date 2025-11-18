use std::marker::PhantomData;

use ecs::{Component, IntoSystem, World};

#[derive(Debug)]
pub struct PositionComponent {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for PositionComponent {}

fn my_system(world: &World) {
    for entity in world.get_entites() {
        let Some(mut entity) = world.get_entity_mut(entity) else {
            continue;
        };

        if let Some(component) = entity.get_component_mut::<PositionComponent>() {
            component.x += 10.0;

            println!("{component:?}")
        }
    }
}

fn main() {
    let mut world = World::default();
    world.add_system(IntoSystem {
        func: my_system,
        marker: PhantomData::<&World>,
    });

    let mut entity = world.spawn();
    entity.add_component(PositionComponent {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });

    world.run();
}
