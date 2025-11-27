use std::{cell::RefCell, iter::zip, rc::Rc};

use crate::{
    AssignmentOperations, AstNode, AstNodeType, Error, FunctionExecutionStrategy, FunctionType,
    InfixOperator, InterpreterValue, PrefixOperator, Scope, Stage, StageResult, Symbol, TypeSymbol,
    TypeSymbolType,
};

pub enum IsReturn {
    NoReturn(InterpreterValue),
    Return(InterpreterValue),
}

impl IsReturn {
    pub fn unwrap(self) -> InterpreterValue {
        match self {
            IsReturn::NoReturn(v) => v,
            IsReturn::Return(v) => v,
        }
    }
}

pub struct Environment {
    scope: Rc<RefCell<Scope>>,
    fn_signature: Option<TypeSymbol>,
}

pub struct Interpreter {
    environments: Vec<Environment>,
    ast: Vec<AstNode>,
    entrypoint_fn: Symbol,
}

impl Interpreter {
    pub fn new(entrypoint_fn: Symbol) -> Self {
        Self {
            environments: vec![],
            ast: Vec::new(),
            entrypoint_fn,
        }
    }

    pub fn get_current_scope(&self) -> Rc<RefCell<Scope>> {
        Rc::clone(
            &self
                .environments
                .last()
                .expect("must be present, or init was not called yet")
                .scope,
        )
    }

    pub fn eval_symbol(&mut self, symbol: &Symbol) -> Result<InterpreterValue, Error> {
        let scope = self.get_current_scope();
        let scope = scope.borrow();

        if let Some(val) = scope.resolve_value(symbol) {
            Ok(val)
        } else {
            Err(Error::SymbolNotFound(symbol.clone()))
        }
    }

    pub fn eval_infix_call(
        &mut self,
        left: &AstNode,
        op: &InfixOperator,
        right: &AstNode,
    ) -> Result<InterpreterValue, Error> {
        let lval = self.eval_node(left)?.unwrap();
        let rval = self.eval_node(right)?.unwrap();

        let new_val = match op {
            InfixOperator::Plus => lval.add(rval),
            InfixOperator::Minus => lval.subtract(rval),
            InfixOperator::Multiply => lval.multiply(rval),
            InfixOperator::Divide => lval.divide(rval),
            InfixOperator::Modulo => lval.modulo(rval),
            InfixOperator::And => lval.logical_and(rval),
            InfixOperator::Or => lval.logical_or(rval),
            InfixOperator::Equals => lval.equals(rval),
            InfixOperator::NotEquals => lval.not_equals(rval),
            InfixOperator::LessThan => lval.less_than(rval),
            InfixOperator::LessThanEquals => lval.less_than_equals(rval),
            InfixOperator::GreaterThan => lval.greater_than(rval),
            InfixOperator::GreaterThanEquals => lval.greater_than_equals(rval),
        };

        if let Ok(v) = new_val {
            Ok(v.make_reference_counted()?)
        } else {
            new_val
        }
    }

    pub fn eval_prefix_call(
        &mut self,
        op: &PrefixOperator,
        right: &AstNode,
    ) -> Result<InterpreterValue, Error> {
        let rval = self.eval_node(right)?.unwrap();

        match op {
            PrefixOperator::Not => rval.negate_bool(),
            PrefixOperator::Negate => rval.negate_number(),
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn eval_declaration(
        &mut self,
        new_symbol: &Symbol,
        expression: &AstNode,
        assumed_type: &Option<TypeSymbol>,
    ) -> Result<(), Error> {
        let value = self.eval_node(expression)?.unwrap();
        if let InterpreterValue::Empty = value {
            return Err(Error::CantBeEmpty);
        }

        let scope = self.get_current_scope();
        let mut scope = scope.borrow_mut();

        if let Some(type_of) = assumed_type {
            // TODO: Type checking
            scope.declare_variable(new_symbol.clone(), value, type_of.clone(), false, false)?;
        } else {
            let type_of: Option<TypeSymbol> = value.clone().into();
            if type_of.is_some() {
                scope.declare_variable(
                    new_symbol.clone(),
                    value,
                    type_of.expect("already checked"),
                    false,
                    false,
                )?;
            } else {
                return Err(Error::TypeDeductionError);
            }
        }

        Ok(())
    }

    pub fn eval_assignment_op(
        &mut self,
        recipient: &Symbol,
        op: &AssignmentOperations,
        expression: &AstNode,
    ) -> Result<(), Error> {
        let value = self.eval_node(expression)?.unwrap();
        if let InterpreterValue::Empty = value {
            return Err(Error::CantBeEmpty);
        }

        let scope = self.get_current_scope();
        let mut scope = scope.borrow_mut();
        if let Some(old_value) = scope.resolve_value(recipient) {
            let new_value = match op {
                AssignmentOperations::Add => old_value.add(value)?,
                AssignmentOperations::Subtract => old_value.subtract(value)?,
                AssignmentOperations::Multiply => old_value.multiply(value)?,
                AssignmentOperations::Divide => old_value.divide(value)?,
                AssignmentOperations::Modulo => old_value.modulo(value)?,
                AssignmentOperations::Identity => value,
            };

            scope.set_value(recipient.clone(), new_value.make_reference_counted()?)?;
        }

        Ok(())
    }

    pub fn eval_weak(&mut self, inner: &AstNode) -> Result<InterpreterValue, Error> {
        let val = self.eval_node(inner)?.unwrap();
        if let InterpreterValue::Strong(rc) = val {
            Ok(InterpreterValue::Weak(Rc::downgrade(&rc)))
        } else {
            Err(Error::MainNotFound)
        }
    }

    pub fn eval_node(&mut self, node: &AstNode) -> Result<IsReturn, Error> {
        let evaluated = match &node.type_of {
            // Primitives
            AstNodeType::Bool(b) => {
                IsReturn::NoReturn(InterpreterValue::new_strong(InterpreterValue::Bool(*b)))
            }
            AstNodeType::Int(i) => {
                IsReturn::NoReturn(InterpreterValue::new_strong(InterpreterValue::Int(*i)))
            }
            AstNodeType::Float(f) => {
                IsReturn::NoReturn(InterpreterValue::new_strong(InterpreterValue::Float(*f)))
            }
            AstNodeType::String(s) => IsReturn::NoReturn(InterpreterValue::new_strong(
                InterpreterValue::String(s.clone()),
            )),
            AstNodeType::Symbol(s) => IsReturn::NoReturn(self.eval_symbol(s)?),
            AstNodeType::Weak(inner) => IsReturn::NoReturn(self.eval_weak(inner.as_ref())?),
            // Infix call and prefix calls
            AstNodeType::InfixCall(left, op, right) => {
                IsReturn::NoReturn(self.eval_infix_call(left.as_ref(), op, right.as_ref())?)
            }
            AstNodeType::PrefixCall(prefix, right) => {
                IsReturn::NoReturn(self.eval_prefix_call(prefix, right.as_ref())?)
            }

            // Assignent and declaration
            AstNodeType::Declaration {
                new_symbol,
                expression,
                assumed_type,
            } => {
                self.eval_declaration(new_symbol, expression.as_ref(), assumed_type)?;
                IsReturn::NoReturn(InterpreterValue::Empty)
            }
            AstNodeType::AssignmentOp {
                recipient,
                operation,
                expression,
            } => {
                self.eval_assignment_op(recipient, operation, expression.as_ref())?;
                IsReturn::NoReturn(InterpreterValue::Empty)
            }
            AstNodeType::FunctionCall {
                function_name,
                params,
            } => {
                let fn_type = {
                    // Scoped to free borrowed refcell
                    let scope = self.get_current_scope();
                    let scope = scope.borrow();
                    scope.resolve_type(function_name)
                };

                if let Some(fn_type) = fn_type {
                    let res = self.call_function(function_name, params, fn_type)?;
                    IsReturn::NoReturn(res)
                } else {
                    Err(Error::SymbolNotFound(function_name.clone()))?
                }
            }
            AstNodeType::ReturnStatement { return_value } => {
                IsReturn::Return(self.eval_node(return_value.as_ref())?.unwrap())
            }
            _ => IsReturn::NoReturn(InterpreterValue::Empty),
        };

        Ok(evaluated)
    }

    pub fn eval_nodes(&mut self, nodes: &Vec<Box<AstNode>>) -> Result<IsReturn, Error> {
        for node in nodes {
            let res = self.eval_node(node.as_ref())?;

            // Early exit until function call is reached
            match res {
                IsReturn::NoReturn(_) => (),
                IsReturn::Return(_) => return Ok(res),
            }
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn call_function(
        &mut self,
        fn_name: &Symbol,
        params: &Vec<Box<AstNode>>,
        fn_signature: TypeSymbol,
    ) -> Result<InterpreterValue, Error> {
        if let TypeSymbolType::Function(fn_type) = &fn_signature.type_of {
            // TODO: add error handling
            let mut evaled_params = Vec::new();

            for param in params {
                evaled_params.push(self.eval_node(param.as_ref())?);
            }

            // Create a new stack entry with its own scope
            self.environments.push(Environment {
                scope: Rc::new(RefCell::new(Scope::new_parented(self.get_current_scope()))),
                fn_signature: Some(fn_signature.clone()),
            });

            {
                // scoped to free refcell borrow_mut
                let scope = self.get_current_scope();
                let mut scope_mut = scope.borrow_mut();
                for (value, (param, type_of)) in zip(evaled_params, &fn_type.params) {
                    // TODO: Type check here
                    let value = value.unwrap();
                    if let InterpreterValue::Empty = value {
                        return Err(Error::ExpectedValue(param.to_owned()));
                    }

                    scope_mut.declare_variable(
                        param.clone(),
                        value,
                        type_of.clone(),
                        true,
                        false,
                    )?;
                }
            }

            let result = match &fn_type.execution_body {
                FunctionExecutionStrategy::Interpreted(body) => self.eval_nodes(body)?,
                FunctionExecutionStrategy::Buildin(callback) => callback(self.get_current_scope())?,
            };

            match result {
                IsReturn::NoReturn(InterpreterValue::Empty) => Ok(InterpreterValue::Empty),
                IsReturn::Return(v) => Ok(v),
                _ => Err(Error::MissingReturn(fn_name.clone())),
            }
        } else {
            unimplemented!("error here")
        }
    }
}

impl Stage for Interpreter {
    fn init(&mut self, prev_stage_result: StageResult) -> Result<(), crate::Error> {
        match prev_stage_result {
            StageResult::Stage1(global_scope, ast) => {
                self.ast = ast;
                self.environments.push(Environment {
                    scope: Rc::new(RefCell::new(global_scope)),
                    fn_signature: None,
                });
                Ok(())
            }
            _ => Err(Error::StageError(1, prev_stage_result.into())),
        }
    }

    fn run(mut self) -> Result<StageResult, Error> {
        let main_fn = self
            .get_current_scope()
            .borrow()
            .resolve_value(&self.entrypoint_fn);
        if let Some(main) = main_fn {
            if let InterpreterValue::Function(_) = main {
                let main_fn = self
                    .get_current_scope()
                    .borrow()
                    .resolve_type(&self.entrypoint_fn)
                    .expect("must be present if value is present");
                self.call_function(&self.entrypoint_fn.clone(), &vec![], main_fn)?;
            } else {
                return Err(Error::WrongType(
                    self.entrypoint_fn.clone(),
                    TypeSymbolType::Function(FunctionType {
                        name: "main".to_string(),
                        params: vec![],
                        return_type: None,
                        execution_body: FunctionExecutionStrategy::Interpreted(vec![]),
                    })
                    .to_string(),
                    self.get_current_scope()
                        .borrow()
                        .resolve_type(&self.entrypoint_fn)
                        .expect("must be present if value is presen")
                        .to_string(),
                ));
            }
        } else {
            return Err(Error::MainNotFound);
        }

        Ok(StageResult::Stage2)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Interpreter, Preprocessor, Stage, StageResult, Stages, ast_grammar, run_stages};

    #[test]
    fn test_basic_interpretation() {
        let source = r#"
           fn main() {
            a := 10;
            a += 20;
           }
           "#;

        let ast = ast_grammar::ProgrammParser::new().parse(source).unwrap();

        let stages = vec![
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::Stage0(ast);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn test_basic_interpretation2() {
        let source = r#"
           fn main() {
            a := 10;
            a += 20;
            println(a);
           }
           "#;

        let ast = ast_grammar::ProgrammParser::new().parse(source).unwrap();

        let stages = vec![
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::Stage0(ast);

        let _ = run_stages(stages, state).unwrap();
    }
}
