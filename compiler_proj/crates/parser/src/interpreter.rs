use crate::{
    ActualTypedValue, AstNode, AstNodeType, AstTypeDefinition, Error, FunctionType, StructType,
    TypeSymbol, TypeSymbolType, typed::Scope,
};

pub enum StageResult {
    Stage0(Vec<AstNode>),
    Stage1(Scope, Vec<AstNode>),
}

impl From<StageResult> for usize {
    fn from(value: StageResult) -> Self {
        match value {
            StageResult::Stage0(_) => 0,
            StageResult::Stage1(_, _) => 1,
        }
    }
}

pub trait Stage {
    fn init(&mut self, prev_stage_result: StageResult) -> Result<(), Error>;
    fn run(self) -> Result<StageResult, Error>;
}

pub struct Preprocessor {
    ast: Vec<AstNode>,
    global_scope: Scope,
}

impl Preprocessor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            global_scope: Scope::new(),
            ast: vec![],
        })
    }
}

impl Stage for Preprocessor {
    fn init(&mut self, old_output: StageResult) -> Result<(), Error> {
        if let StageResult::Stage0(ast) = old_output {
            self.ast = ast;
        } else {
            return Err(Error::StageError(1, old_output.into()));
        }

        let scope = &mut self.global_scope;
        scope.declare_type("int".to_owned(), TypeSymbol::strong(TypeSymbolType::Int), false)?;
        scope.declare_type("float".to_owned(), TypeSymbol::strong(TypeSymbolType::Float), false)?;
        scope.declare_type("bool".to_owned(), TypeSymbol::strong(TypeSymbolType::Bool), false)?;
        scope.declare_type("string".to_owned(), TypeSymbol::strong(TypeSymbolType::String), false)?;

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
                            let fun = ActualTypedValue::Function;
                            let fun_type =
                                TypeSymbol::strong(TypeSymbolType::Function(FunctionType {
                                    name: typename.clone(),
                                    params,
                                    return_type: return_type.map(|r| Box::new(r)),
                                    execution_body,
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
                                        return_type: return_type.map(|r| Box::new(r)),
                                        execution_body,
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
                        _ => (),
                    }
                }
                _ => other_nodes.push(node),
            }
        }
        Ok(StageResult::Stage1(
            self.global_scope.check_all_types_after_pre_resolve()?,
            other_nodes,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{BeautifyError, Preprocessor, Stage, StageResult, ast_grammar};

    #[test]
    fn test_preprocessing() {
        let source = r#"fn abc(a: B, c: int, d: float) {}
                                struct B {
                                    a: float,
                                }
                        "#;
        let expr = ast_grammar::ProgrammParser::new().parse(source);

        if let Err(expr) = expr {
            expr.panic_error(source);
        } else {
            let expr = expr.unwrap();

            let s0 = StageResult::Stage0(expr);

            let mut processor = Preprocessor::new().unwrap();
            processor.init(s0).unwrap();
            processor.run().unwrap();
        }
    }
}
