use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::Path,
    rc::Rc,
};

use ecs::World;
use parser_types::{
    AstNode, AstNodeType, AstTypeDefinition, BeautifyError, BuiltinComponent, BuiltinStruct,
    ComponentType, Error, ErrorWithRange, FunctionExecutionStrategy, FunctionType,
    InterpreterValue, RegisterType, Scope, ScopeLike, StructType, SystemExecutionStrategy,
    SystemType, TypeSymbol, TypeSymbolType,
};

use crate::{
    BuiltinFunctionDescription, Interpreter, SourceLoader, parse_content,
    register_buildin_component, register_buildin_function, register_buildin_functions,
    register_buildin_struct, register_buildin_structs_and_comps,
};

pub fn run_system(
    sys_name: String,
    interpreter: Rc<RefCell<Interpreter>>,
    source: &'static str,
) -> impl FnMut(&World) {
    let interpreter = Rc::clone(&interpreter);
    move |world: &World| {
        let mut interp = interpreter.borrow_mut();
        let scope = interp.get_current_scope();
        let type_of = scope.borrow_mut().resolve_type(&sys_name).unwrap();
        if let Err(err) = interp.call_system(&sys_name, world, &scope, type_of, source) {
            err.panic_error(source);
        }
    }
}

pub struct Preprocessor<'s, T> {
    global_scope: Rc<RefCell<Scope>>,
    source_loader: &'s T,
}

impl<'s, T> Preprocessor<'s, T>
where
    T: SourceLoader,
{
    pub fn new(source_loader: &'s T) -> Self {
        Self {
            global_scope: Rc::new(RefCell::new(Scope::default())),
            source_loader,
        }
    }

    pub fn register_builtin_struct<S: BuiltinStruct>(&self, strct: S) -> Result<(), Error> {
        register_buildin_struct(Rc::clone(&self.global_scope), strct)
    }

    pub fn register_builtin_component<C: BuiltinComponent>(&self, strct: C) -> Result<(), Error> {
        register_buildin_component(Rc::clone(&self.global_scope), strct)
    }

    pub fn register_builtin_function(&self, func: BuiltinFunctionDescription) -> Result<(), Error> {
        register_buildin_function(Rc::clone(&self.global_scope), func)
    }

    fn register_builtins(&self, file: &'static str) -> Result<(), ErrorWithRange> {
        self.global_scope
            .borrow_mut()
            .declare_type(
                "int".to_owned(),
                TypeSymbol::strong(TypeSymbolType::Int),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange {
                err,
                range: 0..1,
                file,
            })?;
        self.global_scope
            .borrow_mut()
            .declare_type(
                "float".to_owned(),
                TypeSymbol::strong(TypeSymbolType::Float),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange {
                err,
                range: 0..1,
                file,
            })?;

        self.global_scope
            .borrow_mut()
            .declare_type(
                "bool".to_owned(),
                TypeSymbol::strong(TypeSymbolType::Bool),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange {
                err,
                range: 0..1,
                file,
            })?;

        self.global_scope
            .borrow_mut()
            .declare_type(
                "string".to_owned(),
                TypeSymbol::strong(TypeSymbolType::String),
                false,
                0..1,
            )
            .map_err(|err| ErrorWithRange {
                err,
                range: 0..1,
                file,
            })?;

        register_buildin_functions(Rc::clone(&self.global_scope)).map_err(|err| {
            ErrorWithRange {
                err,
                range: 0..1,
                file,
            }
        })?;
        register_buildin_structs_and_comps(Rc::clone(&self.global_scope)).map_err(|err| {
            ErrorWithRange {
                err,
                range: 0..1,
                file,
            }
        })?;

        Ok(())
    }

    pub fn preprocess(
        &self,
        interpreter: Rc<RefCell<Interpreter>>,
        world: &World,
    ) -> Result<(Vec<AstNode>, Rc<RefCell<Scope>>), ErrorWithRange> {
        let empty_static = self.source_loader.empty_string();
        let main_file = self
            .source_loader
            .load_main_file()
            .map_err(|err| ErrorWithRange {
                err,
                range: 0..1,
                file: empty_static,
            })?;

        self.register_builtins(main_file)?;

        let rest = Self::eval_scope_and_file(
            Rc::clone(&self.global_scope),
            self.source_loader,
            main_file,
            world,
            Rc::clone(&interpreter),
        )?;

        Ok((rest, Rc::clone(&self.global_scope)))
    }

    fn eval_scope_and_file(
        scope: Rc<RefCell<Scope>>,
        loader: &'s T,
        file: &'static str,
        world: &World,
        interpreter: Rc<RefCell<Interpreter>>,
    ) -> Result<Vec<AstNode>, ErrorWithRange> {
        let mut other_nodes = Vec::new();
        let nodes = parse_content(file);

        for node in nodes {
            match node.type_of {
                AstNodeType::Import(module, alias) => {
                    let content = loader
                        .load_file(Path::new(&format!("{module}.eij")))
                        .map_err(|err| ErrorWithRange {
                            err,
                            range: node.range.clone(),
                            file,
                        })?;

                    // NOTE(Jan): must not be parented, as its a self contained module
                    let local_scope = Rc::new(RefCell::new(Scope::default()));
                    let _ = Self::eval_scope_and_file(
                        Rc::clone(&local_scope),
                        loader,
                        content,
                        world,
                        Rc::clone(&interpreter),
                    )?;

                    if let Some(alias) = alias {
                        scope
                            .borrow_mut()
                            .declare_variable(
                                alias,
                                InterpreterValue::Module(local_scope, content),
                                TypeSymbol::strong(TypeSymbolType::Any),
                                false,
                                false,
                                node.range.clone(),
                            )
                            .map_err(|err| ErrorWithRange {
                                err,
                                range: node.range.clone(),
                                file,
                            })?;
                    } else {
                        scope
                            .borrow_mut()
                            .declare_variable(
                                module,
                                InterpreterValue::Module(local_scope, content),
                                TypeSymbol::strong(TypeSymbolType::Any),
                                false,
                                false,
                                node.range.clone(),
                            )
                            .map_err(|err| ErrorWithRange {
                                err,
                                range: node.range.clone(),
                                file,
                            })?;
                    }
                }
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
                            scope
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
                                    file,
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

                            scope
                                .borrow_mut()
                                .declare_type(typename, struct_def, true, node.range.clone())
                                .map_err(|err| ErrorWithRange {
                                    err,
                                    range: node.range.clone(),
                                    file,
                                })?;
                        }
                        AstTypeDefinition::Component(attributes) => {
                            let struct_def =
                                TypeSymbol::strong(TypeSymbolType::Component(ComponentType {
                                    name: typename.clone(),
                                    fields: attributes,
                                    prefab: None,
                                }));

                            scope
                                .borrow_mut()
                                .declare_type(typename, struct_def, true, node.range.clone())
                                .map_err(|err| ErrorWithRange {
                                    err,
                                    range: node.range.clone(),
                                    file,
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
                                    file,
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
                                            file,
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
                                                file,
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
                            scope
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
                                    file,
                                })?;
                        }
                        _ => (),
                    }
                }
                AstNodeType::Register { schedule_entity } => match schedule_entity {
                    RegisterType::Chain(chain) => {
                        if chain.is_empty() {
                            Err(ErrorWithRange {
                                err: Error::OperationUnsupported {
                                    operation: "register".to_owned(),
                                    type_of: "at least one register required".to_owned(),
                                },
                                range: node.range.clone(),
                                file,
                            })?
                        }

                        for sys in &chain {
                            let sys_reg = sys.clone();

                            world.add_system(run_system(
                                sys_reg,
                                Rc::clone(&interpreter),
                                // TODO: Performace optimiziation
                                file,
                            ));
                        }
                    }
                    _ => Err(ErrorWithRange {
                        err: Error::OperationUnsupported {
                            operation: "register".to_owned(),
                            type_of: "other than chain".to_owned(),
                        },
                        range: node.range.clone(),
                        file,
                    })?,
                },
                _ => other_nodes.push(node),
            }
        }
        Ok(other_nodes)
    }
}
