use std::{cell::RefCell, iter::zip, rc::Rc};

use crate::{
    AssignmentOperations, AstNode, AstNodeType, Error, ErrorWithRange, FunctionExecutionStrategy, FunctionType, InfixOperator, InterpreterValue, MemberAccess, MemberAccessType, PrefixOperator, Scope, Stage, StageResult, Symbol, TypeSymbol, TypeSymbolType
};

fn type_of_i_value(a:InterpreterValue) -> &'static str {
    match a{
        InterpreterValue::Int(_) => "int",
        InterpreterValue::Float(_) => "float",
        InterpreterValue::String(_) => "String",
        InterpreterValue::Bool(_) => "bool",
        InterpreterValue::List(interpreter_values) => todo!(),
        InterpreterValue::Map(hash_map) => todo!(),
        InterpreterValue::Struct(_, hash_map) => todo!(),
        InterpreterValue::Option(interpreter_value) => todo!(),
        InterpreterValue::Result(interpreter_value) => todo!(),
        InterpreterValue::Function(_) => todo!(),
        InterpreterValue::Weak(weak) => todo!(),
        InterpreterValue::Strong(interpreter_value) => todo!(),
        InterpreterValue::Entity(index) => todo!(),
        InterpreterValue::Component(_, hash_map) => todo!(),
        InterpreterValue::System(_) => todo!(),
        InterpreterValue::Module(ref_cell) => todo!(),
        InterpreterValue::Empty => todo!(),
    }
}

macro_rules! scoped {
    ($s:ident, $inner:block) => {{
        $s.push_scope();
        let ret = { $inner };
        $s.pop_scope();
        ret
    }};
}

macro_rules! return_on_return {
    ($res:expr) => {
        match $res {
            IsReturn::Return(_) => return Ok($res),
            IsReturn::NoReturn(_) => (),
        }
    };
}

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
    // TODO: Add is_in_loop flag for checking returns, alternatively handle this only in preprocessing
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

    pub fn push_scope(&mut self) {
        self.environments.push(Environment {
            scope: Rc::new(RefCell::new(Scope::new_parented(self.get_current_scope()))),
        });
    }

    pub fn pop_scope(&mut self) {
        self.environments.pop();
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
    ) -> Result<InterpreterValue, ErrorWithRange> {
        let lval = self.eval_node(left)?.unwrap();
        let rval = self.eval_node(right)?.unwrap();

        let new_val = match op {
            InfixOperator::Plus => lval + rval,
            InfixOperator::Minus => lval - rval,
            InfixOperator::Multiply => lval * rval,
            InfixOperator::Divide => lval / rval,
            InfixOperator::Modulo => lval % rval,
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
            Ok(v.make_reference_counted().map_err(|e| ErrorWithRange{
                err:e,
                range: left.range.clone()
            })?)
        } else {
            let e = new_val.unwrap_err(); 
            Err(ErrorWithRange {
                err: e,
                range: left.range.clone(),
            })
        }
    }

    pub fn eval_prefix_call(
        &mut self,
        op: &PrefixOperator,
        right: &AstNode,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        let rval = self.eval_node(right)?.unwrap();

        let new_val = match op {
            PrefixOperator::Not => rval.negate_bool(),
            PrefixOperator::Negate => rval.negate_number(),
        };

        if let Ok(v) = new_val {
            Ok(v.make_reference_counted().map_err(|e| ErrorWithRange{
                err:e,
                range: right.range.clone()
            })?)
        } else {
            let e = new_val.unwrap_err(); 
            Err(ErrorWithRange {
                err: e,
                range: right.range.clone(),
            })
        }
    }

    pub fn eval_declaration(
        &mut self,
        new_symbol: &Symbol,
        expression: &AstNode,
        assumed_type: &Option<TypeSymbol>,
    ) -> Result<(), ErrorWithRange> {
        let value = self.eval_node(expression)?.unwrap();
        if let InterpreterValue::Empty = value {
            return Err(ErrorWithRange{
                err :Error::CantBeEmpty,
                range: expression.range.clone()
        });
        }

        let scope = self.get_current_scope();
        let mut scope = scope.borrow_mut();

        if let Some(type_of) = assumed_type {
            // TODO: Type checking
            let decl_var = scope.declare_variable(new_symbol.clone(), value, type_of.clone(), false, false);
            if let Err(e) = decl_var{
                return Err(ErrorWithRange{
                    err :e,
                    range: expression.range.clone()
                });
            }
        } else {
            let type_of: Option<TypeSymbol> = value.clone().into();
            if type_of.is_some() {
                let decl_var = scope.declare_variable(
                    new_symbol.clone(),
                    value,
                    type_of.expect("already checked"),
                    false,
                    false,
                );
                if let Err(e) = decl_var{
                return Err(ErrorWithRange{
                    err :e,
                    range: expression.range.clone()
                });
            }
            } else {
                return Err(ErrorWithRange{
                    err :Error::TypeDeductionError,
                    range: expression.range.clone()
                });
            }
        }

        Ok(())
    }

    pub fn eval_assignment_op(
        &mut self,
        recipient: &Symbol,
        op: &AssignmentOperations,
        expression: &AstNode,
    ) -> Result<(), ErrorWithRange> {
        let value = self.eval_node(expression)?.unwrap();
        if let InterpreterValue::Empty = value {
            return Err(ErrorWithRange{
                err: Error::CantBeEmpty,
                range: expression.range.clone()
            });
        }

        let scope = self.get_current_scope();
        let mut scope = scope.borrow_mut();
        if let Some(old_value) = scope.resolve_value(recipient) {
            let new_value = match op {
                AssignmentOperations::Add => old_value + value,
                AssignmentOperations::Subtract => old_value - value,
                AssignmentOperations::Multiply => old_value * value,
                AssignmentOperations::Divide => old_value / value,
                AssignmentOperations::Modulo => old_value % value,
                AssignmentOperations::Identity => Ok(value),
            }
            .map_err(|e| ErrorWithRange{
                err: e,
                range:expression.range.clone()
            })?;

            scope.set_value(recipient.clone(), new_value.make_reference_counted()
            .map_err(|e| ErrorWithRange{
                err: e,
                range:expression.range.clone()
            })?).map_err(|e| ErrorWithRange{
                err: e,
                range:expression.range.clone()
            })?;
        } else {
            return Err(ErrorWithRange{
                err: Error::SymbolNotFound(recipient.clone()),
                range:expression.range.clone()
                });
        }

        Ok(())
    }

    pub fn eval_weak(&mut self, inner: &AstNode) -> Result<InterpreterValue, ErrorWithRange> {
        let val = self.eval_node(inner)?.unwrap();
        if let InterpreterValue::Strong(rc) = val {
            Ok(InterpreterValue::Weak(Rc::downgrade(&rc)))
        } else {
            return Err(ErrorWithRange { 
                err: Error::MainNotFound, 
                range: inner.range.clone() 
            });
        }
    }

    pub fn eval_branch(
        &mut self,
        cond: &AstNode,
        body: &Vec<Box<AstNode>>,
        else_ifs: &Vec<(Box<AstNode>, Vec<Box<AstNode>>)>,
        else_branch: &Option<Vec<Box<AstNode>>>,
    ) -> Result<IsReturn, ErrorWithRange> {
        // NOTE: Cannot be return, hence safe to unwrap
        let cond1 = self.eval_node(cond)?.unwrap();

        let InterpreterValue::Bool(cond1) = cond1 else {
            return Err(ErrorWithRange { 
                err: Error::OperationUnsupported{a:"==".to_string(), b:"==".to_string(), c:"==".to_string()}, 
                range: cond.range.clone() 
            });
        };

        if cond1 {
            let res = scoped!(self, { self.eval_nodes(body)? });

            return_on_return!(res);
        } else {
            let mut executed_case = false;

            for elif in else_ifs {
                let cond = self.eval_node(elif.0.as_ref())?.unwrap();
                let InterpreterValue::Bool(cond) = cond else {
                    return Err(ErrorWithRange{ 
                        err: Error::OperationUnsupported{a:"==".to_string(), b:"==".to_string(), c:"==".to_string()}, 
                        range: elif.0.range.clone()});
                };

                if cond {
                    let res = scoped!(self, { self.eval_nodes(&elif.1)? });

                    return_on_return!(res);
                    executed_case = true;
                    break;
                }
            }

            if !executed_case && else_branch.is_some() {
                let else_branch = else_branch.as_ref().expect("checked");

                let res = scoped!(self, { self.eval_nodes(else_branch)? });
                return_on_return!(res);
            }
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn eval_while(
        &mut self,
        cond: &AstNode,
        body: &Vec<Box<AstNode>>,
    ) -> Result<IsReturn, ErrorWithRange> {
        loop {
            let cond1 = self.eval_node(cond)?.unwrap();

            if !cond1.as_bool().map_err(|e| ErrorWithRange{
                err: e,
                range:cond.range.clone()})? {
                break;
            }

            let res = scoped!(self, { self.eval_nodes(body)? });
            return_on_return!(res);
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn eval_for(
        &mut self,
        init: &Option<Box<AstNode>>,
        cond: &Option<Box<AstNode>>,
        step: &Option<Box<AstNode>>,
        body: &Vec<Box<AstNode>>,
    ) -> Result<IsReturn, ErrorWithRange> {
        scoped!(self, {
            // Init condition
            if let Some(init) = init.as_ref() {
                match &init.type_of {
                    AstNodeType::Declaration {
                        new_symbol: _,
                        expression: _,
                        assumed_type: _,
                    } => {
                        self.eval_node(init.as_ref())?;
                    }
                    _ => return Err(ErrorWithRange{
                        err: Error::OperationUnsupported{a:"==".to_string(), b:"==".to_string(), c:"==".to_string()},
                        range: init.range.clone()}),
                }
            }

            loop {
                if let Some(cond) = cond.as_ref() {
                    let cond1 = self.eval_node(cond.as_ref())?.unwrap();

                    if !cond1.as_bool().map_err(|e| ErrorWithRange{
                        err: e,
                        range:cond.range.clone()})? {
                        break;
                    }
                }

                let res = scoped!(self, { self.eval_nodes(body)? });
                return_on_return!(res);

                if let Some(step) = step.as_ref() {
                    match &step.type_of {
                        AstNodeType::AssignmentOp {
                            recipient: _,
                            operation: _,
                            expression: _,
                        } => {
                            self.eval_node(step.as_ref())?;
                        }
                        _ => return Err(ErrorWithRange{
                        err: Error::OperationUnsupported{a:"==".to_string(), b:"==".to_string(), c:"==".to_string()},
                        range: step.range.clone()}),
                    }
                }
            }
        });

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn eval_for_each(
        &mut self,
        recipient: &Symbol,
        iterable: &AstNode,
        body: &Vec<Box<AstNode>>,
    ) -> Result<IsReturn, ErrorWithRange> {
        let iterable1 = self.eval_node(iterable)?.unwrap();

        for entry in iterable1.as_list().map_err(|e| ErrorWithRange{
                        err: e,
                        range:iterable.range.clone()})? {
            scoped!(self, {
                let Some(type_of) = entry.clone().into() else {
                    return Err(ErrorWithRange{
                        err: Error::OperationUnsupported{a:"==".to_string(), b:"==".to_string(), c:"==".to_string()},
                        range: iterable.range.clone()});
                };

                self.get_current_scope().borrow_mut().declare_variable(
                    recipient.clone(),
                    entry,
                    type_of,
                    true,
                    false,
                ).map_err(|e| ErrorWithRange{
                        err: e,
                        range:iterable.range.clone()})?;

                scoped!(self, {
                    let res = self.eval_nodes(body)?;
                    return_on_return!(res);
                });
            });
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn eval_list(&mut self, values: &Vec<Box<AstNode>>) -> Result<InterpreterValue, ErrorWithRange> {
        let mut list_elems = Vec::new();

        for value in values {
            list_elems.push(self.eval_node(value.as_ref())?.unwrap());
        }

        Ok(InterpreterValue::List(list_elems))
    }

    pub fn eval_map(
        &mut self,
        _values: &Vec<(Box<AstNode>, Box<AstNode>)>,
    ) -> Result<InterpreterValue, Error> {
        todo!()
        // let mut map = HashMap::new();

        // for value in values {
        //         "Implement hashable interpreter value, consisting of only primitives like bool, string and int (float will be unsupported)"
        //     );
        //     // map.insert(self.eval_node(value.0.as_ref())?.unwrap(), self.eval_node(value.1.as_ref())?.unwrap());
        // }

        // Ok(InterpreterValue::Map(map))
    }

    /// Member call represents any type of member call, a, a.b, a.b().c, a.b(a()).c, etc
    pub fn eval_member_call(&mut self, calls: &[MemberAccess]) -> Result<IsReturn, ErrorWithRange> {
        assert!(calls.len() == 1, "currently, only one call is supported");

        let call = &calls[0];
        println!("{:?}", &call.type_of);

        let res = match &call.type_of {
            MemberAccessType::Function(params) => {
                let fn_type = {
                    // Scoped to free borrowed refcell
                    self.get_current_scope().borrow().resolve_type(&call.member)
                };

                if let Some(fn_type) = fn_type {
                    let res = self.call_function(&call.member, params, fn_type)?;
                    IsReturn::NoReturn(res)
                } else {
                    Err(ErrorWithRange{
                        err: Error::SymbolNotFound(call.member.clone()),
                        range: call.range.clone()
                    })?
                }
            }
            MemberAccessType::Symbol => IsReturn::NoReturn(self.eval_symbol(&call.member).map_err(
                |e|ErrorWithRange{
                    err: e,
                    range: call.range.clone()
                })?),
            _ => Err(ErrorWithRange{
                    err: Error::OperationUnsupported{a:"==".to_string(), b:"==".to_string(), c:"==".to_string()},
                    range: call.range.clone()
                })?,
        };

        Ok(res)
    }

    pub fn eval_node(&mut self, node: &AstNode) -> Result<IsReturn, ErrorWithRange> {
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
            AstNodeType::List(values) => IsReturn::NoReturn(self.eval_list(values)?),
            AstNodeType::Map(values) => IsReturn::NoReturn(self.eval_map(values).map_err(
                |e|ErrorWithRange{
                    err: e,
                    range: node.range.clone()
                })?),
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
            // Member call can be anything that is of the form a.b.c.d(a,b).c etc. a() and a are also member calls with length 1
            AstNodeType::MemberCall { calls } => self.eval_member_call(calls)?,
            // AstNodeType::MemberCall { calls } => self.eval_member_call(calls).map_err(
            //     |e|ErrorWithRange{
            //         err: e,
            //         range: node.range.clone()
            //     })?,
            AstNodeType::ReturnStatement { return_value } => {
                IsReturn::Return(self.eval_node(return_value.as_ref())?.unwrap())
            }
            AstNodeType::Branch {
                cond,
                body,
                else_if_branches,
                else_branch,
            } => self.eval_branch(cond.as_ref(), body, else_if_branches, else_branch)?,
            AstNodeType::While { cond, body } => self.eval_while(cond.as_ref(), body)?,
            AstNodeType::For {
                declaration,
                condition,
                assignment,
                body,
            } => self.eval_for(declaration, condition, assignment, body)?,
            AstNodeType::ForEach {
                recipient,
                iterable,
                body,
            } => self.eval_for_each(recipient, iterable, body)?,
            _ => IsReturn::NoReturn(InterpreterValue::Empty),
        };

        Ok(evaluated)
    }

    pub fn eval_nodes(&mut self, nodes: &Vec<Box<AstNode>>) -> Result<IsReturn, ErrorWithRange> {
        for node in nodes {
            let res = self.eval_node(node.as_ref())?;

            // Early exit until function call is reached
            return_on_return!(res);
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn call_function(
        &mut self,
        fn_name: &Symbol,
        params: &Vec<Box<AstNode>>,
        fn_signature: TypeSymbol,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        if let TypeSymbolType::Function(fn_type) = &fn_signature.type_of {
            // TODO: add error handling
            let mut evaled_params = Vec::new();

            for param in params {
                evaled_params.push(self.eval_node(param.as_ref())?);
            }

            // Create a new stack entry with its own scope
            let result = scoped!(self, {
                // scoped to free refcell borrow_mut
                {
                    let scope = self.get_current_scope();
                    let mut scope_mut = scope.borrow_mut();
                    for (value, (param, type_of)) in zip(evaled_params, &fn_type.params) {
                        // TODO: Type check here
                        let value = value.unwrap();
                        if let InterpreterValue::Empty = value {
                            return Err(ErrorWithRange{
                                err: Error::ExpectedValue(param.to_owned()),
                                range: 1..2
                        });
                        }

                        scope_mut.declare_variable(
                            param.clone(),
                            value,
                            type_of.clone(),
                            true,
                            false,
                        ).map_err(|e|ErrorWithRange{
                            err: e,
                            range: 1..2
                        })?;
                    }
                }
                match &fn_type.execution_body {
                    FunctionExecutionStrategy::Interpreted(body) => self.eval_nodes(body)?,
                    FunctionExecutionStrategy::Buildin(callback) => {
                        callback(self.get_current_scope()).map_err(|e|ErrorWithRange{
                            err: e,
                            range: 1..2
                        })?
                    }
                }
            });

            match result {
                IsReturn::NoReturn(InterpreterValue::Empty) => Ok(InterpreterValue::Empty),
                IsReturn::Return(v) => Ok(v),
                _ => Err(ErrorWithRange{
                    err: Error::MissingReturn(fn_name.clone()),
                    range: 1..2
                }),
            }
        } else {
            unimplemented!("error here")
        }
    }
}

impl Stage for Interpreter {
    fn init(&mut self, prev_stage_result: StageResult) -> Result<(), crate::Error> {
        match prev_stage_result {
            StageResult::Preprocessor(global_scope, ast) => {
                self.ast = ast;

                self.environments = vec![Environment {
                    scope: Rc::new(RefCell::new(global_scope)),
                }];

                Ok(())
            }
            _ => Err(Error::StageError(1, prev_stage_result.into())),
        }
    }

    fn run(mut self) -> Result<StageResult, ErrorWithRange> {
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
                return Err(ErrorWithRange{
                    err: Error::WrongType(
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
                    ),
                    range: 1..1
                });
            }
        } else {
            return Err(ErrorWithRange{
                err: Error::MainNotFound,
                range: 1..1
            });
        }

        Ok(StageResult::Interpretation)
    }
}

#[cfg(test)]
mod tests {
    use crate::{BeautifyError, Interpreter, Parser, Preprocessor, StageResult, Stages, ast_grammar, run_stages};

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

        let state = StageResult::Parsing(ast);

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

        let state = StageResult::Parsing(ast);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn function_definition_and_returning() {
        let source = r#"

           fn test(a: int): int {
            return a + 10;
           }

           fn main() {
            a := 10;
            println(test(a));
           }
           "#
        .to_owned();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn loop1() {
        let source = r#"
           fn main() {
            a := 10;
            while (a > 0) {
                a -= 1;
            }
           }
           "#
        .to_owned();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn loop2() {
        let source = r#"
           fn main() {
                for (a := 10; a > 0; a -= 1) {
                }
           }
           "#
        .to_owned();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let _ = run_stages(stages, state).unwrap();
    }

    #[test]
    fn loop3() {
        let source = r#"
           fn main() {
                res := 0;
                for (a in [10, 20, 30, 40]) {
                    res += a;
                }
                assert(res == true);
           }
           "#
        .to_owned();

        let source_safe = source.clone();

        let stages = vec![
            Stages::Parser(Parser::default()),
            Stages::Preprocessor(Preprocessor::new().unwrap()),
            Stages::Interpreter(Interpreter::new("main".to_string())),
        ];

        let state = StageResult::PreParse(source);

        let result = run_stages(stages, state);
        if let Err(occured_error) = result {
            occured_error.print_error(&source_safe);
                        panic!("{}", occured_error)
        }
    }
}
