pub mod query_cond;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ecs::{EntityIndex, World};
pub use query_cond::*;

pub mod query_type;
pub use query_type::*;

pub mod query_term;
pub use query_term::*;

use crate::{
    BuiltinStruct, Error, Instantiable, InterpreterValue, Scope, Symbol, TypeSymbolType, WorldObj,
    instantiate_struct_as_t,
};

/// This is not an actual system parameter, but one that can be used when access to the world exists.
/// The main difference is, that instantiate from world takes &self.
/// This is sometimes needed, when calling systems with completely dynamic system params.
/// This can still be used with worlds, but not the same way normal systems can be used (using `SystemFn`s)
pub trait PseudoSystemParameter {
    type State;
    type Item<'w>: PseudoSystemParameter<State = Self::State>;

    fn instantiate_from_world(&self, world: &World) -> Self::State;

    fn get_param<'w>(
        state: &mut Self::State,
        world: &'w World,
        scope: Rc<RefCell<Scope>>,
    ) -> Result<Self::Item<'w>, Error>;
}

#[macro_export]
macro_rules! apply_pseudo_system_param {
    ($world:expr, $query:expr, $scope:expr, $inner:block) => {{
        let mut __state = $query.instantiate_from_world($world);
        $inner;
        ::parser_types::Query::get_param(&mut __state, $world, Rc::clone($scope))
    }};
    ($world:expr, $query:expr, $scope:expr) => {
        apply_pseudo_system_param!($world, $query, $scope, {})
    };
    ($world:expr, $query:expr) => {
        apply_pseudo_system_param!(
            $world,
            $query,
            &std::rc::Rc::new(std::cell::RefCell::new(Scope::default())),
            {}
        )
    };
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Query {
    pub symbol: Symbol,
    pub type_of: QueryType,
}

#[allow(unused)]
#[derive(Debug)]
pub struct QueryItem {
    symbol: Symbol,
    type_of: QueryType,
    pub components: InterpreterValue,
}

impl PseudoSystemParameter for QueryItem {
    type Item<'w> = QueryItem;
    type State = QueryState;

    fn instantiate_from_world(&self, _world: &ecs::World) -> Self::State {
        todo!()
    }

    fn get_param<'w>(
        _state: &mut Self::State,
        _world: &'w ecs::World,
        _scope: Rc<RefCell<Scope>>,
    ) -> Result<Self::Item<'w>, Error> {
        todo!()
    }
}

pub struct QueryState {
    symbol: Symbol,
    type_of: QueryType,
    entities: Vec<EntityIndex>,
}

impl PseudoSystemParameter for Query {
    type State = QueryState;
    type Item<'w> = QueryItem;

    fn instantiate_from_world(&self, world: &ecs::World) -> Self::State {
        let mut entities = Vec::new();
        for entity in world.get_entites() {
            if self.type_of.entity_conforms_condition(entity, world) {
                entities.push(entity);

                // If single, only accept the first found entity. this is possibly nondeterministic.
                if matches!(
                    self.type_of,
                    QueryType::Single {
                        select: _,
                        condition: _
                    }
                ) {
                    break;
                }
            }
        }

        QueryState {
            symbol: self.symbol.clone(),
            type_of: self.type_of.clone(),
            entities,
        }
    }

    fn get_param<'w>(
        state: &mut Self::State,
        world: &'w ecs::World,
        scope: Rc<RefCell<Scope>>,
    ) -> Result<Self::Item<'w>, Error> {
        let mut components = Vec::new();

        if matches!(state.type_of, QueryType::World) {
            let (instance, _world_ref) =
                instantiate_struct_as_t!(scope, "WorldObj" => WorldObj, HashMap::new());

            return Ok(QueryItem {
                symbol: state.symbol.clone(),
                type_of: state.type_of.clone(),
                components: instance,
            });
        }

        let requested_components = state.type_of.get_components();
        for entity in &state.entities {
            let Some(entt) = world.get_entity_mut(*entity) else {
                continue;
            };
            let mut entry_comps = Vec::new();

            for requested_comp in requested_components {
                if requested_comp == "Entity" {
                    entry_comps.push(InterpreterValue::Entity(entt.id()));
                } else if let Some(interpreter_value) =
                    entt.get_component_by_name::<InterpreterValue>(requested_comp)
                {
                    entry_comps.push(interpreter_value.clone());
                }
            }

            components.push(InterpreterValue::List(entry_comps));
        }

        Ok(QueryItem {
            symbol: state.symbol.clone(),
            type_of: state.type_of.clone(),
            components: InterpreterValue::List(components),
        })
    }
}

#[macro_export]
macro_rules! parse_or {
    ( $left:tt || $($rest:tt)+ ) => {
        {
            ::parser_types::QueryCond::Or(Box::new(parse_or!{ $left }), Box::new(parse_or! { $($rest)+ }))
        }
    };
    ( $($single:tt)* ) => {
        parse_and! { $($single)* }
    };
}

#[macro_export]
macro_rules! parse_and {
    ( $left:tt && $($rest:tt)+ ) => {
        {
            ::parser_types::QueryCond::And(Box::new(parse_and!{ $left }), Box::new(parse_and! { $($rest)+ }))
        }
    };
    ( $($single:tt)* ) => {
        parse_primary! { $($single)* }
    };
}

#[macro_export]
macro_rules! parse_primary {
    ( ( $($inner:tt)* ) ) => {
        {
            parse_or! { $($inner)* }
        }
    };
    ( $lit:literal ) => {
        {
            ::parser_types::QueryCond::Component($lit.to_owned())
        }
    };
}

#[macro_export]
/// build a simple query. `not` operator is not supported, hence it is advised to only build simple queries using this technology
macro_rules! build_query {
    (list { $( $names:literal ),* $(,)? $( % $($query:tt)* )? } ) => {
        {
            ::parser_types::Query {
                symbol: "".to_owned(),
                type_of:
                    ::parser_types::QueryType::List {
                        select: ::parser_types::QueryTerm {
                            components: vec![$($names.to_owned()),*]
                        },
                        condition: build_query! { @inner $( parse_or! { $($query)* } )? }
                    }
            }
        }
    };
    (single { $( $names:literal ),* $(,)? $( % $($query:tt)* )? } ) => {
        {
            ::parser_types::Query {
                symbol: "".to_owned(),
                type_of:
                    ::parser_types::QueryType::Single {
                        select: ::parser_types::QueryTerm {
                            components: vec![$($names.to_owned()),*]
                        },
                        condition: build_query! { @inner $( parse_or! { $($query)* } )? }
                    }
            }
        }
    };
    (@inner $e:expr) => { Some($e) };
    (@inner) => { None };
}

#[cfg(test)]
pub mod tests {
    #[test]
    pub fn test_query_build1() {
        let query = build_query!(list { "hallo" % "a" || "b" });
        println!("{query:?}");
    }

    #[test]
    pub fn test_query_build2() {
        let query = build_query!(list { "hallo" % "a" || "b" && "c" });
        println!("{query:?}");
    }

    #[test]
    pub fn test_query_build3() {
        let query = build_query!(list { "hallo" % ("a" || "b") && "c" });
        println!("{query:?}");
    }

    #[test]
    pub fn test_query_build4() {
        let query = build_query!(list { "hallo", "my_name_is" % ("a" || "b") && ("c" || "d") });
        println!("{query:?}");
    }
}
