use std::collections::{HashMap, HashSet};

use crate::{
    AstNode, AstNodeType, AstTypeDefinition, Error, FunctionType, InterpreterValue, Scope, Stage,
    StageResult, StructType, SystemType, TypeSymbol, TypeSymbolType, register_buildin,
};

pub struct Preprocessor {
    ast: Vec<AstNode>,
    global_scope: Scope,
}

impl Preprocessor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            global_scope: Scope::default(),
            ast: vec![],
        })
    }
}

impl Stage for Preprocessor {
    fn init(&mut self, old_output: StageResult) -> Result<(), Error> {
        if let StageResult::Parsing(ast) = old_output {
            self.ast = ast;
        } else {
            return Err(Error::StageError(0, old_output.into()));
        }

        let scope = &mut self.global_scope;
        scope.declare_type(
            "int".to_owned(),
            TypeSymbol::strong(TypeSymbolType::Int),
            false,
        )?;
        scope.declare_type(
            "float".to_owned(),
            TypeSymbol::strong(TypeSymbolType::Float),
            false,
        )?;
        scope.declare_type(
            "bool".to_owned(),
            TypeSymbol::strong(TypeSymbolType::Bool),
            false,
        )?;
        scope.declare_type(
            "string".to_owned(),
            TypeSymbol::strong(TypeSymbolType::String),
            false,
        )?;

        register_buildin(scope)?;

        Ok(())
    }

    fn run(mut self) -> Result<StageResult, Error> {
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
                                    params,
                                    return_type: return_type.map(Box::new),
                                    execution_body: crate::FunctionExecutionStrategy::Interpreted(
                                        execution_body,
                                    ),
                                }));
                            // SAFETY: Is always initialized
                            self.global_scope
                                .declare_function(typename, fun, fun_type, false, true)?;
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
                                    let is_method = params[0].1.type_of == TypeSymbolType::SelfType;

                                    let fun_type = FunctionType {
                                        name: methodname.clone(),
                                        params,
                                        return_type: return_type.map(Box::new),
                                        execution_body:
                                            crate::FunctionExecutionStrategy::Interpreted(
                                                execution_body,
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
                                }));

                            self.global_scope.declare_type(typename, struct_def, true)?;
                        }
                        AstTypeDefinition::System(params, queries) => {
                            // first, validate the params, if all params have a matching query
                            if !params.is_empty() && queries.is_none()
                                || params.is_empty()
                                    && queries.is_some()
                                    && !queries.as_ref().expect("already checked").is_empty()
                            {
                                Err(Error::OperationUnsupported)?;
                            }

                            if !params.is_empty()
                                && let Some(queries) = &queries
                            {
                                let mut query_resolver = HashMap::new();
                                for query in queries {
                                    query_resolver.insert(query.symbol.clone(), query.clone());
                                }
                                let mut visited_quries = HashSet::new();

                                for param in &params {
                                    if query_resolver.contains_key(&param.1) {
                                        visited_quries.insert(param.1.clone());
                                    } else {
                                        Err(Error::OperationUnsupported)?;
                                    }
                                }

                                if visited_quries.len() < query_resolver.len() {
                                    for query in &query_resolver {
                                        if !visited_quries.contains(query.0) {
                                            Err(Error::OperationUnsupported)?;
                                        }
                                    }
                                }
                            }

                            let sys = InterpreterValue::System(typename.clone());
                            let sys_type = TypeSymbol::strong(TypeSymbolType::System(SystemType {
                                name: typename.clone(),
                                params,
                                queries,
                                execution_body: crate::SystemExecutionStrategy::Interpreted(
                                    execution_body,
                                ),
                            }));
                            // SAFETY: Is always initialized
                            self.global_scope
                                .declare_system(typename, sys, sys_type, true, true)?;
                        }
                        _ => (),
                    }
                }
                _ => other_nodes.push(node),
            }
        }
        Ok(StageResult::Preprocessor(
            self.global_scope.check_all_types_after_pre_resolve()?,
            other_nodes,
        ))
    }
}
