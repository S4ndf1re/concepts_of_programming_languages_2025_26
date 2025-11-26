use std::{cell::RefCell, iter::zip, rc::Rc};

use crate::{
    AssignmentOperations, AstNode, AstNodeType, Error, FunctionType, InfixOperator,
    InterpreterValue, PrefixOperator, Scope, Stage, StageResult, Symbol, TypeSymbol,
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

    pub fn eval_infix_call(
        &mut self,
        left: &AstNode,
        op: &InfixOperator,
        right: &AstNode,
    ) -> Result<InterpreterValue, Error> {
        let lval = self.eval_node(left)?.unwrap();
        let rval = self.eval_node(right)?.unwrap();

        match op {
            InfixOperator::Plus => lval.add(rval),
            InfixOperator::Minus => lval.subtract(rval),
            InfixOperator::Multiply => lval.multiply(rval),
            InfixOperator::Divide => lval.divide(rval),
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn eval_prefix_call(
        &mut self,
        op: &PrefixOperator,
        right: &AstNode,
    ) -> Result<InterpreterValue, Error> {
        // TODO: Figure out how to negate while respecting type bounds
        todo!()
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
        todo!()
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
            AstNodeType::Bool(b) => IsReturn::NoReturn(InterpreterValue::Strong(Rc::new(
                InterpreterValue::Bool(*b),
            ))),
            AstNodeType::Int(i) => {
                IsReturn::NoReturn(InterpreterValue::Strong(Rc::new(InterpreterValue::Int(*i))))
            }
            AstNodeType::Float(f) => IsReturn::NoReturn(InterpreterValue::Strong(Rc::new(
                InterpreterValue::Float(*f),
            ))),
            AstNodeType::String(s) => IsReturn::NoReturn(InterpreterValue::Strong(Rc::new(
                InterpreterValue::String(s.clone()),
            ))),
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
        fn_name: Symbol,
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

            let scope = self.get_current_scope();
            let mut scope_mut = scope.borrow_mut();
            for (value, (param, type_of)) in zip(evaled_params, &fn_type.params) {
                // TODO: Type check here
                let value = value.unwrap();
                if let InterpreterValue::Empty = value {
                    return Err(Error::ExpectedValue(param.to_owned()));
                }

                scope_mut.declare_variable(param.clone(), value, type_of.clone(), true, false)?;
            }

            let result = self.eval_nodes(&fn_type.execution_body)?;
            match result {
                IsReturn::NoReturn(InterpreterValue::Empty) => Ok(InterpreterValue::Empty),
                IsReturn::Return(v) => Ok(v),
                _ => Err(Error::MissingReturn(fn_name)),
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
                self.call_function(self.entrypoint_fn.clone(), &vec![], main_fn)?;
            } else {
                return Err(Error::WrongType(
                    self.entrypoint_fn.clone(),
                    TypeSymbolType::Function(FunctionType {
                        name: "main".to_string(),
                        params: vec![],
                        return_type: None,
                        execution_body: vec![],
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
