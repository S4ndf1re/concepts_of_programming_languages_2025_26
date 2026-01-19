use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ecs::{Component, EntityIndex, SingleQuery, World};
use parser_macros::BuiltinComponent;
use parser_types::{
    BuiltinComponent, Error, Instantiable, InterpreterValue, IsReturn, PseudoSystemParameter,
    Scope, TypeSymbolType, apply_pseudo_system_param, build_query, instantiate_component_as_t,
    interpreter_component_instance_as_t,
};
use raylib::{
    color::Color,
    prelude::{RaylibDraw, RaylibDrawHandle},
};

#[derive(Debug, BuiltinComponent)]
pub struct WasdControl {
    pub w: InterpreterValue,
    pub a: InterpreterValue,
    pub s: InterpreterValue,
    pub d: InterpreterValue,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[derive(BuiltinComponent, Debug)]
pub struct Position2d {
    pub x: InterpreterValue,
    pub y: InterpreterValue,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[derive(BuiltinComponent, Debug)]
pub struct Colorable {
    pub r: InterpreterValue,
    pub g: InterpreterValue,
    pub b: InterpreterValue,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[derive(BuiltinComponent, Debug)]
pub struct RectangleShape {
    pub w: InterpreterValue,
    pub h: InterpreterValue,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

pub struct RaylibHandle(raylib::RaylibHandle, raylib::RaylibThread);
impl Component for RaylibHandle {
    fn get_ident(&self) -> String {
        Self::ident()
    }
}

pub fn raylib_init(scope: Rc<RefCell<Scope>>, world: &World) -> Result<IsReturn, Error> {
    let (mut rl, thread) = raylib::init().size(640, 480).title("ECSInject").build();
    rl.set_target_fps(60);

    let mut entity = world.spawn();
    entity.add_component(RaylibHandle(rl, thread));

    let mut entity = world.spawn();
    let (instance, _component) = instantiate_component_as_t!(scope, "WasdControl" => WasdControl,
            vec![("w".to_owned(), Box::new(InterpreterValue::Bool(false))),
                ("a".to_owned(), Box::new(InterpreterValue::Bool(false))),
                ("s".to_owned(), Box::new(InterpreterValue::Bool(false))),
                ("d".to_owned(), Box::new(InterpreterValue::Bool(false)))].into_iter().collect::<HashMap<_, _>>());
    entity.add_component(instance);

    Ok(IsReturn::Return(InterpreterValue::Empty))
}

fn draw_single(
    d: &mut RaylibDrawHandle<'_>,
    world: &World,
    entry: &InterpreterValue,
) -> Result<(), Error> {
    let components = entry.as_list()?;

    let entt: EntityIndex = components[0].deref_value()?.try_into()?;

    let (_, position) = interpreter_component_instance_as_t!(&components[1] => Position2d);
    let (_, rect_shape) = interpreter_component_instance_as_t!(&components[2] => RectangleShape);

    let x: f64 = position.x.clone().try_into()?;
    let y: f64 = position.y.clone().try_into()?;
    let w: f64 = rect_shape.w.clone().try_into()?;
    let h: f64 = rect_shape.h.clone().try_into()?;

    let entt = world.get_entity_mut(entt).unwrap();
    if let Some(color) = entt.get_component_by_name::<InterpreterValue>(&"Colorable".to_string()) {
        let (_, colorable) = interpreter_component_instance_as_t!(color => Colorable);
        let r: i64 = colorable.r.clone().try_into()?;
        let g: i64 = colorable.g.clone().try_into()?;
        let b: i64 = colorable.b.clone().try_into()?;
        let color = Color::new(r as u8, g as u8, b as u8, 255);
        d.draw_rectangle(x as i32, y as i32, w as i32, h as i32, color);
    } else {
        d.draw_rectangle(x as i32, y as i32, w as i32, h as i32, Color::WHITE);
    }

    Ok(())
}

fn raylib_error_helper(
    world: &World,
    mut raylib_handler: SingleQuery<RaylibHandle>,
) -> Result<(), Error> {
    if raylib_handler.components.len() != 1 {
        return Ok(());
    }

    let raylib_handler = raylib_handler
        .components
        .get_mut(0)
        .expect("already checked above");

    if raylib_handler.1.0.window_should_close() {
        // No longer render
        world.despawn(raylib_handler.0);
        world.stop();
        return Ok(());
    }

    let query = build_query!(list {"Entity", "Position2d", "RectangleShape"});
    let applied_args = apply_pseudo_system_param!(world, query)?;

    let applied_components = applied_args.components.as_list()?;

    let mut d = raylib_handler.1.0.begin_drawing(&raylib_handler.1.1);
    d.clear_background(Color::BLACK);

    for entry in applied_components {
        let _ = draw_single(&mut d, world, &entry);
    }

    Ok(())
}

pub fn raylib_system(world: &World, raylib_handler: SingleQuery<RaylibHandle>) {
    if let Err(err) = raylib_error_helper(world, raylib_handler) {
        println!("{err}");
    }
}

fn raylib_input_helper(
    world: &World,
    raylib_handler: SingleQuery<RaylibHandle>,
) -> Result<(), Error> {
    let query = build_query!(list { "WasdControl" });
    let applied = apply_pseudo_system_param!(world, query)?;

    let mut raylib_handler = raylib_handler.components;

    if raylib_handler.len() != 1 {
        return Ok(());
    }

    let raylib_handler = raylib_handler.get_mut(0).ok_or(Error::CantBeEmpty)?;

    if raylib_handler.1.0.window_should_close() {
        return Ok(());
    }

    let handle = &raylib_handler.1.0;
    let w_down = handle.is_key_down(raylib::prelude::KeyboardKey::KEY_W);
    let a_down = handle.is_key_down(raylib::prelude::KeyboardKey::KEY_A);
    let s_down = handle.is_key_down(raylib::prelude::KeyboardKey::KEY_S);
    let d_down = handle.is_key_down(raylib::prelude::KeyboardKey::KEY_D);

    for entry in applied.components.as_list()? {
        let entry = entry.as_list()?;

        let (_, position) = interpreter_component_instance_as_t!(&entry[0] => WasdControl);
        position.w = InterpreterValue::Bool(w_down);
        position.a = InterpreterValue::Bool(a_down);
        position.s = InterpreterValue::Bool(s_down);
        position.d = InterpreterValue::Bool(d_down);
    }

    Ok(())
}

pub fn raylib_input_system(world: &World, raylib_handler: SingleQuery<RaylibHandle>) {
    if let Err(err) = raylib_input_helper(world, raylib_handler) {
        println!("{err}");
    }
}
