use std::fs::File;
use std::io::Write;

use ecs::{Component, World};
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::{
    attributes::*,
    cmd::{CommandArg, Format},
    exec, exec_dot,parse,
    printer::{DotPrinter, PrinterContext},
};

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

    let mut g = graph!(id!("id");
         node!("nod"),
         subgraph!("sb";
             edge!(node_id!("a") => subgraph!(;
                node!("n";
                NodeAttributes::color(color_name::blue), NodeAttributes::shape(shape::egg))
            ))
        ),
        edge!(node_id!("a1") => node_id!(esc "a2"))
    );
    let dot = g.print(&mut PrinterContext::default());
    println!("{}", dot);
    let format = Format::Svg;

    let graph_svg = exec_dot(dot.clone(), vec![format.into()]).unwrap();

    let mut file = File::create("test.svg").unwrap();
    file.write_all(&graph_svg).unwrap();


    // let mut world = World::default();

    // world.add_system(my_system);

    // let mut entity = world.spawn();
    // entity.add_component(PositionComponent {
    //     x: 0.0,
    //     y: 0.0,
    //     z: 0.0,
    // });

    // world.run();
}
