use ecs::{Component, NamedComponent, NamedSingleQuery, SingleQuery, World};

#[derive(Debug, Default)]
pub struct PositionComponent {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for PositionComponent {
    fn get_ident(&self) -> String {
        Self::ident()
    }
}

impl NamedComponent for PositionComponent {
    const NAME: &'static str = "compiler_proj::PositionComponent";
}

fn my_system(world: &World) {
    for entity in world.get_entites() {
        let Some(mut entity) = world.get_entity_mut(entity) else {
            continue;
        };

        if let Some(component) = entity.get_component_mut::<PositionComponent>() {
            component.x += 10.0;
            component.y += 5.0;
            component.z -= 5.0;

            println!("{component:?}")
        }
    }
}

fn my_system2(positions: NamedSingleQuery<PositionComponent>) {
    for comp in positions.components {
            comp.x += 10.0;
            comp.y += 5.0;
            comp.z -= 5.0;

            println!("{comp:?}")
    }
}

fn main() {
    let mut world = World::default();

    world.add_system(my_system2);

    let mut entity = world.spawn();
    entity.add_component(PositionComponent {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });

    world.run();
}
