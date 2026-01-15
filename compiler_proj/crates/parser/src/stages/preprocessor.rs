use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use ecs::World;
use parser_types::{
    AstNode, AstNodeType, AstTypeDefinition, BeautifyError, ComponentType, Error, ErrorWithRange,
    FunctionExecutionStrategy, FunctionType, InterpreterValue, RegisterType, Scope, ScopeLike,
    StructType, SystemExecutionStrategy, SystemType, TypeSymbol, TypeSymbolType,
};

use crate::{Interpreter, Stage, StageResult, register_buildin};

pub fn run_system(
    sys_name: String,
    interpreter: Rc<RefCell<Interpreter>>,
    source: String,
) -> impl FnMut(&World) {
    let interpreter = Rc::clone(&interpreter);
    move |world: &World| {
        let mut interp = interpreter.borrow_mut();
        let scope = interp.get_current_scope();
        let type_of = scope.borrow_mut().resolve_type(&sys_name).unwrap();
        if let Err(err) = interp.call_system(&sys_name, world, &scope, type_of) {
            err.panic_error(&source);
        }
    }
}

pub struct Preprocessor<'w> {
    ast: Vec<AstNode>,
    world: Option<&'w World>,
    global_scope: Rc<RefCell<Scope>>,
    interpreter: Option<Rc<RefCell<Interpreter>>>,
}

impl<'w> Preprocessor<'w> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            global_scope: Rc::new(RefCell::new(Scope::default())),
            ast: vec![],
            world: None,
            interpreter: None,
        })
    }
}

impl<'w> Stage<'w> for Preprocessor<'w> {
    fn init(&mut self, old_output: StageResult<'w>) -> Result<(), ErrorWithRange> {
        if let StageResult::Parsing(world, ast, interpreter) = old_output {
            self.ast = ast;
            self.world = Some(world);
            self.interpreter = Some(interpreter);
        } else {
            return Err(ErrorWithRange {
                err: Error::StageError(0, old_output.into()),
                range: 0..1,
            });
        }

        let scope = Rc::clone(&self.global_scope);
        scope
            .borrow_mut()
            .declare_type(
                "int".to_owned(),
                TypeSymbol::strong(TypeSymbolType::Int),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange { err, range: 0..1 })?;
        scope
            .borrow_mut()
            .declare_type(
                "float".to_owned(),
                TypeSymbol::strong(TypeSymbolType::Float),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange { err, range: 0..1 })?;

        scope
            .borrow_mut()
            .declare_type(
                "bool".to_owned(),
                TypeSymbol::strong(TypeSymbolType::Bool),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange { err, range: 0..1 })?;

        scope
            .borrow_mut()
            .declare_type(
                "string".to_owned(),
                TypeSymbol::strong(TypeSymbolType::String),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange { err, range: 0..1 })?;

        register_buildin(scope).map_err(|err| ErrorWithRange { err, range: 0..1 })?;

        Ok(())
    }

    fn run(self, _world: &'w World, source: String) -> Result<StageResult<'w>, ErrorWithRange> {
        let mut other_nodes = Vec::new();

        for node in self.ast {
            match node.type_of {
                AstNodeType::TypeDef {
                    typename,
                    typedef,
                    execution_body,
                } => {
                    match typedef {
                        AstTypeDefinition::Function(params, return_type) => {
                            let fun = InterpreterValue::Function(typename.clone());
                            let fun_type =
                                TypeSymbol::strong(TypeSymbolType::Function(FunctionType {
                                    name: typename.clone(),
                                    is_method: false,
                                    params,
                                    return_type: return_type.map(Box::new),
                                    execution_body: FunctionExecutionStrategy::Interpreted(
                                        Rc::new(execution_body),
                                    ),
                                }));
                            // SAFETY: Is always initialized
                            self.global_scope
                                .borrow_mut()
                                .declare_function(
                                    typename,
                                    fun,
                                    fun_type,
                                    false,
                                    true,
                                    node.range.clone(),
                                )
                                .map_err(|err| ErrorWithRange {
                                    err,
                                    range: node.range.clone(),
                                })?;
                        }
                        AstTypeDefinition::Struct(attributes) => {
                            let mut methods = Vec::new();
                            let mut statics = Vec::new();

                            for node in execution_body {
                                if let AstNodeType::TypeDef {
                                    typename: methodname,
                                    typedef: AstTypeDefinition::Function(params, return_type),
                                    execution_body,
                                } = node.type_of
                                {
                                    let is_method = !params.is_empty()
                                        && params[0].1.type_of == TypeSymbolType::SelfType;

                                    let fun_type = FunctionType {
                                        name: methodname.clone(),
                                        is_method,
                                        params,
                                        return_type: return_type.map(Box::new),
                                        execution_body: FunctionExecutionStrategy::Interpreted(
                                            Rc::new(execution_body),
                                        ),
                                    };

                                    if is_method {
                                        methods.push((methodname, fun_type));
                                    } else {
                                        statics.push((methodname, fun_type));
                                    }
                                }
                            }

                            let struct_def =
                                TypeSymbol::strong(TypeSymbolType::Struct(StructType {
                                    name: typename.clone(),
                                    fields: attributes,
                                    methods,
                                    statics,
                                    prefab: None,
                                }));

                            self.global_scope
                                .borrow_mut()
                                .declare_type(typename, struct_def, true, node.range.clone())
                                .map_err(|err| ErrorWithRange {
                                    err,
                                    range: node.range.clone(),
                                })?;
                        }
                        AstTypeDefinition::Component(attributes) => {
                            let struct_def =
                                TypeSymbol::strong(TypeSymbolType::Component(ComponentType {
                                    name: typename.clone(),
                                    fields: attributes,
                                    prefab: None,
                                }));

                            self.global_scope
                                .borrow_mut()
                                .declare_type(typename, struct_def, true, node.range.clone())
                                .map_err(|err| ErrorWithRange {
                                    err,
                                    range: node.range.clone(),
                                })?;
                        }
                        AstTypeDefinition::System(params, queries) => {
                            // first, validate the params, if all params have a matching query
                            if !params.is_empty() && queries.is_none()
                                || params.is_empty()
                                    && queries.is_some()
                                    && !queries.as_ref().expect("already checked").is_empty()
                            {
                                Err(ErrorWithRange {
                                    err: Error::OperationUnsupported {
                                        operation: "system definition".to_owned(),
                                        type_of:
                                            "non matching param list in query and system parameters"
                                                .to_owned(),
                                    },
                                    range: node.range.clone(),
                                })?;
                            }

                            if !params.is_empty()
                                && let Some(queries) = &queries
                            {
                                let mut query_resolver = HashMap::new();
                                for query in queries {
                                    query_resolver.insert(query.symbol.clone(), query.clone());
                                }
                                let mut visited_queries = HashSet::new();

                                for param in &params {
                                    if query_resolver.contains_key(&param.1) {
                                        visited_queries.insert(param.1.clone());
                                    } else {
                                        Err(ErrorWithRange {
                                            err: Error::OperationUnsupported {
                                                operation: "system definition".to_owned(),
                                                type_of: format!(
                                                    "missing query for parameter {}, expected {}",
                                                    param.0, param.1
                                                ),
                                            },
                                            range: node.range.clone(),
                                        })?;
                                    }
                                }

                                if visited_queries.len() < query_resolver.len() {
                                    for query in &query_resolver {
                                        if !visited_queries.contains(query.0) {
                                            Err(ErrorWithRange {
                                                err: Error::OperationUnsupported {
                                                    operation: "system definition".to_owned(),
                                                    type_of: format!(
                                                        "non used query parameter {}",
                                                        query.0
                                                    ),
                                                },
                                                range: node.range.clone(),
                                            })?;
                                        }
                                    }
                                }
                            }

                            let sys = InterpreterValue::System(typename.clone());
                            let sys_type = TypeSymbol::strong(TypeSymbolType::System(SystemType {
                                name: typename.clone(),
                                params,
                                queries,
                                execution_body: SystemExecutionStrategy::Interpreted(
                                    execution_body,
                                ),
                            }));
                            // SAFETY: Is always initialized
                            self.global_scope
                                .borrow_mut()
                                .declare_system(
                                    typename,
                                    sys,
                                    sys_type,
                                    true,
                                    true,
                                    node.range.clone(),
                                )
                                .map_err(|err| ErrorWithRange {
                                    err,
                                    range: node.range.clone(),
                                })?;
                        }
                        _ => (),
                    }
                }
                AstNodeType::Register { schedule_entity } => match schedule_entity {
                    RegisterType::Chain(chain) => {
                        if chain.len() > 1 {
                            Err(ErrorWithRange {
                                err: Error::OperationUnsupported {
                                    operation: "register".to_owned(),
                                    type_of: "other than chain".to_owned(),
                                },
                                range: node.range.clone(),
                            })?
                        } else if chain.is_empty() {
                            Err(ErrorWithRange {
                                err: Error::OperationUnsupported {
                                    operation: "register".to_owned(),
                                    type_of: "at least one register required".to_owned(),
                                },
                                range: node.range.clone(),
                            })?
                        }

                        let sys_reg = chain[0].clone();

                        let _ = self.world.map(|w| {
                            w.add_system(run_system(
                                sys_reg,
                                Rc::clone(self.interpreter.as_ref().expect("must be present")),
                                // TODO: Performace optimiziation
                                source.clone(),
                            ))
                        });
                    }
                    _ => Err(ErrorWithRange {
                        err: Error::OperationUnsupported {
                            operation: "register".to_owned(),
                            type_of: "other than chain".to_owned(),
                        },
                        range: node.range.clone(),
                    })?,
                },
                _ => other_nodes.push(node),
            }
        }
        Ok(StageResult::Preprocessor(
            // NOTE(Jan): make sure to never use global_scope ever again
            self.global_scope
                .take()
                .check_all_types_after_pre_resolve()
                .map_err(|err| ErrorWithRange { err, range: 0..1 })?,
            other_nodes,
        ))
    }
}
