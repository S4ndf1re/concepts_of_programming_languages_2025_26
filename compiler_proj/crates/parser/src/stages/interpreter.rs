use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    iter::zip,
    rc::Rc,
};

use ecs::World;
use parser_types::{
    AssignmentOperations, AstNode, AstNodeType, Error, ErrorWithRange, FunctionExecutionStrategy,
    FunctionType, InfixOperator, Instantiable, InterpreterValue, IsReturn, MemberAccess,
    MemberAccessType, PrefixOperator, PseudoSystemParameter, Scope, ScopeLike, Symbol,
    SystemExecutionStrategy, TypeSymbol, TypeSymbolType, apply_pseudo_system_param,
};

macro_rules! scoped {
    ($s:ident, $inner:block) => {{
        $s.push_scope();
        let ret = { $inner };
        $s.pop_scope();
        ret
    }};
}

macro_rules! with_scope {
    ($s:ident, $scope:ident, $inner:block) => {{
        $s.environments.push(Environment {
            scope: Rc::clone(&$scope),
        });
        let ret = { $inner };
        $s.pop_scope();
        ret
    }};
}

#[allow(unused)]
macro_rules! with_parent_scope {
    ($s:ident, $parent:ident, $scope:ident $inner:block) => {{
        let old_parent = scope.get_parent_scope();
        scope.set_parent_scope(parent);
        let ret = { $inner };
        scope.set_parent_scope(old_parent);
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

pub struct Environment {
    scope: Rc<RefCell<Scope>>,
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
            ast: vec![],
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

    pub fn eval_infix_call(
        &mut self,
        left: &AstNode,
        op: &InfixOperator,
        right: &AstNode,
        world: &World,
        file: &'static str,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        let lval = self.eval_node(left, world, file)?.unwrap();
        let rval = self.eval_node(right, world, file)?.unwrap();

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
            Ok(v.make_reference_counted().map_err(|e| ErrorWithRange {
                err: e,
                range: left.range.clone(),
                file,
            })?)
        } else {
            let e = new_val.unwrap_err();
            Err(ErrorWithRange {
                err: e,
                range: left.range.clone(),
                file,
            })
        }
    }

    pub fn eval_prefix_call(
        &mut self,
        op: &PrefixOperator,
        right: &AstNode,
        world: &World,
        file: &'static str,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        let rval = self.eval_node(right, world, file)?.unwrap();

        let new_val = match op {
            PrefixOperator::Not => rval.negate_bool(),
            PrefixOperator::Negate => rval.negate_number(),
        };

        if let Ok(v) = new_val {
            Ok(v.make_reference_counted().map_err(|e| ErrorWithRange {
                err: e,
                range: right.range.clone(),
                file,
            })?)
        } else {
            let e = new_val.unwrap_err();
            Err(ErrorWithRange {
                err: e,
                range: right.range.clone(),
                file,
            })
        }
    }

    pub fn eval_entity_despawn(
        &mut self,
        node: &AstNode,
        world: &World,
        calls: &[MemberAccess],
        file: &'static str,
    ) -> Result<(), ErrorWithRange> {
        let (_, _, (_, value)) = self.eval_member_call_helper(node, calls, world, file)?;
        if let InterpreterValue::Entity(entity) =
            value.deref_value().map_err(|err| ErrorWithRange {
                err,
                range: node.range.clone(),
                file,
            })?
        {
            world.despawn(entity);
        }

        Ok(())
    }

    pub fn eval_entity_declaration(
        &mut self,
        node: &AstNode,
        world: &World,
        new_symbol: &Symbol,
        file: &'static str,
    ) -> Result<(), ErrorWithRange> {
        let entity = world.spawn().id();

        let scope = self.get_current_scope();
        let mut scope = scope.borrow_mut();

        scope
            .declare_variable(
                new_symbol.clone(),
                InterpreterValue::Entity(entity),
                TypeSymbol::strong(TypeSymbolType::Entity),
                false,
                false,
                node.range.clone(),
            )
            .map_err(|err| ErrorWithRange {
                err,
                range: node.range.clone(),
                file,
            })?;

        Ok(())
    }

    pub fn eval_declaration(
        &mut self,
        node: &AstNode,
        new_symbol: &Symbol,
        expression: &AstNode,
        assumed_type: &Option<TypeSymbol>,
        world: &World,
        file: &'static str,
    ) -> Result<(), ErrorWithRange> {
        let value = self.eval_node(expression, world, file)?.unwrap();
        if let InterpreterValue::Empty = value {
            return Err(ErrorWithRange {
                err: Error::CantBeEmpty,
                range: expression.range.clone(),
                file,
            });
        }

        if let Some(type_of) = assumed_type {
            // The user provided a type. Check if the types align. if yes, everything is ok, else throw error
            // TODO: Type checking
            let decl_var = {
                let scope = self.get_current_scope();
                let mut scope = scope.borrow_mut();
                scope.declare_variable(
                    new_symbol.clone(),
                    value,
                    type_of.clone(),
                    false,
                    false,
                    node.range.clone(),
                )
            };

            if let Err(e) = decl_var {
                return Err(ErrorWithRange {
                    err: e,
                    range: expression.range.clone(),
                    file,
                });
            }
        } else {
            // Here, the type is not actually provided by the developer, hence, automatic type coercion must occur
            let type_of: Option<TypeSymbol> = value.clone().into();
            if let Some(type_of) = type_of {
                let decl_var = {
                    let scope = self.get_current_scope();
                    let mut scope = scope.borrow_mut();
                    scope.declare_variable(
                        new_symbol.clone(),
                        value,
                        type_of,
                        false,
                        false,
                        node.range.clone(),
                    )
                };
                if let Err(e) = decl_var {
                    return Err(ErrorWithRange {
                        err: e,
                        range: expression.range.clone(),
                        file,
                    });
                }
            } else {
                return Err(ErrorWithRange {
                    err: Error::TypeDeductionError,
                    range: expression.range.clone(),
                    file,
                });
            }
        }

        Ok(())
    }

    pub fn eval_assignment_op(
        &mut self,
        node: &AstNode,
        recipient: &[MemberAccess],
        op: &AssignmentOperations,
        expression: &AstNode,
        world: &World,
        file: &'static str,
    ) -> Result<(), ErrorWithRange> {
        let (mut scope, list_like, (recipient, old_value)) =
            self.eval_member_call_helper(node, recipient, world, file)?;

        // NOTE: When we have an Entity we have a different definition of assignment operations. only assign add and subtract are supported
        if let InterpreterValue::Entity(entity) =
            old_value.deref_value().map_err(|err| ErrorWithRange {
                err,
                range: expression.range.clone(),
                file,
            })?
        {
            if matches!(op, AssignmentOperations::Add) {
                let value = self.eval_node(expression, world, file)?.unwrap();
                // SAFETY: it can be assumed, that this is always reference counted, and possibly never weak
                let value_deref = value.deref_value().map_err(|err| ErrorWithRange {
                    err,
                    range: expression.range.clone(),
                    file,
                })?;
                if matches!(value_deref, InterpreterValue::Component(_, _, _)) {
                    if let Some(mut entt) = world.get_entity_mut(entity) {
                        entt.add_component(value.clone());
                    }
                } else {
                    Err(ErrorWithRange {
                        err: Error::OperationUnsupported {
                            operation: "assignment operation".to_owned(),
                            type_of: "must assign component to entity".to_owned(),
                        },
                        range: expression.range.clone(),
                        file,
                    })?;
                }
            } else if matches!(op, AssignmentOperations::Subtract) {
                let mut expression = expression.clone();
                expression.partial_resolve_symbols = false;
                let value = self.eval_node(&expression, world, file)?.unwrap();
                // SAFETY: it can be assumed, that this is always reference counted, and possibly never weak
                let value_deref = value.deref_value().map_err(|err| ErrorWithRange {
                    err,
                    range: expression.range.clone(),
                    file,
                })?;
                if matches!(value_deref, InterpreterValue::Component(_, _, _))
                    || matches!(value_deref, InterpreterValue::GenericName(_))
                {
                    if let Some(mut entt) = world.get_entity_mut(entity) {
                        entt.remove_component_by_value(value.clone());
                    }
                } else {
                    Err(ErrorWithRange {
                        err: Error::OperationUnsupported {
                            operation: "assignment operation".to_owned(),
                            type_of: "must assign component to entity".to_owned(),
                        },
                        range: expression.range.clone(),
                        file,
                    })?;
                }
            } else {
                Err(ErrorWithRange {
                    err: Error::OperationUnsupported {
                        operation: "assignment operation".to_owned(),
                        type_of: "must assign component to entity".to_owned(),
                    },
                    range: expression.range.clone(),
                    file,
                })?;
            }
        } else {
            let value = self.eval_node(expression, world, file)?.unwrap();
            if let InterpreterValue::Empty = value {
                return Err(ErrorWithRange {
                    err: Error::CantBeEmpty,
                    range: expression.range.clone(),
                    file,
                });
            }

            let new_value = match op {
                AssignmentOperations::Add => old_value.clone() + value,
                AssignmentOperations::Subtract => old_value.clone() - value,
                AssignmentOperations::Multiply => old_value.clone() * value,
                AssignmentOperations::Divide => old_value.clone() / value,
                AssignmentOperations::Modulo => old_value.clone() % value,
                AssignmentOperations::Identity => Ok(value),
            }
            .map_err(|err| ErrorWithRange {
                err,
                range: expression.range.clone(),
                file,
            });

            let new_value = new_value?;
            if let Some((idx, mut list_value)) = list_like {
                *list_value.index_mut(idx).map_err(|err| ErrorWithRange {
                    err,
                    range: node.range.clone(),
                    file,
                })? = new_value;

                scope
                    .set_value(
                        &recipient,
                        list_value
                            .make_reference_counted()
                            .map_err(|err| ErrorWithRange {
                                err,
                                range: expression.range.clone(),
                                file,
                            })?,
                    )
                    .map_err(|err| ErrorWithRange {
                        err,
                        range: expression.range.clone(),
                        file,
                    })?;
            } else {
                scope
                    .set_value(&recipient, new_value)
                    .map_err(|err| ErrorWithRange {
                        err,
                        range: expression.range.clone(),
                        file,
                    })?;
            }
        }

        Ok(())
    }

    pub fn eval_weak(
        &mut self,
        inner: &AstNode,
        world: &World,
        file: &'static str,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        let val = self.eval_node(inner, world, file)?.unwrap();
        if let InterpreterValue::Strong(rc) = &val {
            Ok(InterpreterValue::Weak(Rc::downgrade(rc)))
        } else {
            Err(ErrorWithRange {
                err: Error::MainNotFound,
                range: inner.range.clone(),
                file,
            })
        }
    }

    pub fn eval_branch(
        &mut self,
        cond: &AstNode,
        body: &Vec<Box<AstNode>>,
        else_ifs: &Vec<(Box<AstNode>, Vec<Box<AstNode>>)>,
        else_branch: &Option<Vec<Box<AstNode>>>,
        world: &World,
        file: &'static str,
    ) -> Result<IsReturn, ErrorWithRange> {
        // NOTE: Cannot be return, hence safe to unwrap
        let cond1 = self
            .eval_node(cond, world, file)?
            .unwrap()
            .deref_value()
            .map_err(|err| ErrorWithRange {
                err,
                range: cond.range.clone(),
                file,
            })?;

        let InterpreterValue::Bool(cond1) = cond1 else {
            return Err(ErrorWithRange {
                err: Error::OperationUnsupported {
                    operation: "if condition".to_owned(),
                    type_of: "must be bool".to_owned(),
                },
                range: cond.range.clone(),
                file,
            });
        };

        if cond1 {
            let res = scoped!(self, { self.eval_nodes(body, world, file)? });

            return_on_return!(res);
        } else {
            let mut executed_case = false;

            for elif in else_ifs {
                let cond = self
                    .eval_node(elif.0.as_ref(), world, file)?
                    .unwrap()
                    .deref_value()
                    .map_err(|err| ErrorWithRange {
                        err,
                        range: cond.range.clone(),
                        file,
                    })?;

                let InterpreterValue::Bool(cond) = cond else {
                    return Err(ErrorWithRange {
                        err: Error::OperationUnsupported {
                            operation: "elseif condition".to_owned(),
                            type_of: "must be bool".to_owned(),
                        },
                        range: elif.0.range.clone(),
                        file,
                    });
                };

                if cond {
                    let res = scoped!(self, { self.eval_nodes(&elif.1, world, file)? });

                    return_on_return!(res);
                    executed_case = true;
                    break;
                }
            }

            if !executed_case && else_branch.is_some() {
                let else_branch = else_branch.as_ref().expect("checked");

                let res = scoped!(self, { self.eval_nodes(else_branch, world, file)? });
                return_on_return!(res);
            }
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn eval_while(
        &mut self,
        cond: &AstNode,
        body: &Vec<Box<AstNode>>,
        world: &World,
        file: &'static str,
    ) -> Result<IsReturn, ErrorWithRange> {
        loop {
            let cond1 = self
                .eval_node(cond, world, file)?
                .unwrap()
                .deref_value()
                .map_err(|err| ErrorWithRange {
                    err,
                    range: cond.range.clone(),
                    file,
                })?;

            let InterpreterValue::Bool(cond) = cond1 else {
                return Err(ErrorWithRange {
                    err: Error::OperationUnsupported {
                        operation: "elseif condition".to_owned(),
                        type_of: "must be bool".to_owned(),
                    },
                    range: cond.range.clone(),
                    file,
                });
            };

            if !cond {
                break;
            }

            let res = scoped!(self, { self.eval_nodes(body, world, file)? });
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
        world: &World,
        file: &'static str,
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
                        self.eval_node(init.as_ref(), world, file)?;
                    }
                    _ => {
                        return Err(ErrorWithRange {
                            err: Error::OperationUnsupported {
                                operation: "for loop declaration".to_owned(),
                                type_of: "must be declaration".to_owned(),
                            },
                            range: init.range.clone(),
                            file,
                        });
                    }
                }
            }

            loop {
                if let Some(cond) = cond.as_ref() {
                    let cond1 = self
                        .eval_node(cond, world, file)?
                        .unwrap()
                        .deref_value()
                        .map_err(|err| ErrorWithRange {
                            err,
                            range: cond.range.clone(),
                            file,
                        })?;

                    let InterpreterValue::Bool(cond) = cond1 else {
                        return Err(ErrorWithRange {
                            err: Error::OperationUnsupported {
                                operation: "elseif condition".to_owned(),
                                type_of: "must be bool".to_owned(),
                            },
                            range: cond.range.clone(),
                            file,
                        });
                    };

                    if !cond {
                        break;
                    }
                }

                let res = scoped!(self, { self.eval_nodes(body, world, file)? });
                return_on_return!(res);

                if let Some(step) = step.as_ref() {
                    match &step.type_of {
                        AstNodeType::AssignmentOp {
                            recipient: _,
                            operation: _,
                            expression: _,
                        } => {
                            self.eval_node(step.as_ref(), world, file)?;
                        }
                        _ => {
                            return Err(ErrorWithRange {
                                err: Error::OperationUnsupported {
                                    operation: "for loop assignment".to_owned(),
                                    type_of: "must be assignment".to_owned(),
                                },
                                range: step.range.clone(),
                                file,
                            });
                        }
                    }
                }
            }
        });

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn eval_for_each(
        &mut self,
        node: &AstNode,
        recipient: &Symbol,
        iterable: &AstNode,
        body: &Vec<Box<AstNode>>,
        world: &World,
        file: &'static str,
    ) -> Result<IsReturn, ErrorWithRange> {
        let iterable1 = self.eval_node(iterable, world, file)?.unwrap();

        for entry in iterable1.as_list().map_err(|e| ErrorWithRange {
            err: e,
            range: iterable.range.clone(),
            file,
        })? {
            scoped!(self, {
                let Some(type_of) = entry.clone().into() else {
                    return Err(ErrorWithRange {
                        err: Error::OperationUnsupported {
                            operation: "foreach".to_owned(),
                            type_of: "non list type".to_owned(),
                        },
                        range: iterable.range.clone(),
                        file,
                    });
                };

                self.get_current_scope()
                    .borrow_mut()
                    .declare_variable(
                        recipient.clone(),
                        entry,
                        type_of,
                        true,
                        false,
                        node.range.clone(),
                    )
                    .map_err(|e| ErrorWithRange {
                        err: e,
                        range: iterable.range.clone(),
                        file,
                    })?;

                scoped!(self, {
                    let res = self.eval_nodes(body, world, file)?;
                    return_on_return!(res);
                });
            });
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn eval_list(
        &mut self,
        values: &Vec<Box<AstNode>>,
        world: &World,
        file: &'static str,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        let mut list_elems = Vec::new();

        for value in values {
            list_elems.push(self.eval_node(value.as_ref(), world, file)?.unwrap());
        }

        Ok(InterpreterValue::List(list_elems))
    }

    pub fn eval_map(
        &mut self,
        _values: &Vec<(Box<AstNode>, Box<AstNode>)>,
    ) -> Result<InterpreterValue, Error> {
        unimplemented!(
            "not planned, because of time limitations. It is needed to actually have hashable types for this features. This is currently not implemented"
        )
        // let mut map = HashMap::new();

        // for value in values {
        //         "Implement hashable interpreter value, consisting of only primitives like bool, string and int (float will be unsupported)"
        //     );
        //     // map.insert(self.eval_node(value.0.as_ref())?.unwrap(), self.eval_node(value.1.as_ref())?.unwrap());
        // }

        // Ok(InterpreterValue::Map(map))
    }

    #[allow(clippy::complexity)]
    fn eval_member_call_helper(
        &mut self,
        node: &AstNode,
        calls: &[MemberAccess],
        world: &World,
        file: &'static str,
    ) -> Result<
        (
            InterpreterValue,
            Option<(i64, InterpreterValue)>,
            (Symbol, InterpreterValue),
        ),
        ErrorWithRange,
    > {
        // mutably borrow here, to allow for more complex pointer casting;
        let mut current_file = file;
        let mut last_scope: Option<InterpreterValue> = None;
        let mut current_scope: InterpreterValue =
            InterpreterValue::Module(Rc::clone(&self.get_current_scope()), current_file);
        let mut last_type: Option<TypeSymbol> = None;

        let mut pre_last_res: Option<(i64, InterpreterValue)> = None;
        let mut last_res: Result<(Symbol, InterpreterValue), ErrorWithRange> =
            Err(ErrorWithRange {
                err: Error::OperationUnsupported {
                    operation: "member call".to_owned(),
                    type_of: "must be at least one member call".to_owned(),
                },
                range: node.range.clone(),
                file,
            });

        for call in calls {
            last_scope = Some(current_scope.clone());
            pre_last_res = None;
            let res = match &call.type_of {
                MemberAccessType::Function(params) => {
                    let local_scope = &current_scope;

                    let fn_type = {
                        // Scoped to free borrowed refcell
                        local_scope.resolve_type(&call.member)
                    }
                    .map_err(|err| ErrorWithRange {
                        err,
                        range: call.range.clone(),
                        file,
                    })?;

                    if let TypeSymbolType::Function(fn_typedef) = &fn_type.type_of {
                        let res = if fn_typedef.is_method {
                            self.call_method(
                                &call.member,
                                params,
                                last_res?.1,
                                last_type.unwrap(),
                                &local_scope
                                    .get_outer_scope()
                                    .map_err(|err| ErrorWithRange {
                                        err,
                                        range: call.range.clone(),
                                        file,
                                    })?,
                                fn_type,
                                world,
                                current_file,
                            )
                        } else {
                            self.call_function(
                                &call.member,
                                params,
                                &local_scope
                                    .get_outer_scope()
                                    .map_err(|err| ErrorWithRange {
                                        err,
                                        range: call.range.clone(),
                                        file,
                                    })?,
                                fn_type,
                                world,
                                current_file,
                            )
                        }?;
                        // Set current scope here. it must be checked before every execution
                        current_scope = res.clone();

                        last_type = res.clone().into();
                        res
                    } else {
                        Err(ErrorWithRange {
                            err: Error::SymbolNotFound(call.member.clone()),
                            range: call.range.clone(),
                            file,
                        })?
                    }
                }
                MemberAccessType::Symbol => {
                    let local_scope = &current_scope;

                    if node.partial_resolve_symbols {
                        let res = local_scope.resolve_value(&call.member).map_err(|err| {
                            ErrorWithRange {
                                err,
                                range: call.range.clone(),
                                file,
                            }
                        })?;
                        current_scope = res.clone();
                        last_type = res.clone().into();
                        res
                    } else {
                        // Build generic type
                        let res = InterpreterValue::GenericName(call.member.clone());
                        current_scope = res.clone();
                        last_type = res.clone().into();
                        res
                    }
                }
                MemberAccessType::Struct(fields_to_assign) => {
                    let local_scope = &current_scope;

                    let struct_type = {
                        // Scoped to free borrowed refcell

                        // NOTE: resolve defined type here, not variable type, as this is a defined type
                        local_scope
                            .get_outer_scope()
                            .map_err(|err| ErrorWithRange {
                                err,
                                range: call.range.clone(),
                                file,
                            })?
                            .borrow()
                            .resolve_defined_type(&call.member)
                    };

                    if let Some(struct_type) = struct_type {
                        match &struct_type.type_of {
                            TypeSymbolType::Struct(struct_type_def) => {
                                let fields_of_struct_type = struct_type_def
                                    .get_required_parameters()
                                    .map_err(|err| ErrorWithRange {
                                        err,
                                        range: call.range.clone(),
                                        file,
                                    })?;

                                let mut assigned_fields = HashSet::<&String>::new();
                                let mut field_values = HashMap::new();

                                for (field, value_node) in fields_to_assign {
                                    if fields_of_struct_type.contains_key(field) {
                                        assigned_fields.insert(field);
                                        let value =
                                            self.eval_node(value_node, world, file)?.unwrap();
                                        field_values.insert(field.clone(), Box::new(value));
                                    } else {
                                        todo!("throw error here, as field does not exist")
                                    }
                                }

                                let struct_value = struct_type_def
                                    .instantiate(
                                        Rc::clone(&local_scope.get_outer_scope().map_err(
                                            |err| ErrorWithRange {
                                                err,
                                                range: call.range.clone(),
                                                file,
                                            },
                                        )?),
                                        field_values,
                                    )
                                    .map_err(|err| ErrorWithRange {
                                        err,
                                        range: call.range.clone(),
                                        file,
                                    })?;

                                current_scope = struct_value.clone();
                                last_type = Some(struct_type);
                                struct_value
                            }
                            TypeSymbolType::Component(struct_type_def) => {
                                let fields_of_struct_type = struct_type_def
                                    .get_required_parameters()
                                    .map_err(|err| ErrorWithRange {
                                        err,
                                        range: call.range.clone(),
                                        file,
                                    })?;

                                let mut assigned_fields = HashSet::<&String>::new();
                                let mut field_values = HashMap::new();

                                for (field, value_node) in fields_to_assign {
                                    if fields_of_struct_type.contains_key(field) {
                                        assigned_fields.insert(field);
                                        let value =
                                            self.eval_node(value_node, world, file)?.unwrap();
                                        field_values.insert(field.clone(), Box::new(value));
                                    } else {
                                        todo!("throw error here, as field does not exist")
                                    }
                                }

                                let struct_value = struct_type_def
                                    .instantiate(
                                        Rc::clone(&local_scope.get_outer_scope().map_err(
                                            |err| ErrorWithRange {
                                                err,
                                                range: call.range.clone(),
                                                file,
                                            },
                                        )?),
                                        field_values,
                                    )
                                    .map_err(|err| ErrorWithRange {
                                        err,
                                        range: call.range.clone(),
                                        file,
                                    })?;

                                current_scope = struct_value.clone();
                                last_type = Some(struct_type);
                                struct_value
                            }
                            _ => todo!("error here, cause type is not a struct like"),
                        }
                    } else {
                        Err(ErrorWithRange {
                            err: Error::SymbolNotFound(call.member.clone()),
                            range: call.range.clone(),
                            file,
                        })?
                    }
                }
                MemberAccessType::Index(idx) => {
                    let local_scope = &current_scope;

                    let res =
                        local_scope
                            .resolve_value(&call.member)
                            .map_err(|err| ErrorWithRange {
                                err,
                                range: call.range.clone(),
                                file,
                            })?;

                    pre_last_res = Some((*idx, res.clone()));
                    let res = res
                        .index(*idx)
                        .map_err(|err| ErrorWithRange {
                            err,
                            range: call.range.clone(),
                            file,
                        })?
                        .clone();

                    current_scope = res.clone();
                    last_type = res.clone().into();
                    res
                }
            };
            if let Some(file) = res.get_file() {
                current_file = file;
            }
            last_res = Ok((call.member.clone(), res));
        }

        if let Some(last_scope) = last_scope {
            Ok((last_scope, pre_last_res, last_res?))
        } else {
            Err(ErrorWithRange {
                err: Error::OperationUnsupported {
                    operation: "member call".to_owned(),
                    type_of: "must be at least one call".to_owned(),
                },
                range: node.range.clone(),
                file,
            })
        }
    }

    /// Member call represents any type of member call, a, a.b, a.b().c, a.b(a()).c, etc
    pub fn eval_member_call(
        &mut self,
        node: &AstNode,
        calls: &[MemberAccess],
        world: &World,
        file: &'static str,
    ) -> Result<IsReturn, ErrorWithRange> {
        let (_, _, (_, res)) = self.eval_member_call_helper(node, calls, world, file)?;
        Ok(IsReturn::NoReturn(res))
    }

    pub fn eval_node(
        &mut self,
        node: &AstNode,
        world: &World,
        file: &'static str,
    ) -> Result<IsReturn, ErrorWithRange> {
        let evaluated = match &node.type_of {
            // Primitives
            AstNodeType::Bool(b) => IsReturn::NoReturn(InterpreterValue::Bool(*b)),
            AstNodeType::Int(i) => IsReturn::NoReturn(InterpreterValue::Int(*i)),
            AstNodeType::Float(f) => IsReturn::NoReturn(InterpreterValue::Float(*f)),
            AstNodeType::String(s) => IsReturn::NoReturn(InterpreterValue::String(s.clone())),
            AstNodeType::List(values) => IsReturn::NoReturn(self.eval_list(values, world, file)?),
            AstNodeType::Map(values) => {
                IsReturn::NoReturn(self.eval_map(values).map_err(|e| ErrorWithRange {
                    err: e,
                    range: node.range.clone(),
                    file,
                })?)
            }
            AstNodeType::Weak(inner) => {
                IsReturn::NoReturn(self.eval_weak(inner.as_ref(), world, file)?)
            }
            // Infix call and prefix calls
            AstNodeType::InfixCall(left, op, right) => IsReturn::NoReturn(self.eval_infix_call(
                left.as_ref(),
                op,
                right.as_ref(),
                world,
                file,
            )?),
            AstNodeType::PrefixCall(prefix, right) => {
                IsReturn::NoReturn(self.eval_prefix_call(prefix, right.as_ref(), world, file)?)
            }
            AstNodeType::EntityDef {
                name,
                default_components: _,
            } => {
                self.eval_entity_declaration(node, world, name, file)?;
                IsReturn::NoReturn(InterpreterValue::Empty)
            }
            AstNodeType::EntityDespawn { name } => {
                self.eval_entity_despawn(node, world, name, file)?;
                IsReturn::NoReturn(InterpreterValue::Empty)
            }
            // Assignent and declaration
            AstNodeType::Declaration {
                new_symbol,
                expression,
                assumed_type,
            } => {
                self.eval_declaration(
                    node,
                    new_symbol,
                    expression.as_ref(),
                    assumed_type,
                    world,
                    file,
                )?;
                IsReturn::NoReturn(InterpreterValue::Empty)
            }
            AstNodeType::AssignmentOp {
                recipient,
                operation,
                expression,
            } => {
                self.eval_assignment_op(
                    node,
                    recipient,
                    operation,
                    expression.as_ref(),
                    world,
                    file,
                )?;
                IsReturn::NoReturn(InterpreterValue::Empty)
            }
            // Member call can be anything that is of the form a.b.c.d(a,b).c etc. a() and a are also member calls with length 1
            AstNodeType::MemberCall { calls } => self.eval_member_call(node, calls, world, file)?,
            AstNodeType::ReturnStatement { return_value } => {
                IsReturn::Return(self.eval_node(return_value.as_ref(), world, file)?.unwrap())
            }
            AstNodeType::Branch {
                cond,
                body,
                else_if_branches,
                else_branch,
            } => self.eval_branch(
                cond.as_ref(),
                body,
                else_if_branches,
                else_branch,
                world,
                file,
            )?,
            AstNodeType::While { cond, body } => {
                self.eval_while(cond.as_ref(), body, world, file)?
            }
            AstNodeType::For {
                declaration,
                condition,
                assignment,
                body,
            } => self.eval_for(declaration, condition, assignment, body, world, file)?,
            AstNodeType::ForEach {
                recipient,
                iterable,
                body,
            } => self.eval_for_each(node, recipient, iterable, body, world, file)?,
            _ => Err(Error::OperationUnsupported {
                operation: format!("{:?}", &node.type_of),
                type_of: "".to_owned(),
            })
            .map_err(|err| ErrorWithRange {
                err,
                range: node.range.clone(),
                file,
            })?,
        };

        Ok(evaluated)
    }

    pub fn eval_nodes(
        &mut self,
        nodes: &Vec<Box<AstNode>>,
        world: &World,
        file: &'static str,
    ) -> Result<IsReturn, ErrorWithRange> {
        for node in nodes {
            let res = self.eval_node(node.as_ref(), world, file)?;

            // Early exit until function call is reached
            return_on_return!(res);
        }

        Ok(IsReturn::NoReturn(InterpreterValue::Empty))
    }

    pub fn call_function(
        &mut self,
        fn_name: &Symbol,
        params: &Vec<Box<AstNode>>,
        call_scope: &Rc<RefCell<Scope>>,
        fn_signature: TypeSymbol,
        world: &World,
        file: &'static str,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        if let TypeSymbolType::Function(fn_type) = &fn_signature.type_of {
            let mut evaled_params = Vec::new();

            for param in params {
                evaled_params.push((param, self.eval_node(param.as_ref(), world, file)?));
            }

            let param_scope = {
                let mut param_scope = Scope::new_parented(Rc::clone(call_scope));
                for ((param_node, value), (param, type_of)) in zip(evaled_params, &fn_type.params) {
                    // TODO: Type check here
                    let value = value.unwrap();
                    if let InterpreterValue::Empty = value {
                        return Err(ErrorWithRange {
                            err: Error::ExpectedValue(param.to_owned()),
                            range: 1..2,
                            file,
                        });
                    }

                    param_scope
                        .declare_variable(
                            param.clone(),
                            value,
                            type_of.clone(),
                            true,
                            false,
                            param_node.range.clone(),
                        )
                        .map_err(|e| ErrorWithRange {
                            err: e,
                            range: param_node.range.clone(),
                            file,
                        })?;
                }
                Rc::new(RefCell::new(param_scope))
            };

            let result = with_scope!(self, param_scope, {
                match &fn_type.execution_body {
                    FunctionExecutionStrategy::Interpreted(body) => {
                        self.eval_nodes(body, world, file)?
                    }
                    FunctionExecutionStrategy::Buildin(callback) => {
                        callback(self.get_current_scope(), world).map_err(|e| ErrorWithRange {
                            err: e,
                            range: 1..2,
                            file,
                        })?
                    }
                }
            });

            match result {
                IsReturn::NoReturn(InterpreterValue::Empty) => Ok(InterpreterValue::Empty),
                IsReturn::Return(v) => Ok(v),
                _ => Err(ErrorWithRange {
                    err: Error::MissingReturn(fn_name.clone()),
                    range: 1..2,
                    file,
                }),
            }
        } else {
            unimplemented!("error here")
        }
    }

    #[allow(clippy::complexity)]
    pub fn call_method(
        &mut self,
        fn_name: &Symbol,
        params: &Vec<Box<AstNode>>,
        self_value: InterpreterValue,
        self_type: TypeSymbol,
        call_scope: &Rc<RefCell<Scope>>,
        fn_signature: TypeSymbol,
        world: &World,
        file: &'static str,
    ) -> Result<InterpreterValue, ErrorWithRange> {
        if let TypeSymbolType::Function(fn_type) = &fn_signature.type_of {
            let mut evaled_params = Vec::new();

            for param in params {
                evaled_params.push((param, self.eval_node(param.as_ref(), world, file)?));
            }

            let param_scope = {
                let mut param_scope = Scope::new_parented(Rc::clone(call_scope));
                for ((param_node, value), (param, type_of)) in zip(evaled_params, &fn_type.params) {
                    // TODO: Type check here
                    let value = value.unwrap();
                    if let InterpreterValue::Empty = value {
                        return Err(ErrorWithRange {
                            err: Error::ExpectedValue(param.to_owned()),
                            range: 1..2,
                            file,
                        });
                    }

                    param_scope
                        .declare_variable(
                            param.clone(),
                            value,
                            type_of.clone(),
                            true,
                            false,
                            param_node.range.clone(),
                        )
                        .map_err(|e| ErrorWithRange {
                            err: e,
                            range: param_node.range.clone(),
                            file,
                        })?;
                }

                param_scope
                    .declare_variable("self".to_owned(), self_value, self_type, true, false, 1..2)
                    .map_err(|e| ErrorWithRange {
                        err: e,
                        range: 1..2,
                        file,
                    })?;

                Rc::new(RefCell::new(param_scope))
            };

            let result = with_scope!(self, param_scope, {
                match &fn_type.execution_body {
                    FunctionExecutionStrategy::Interpreted(body) => {
                        self.eval_nodes(body, world, file)?
                    }
                    FunctionExecutionStrategy::Buildin(callback) => {
                        callback(self.get_current_scope(), world).map_err(|e| ErrorWithRange {
                            err: e,
                            range: 1..2,
                            file,
                        })?
                    }
                }
            });

            match result {
                IsReturn::NoReturn(InterpreterValue::Empty) => Ok(InterpreterValue::Empty),
                IsReturn::Return(v) => Ok(v),
                _ => Err(ErrorWithRange {
                    err: Error::MissingReturn(fn_name.clone()),
                    range: 1..2,
                    file,
                }),
            }
        } else {
            unimplemented!("error here")
        }
    }

    pub fn call_system(
        &mut self,
        _sys_name: &Symbol,
        world: &World,
        call_scope: &Rc<RefCell<Scope>>,
        fn_signature: TypeSymbol,
        file: &'static str,
    ) -> Result<(), ErrorWithRange> {
        if let TypeSymbolType::System(system_type) = fn_signature.type_of {
            // assume already validated sytems
            let mut params = HashMap::new();
            let mut queries = HashMap::new();
            let mut query_states = HashMap::new();

            if let Some(sys_queries) = system_type.queries.as_ref() {
                for query in sys_queries {
                    queries.insert(query.symbol.clone(), &query.type_of);

                    let value =
                        apply_pseudo_system_param!(world, query, call_scope).map_err(|err| {
                            ErrorWithRange {
                                err,
                                range: 0..1,
                                file,
                            }
                        })?;
                    query_states.insert(query.symbol.clone(), value);
                }
            }

            for param in &system_type.params {
                params.insert(param.0.clone(), &param.1);
            }

            let mut param_scope = Scope::new_parented(Rc::clone(call_scope));
            for param in params {
                if let Some(item) = query_states.get(param.1) {
                    param_scope
                        .declare_variable(
                            param.0,
                            item.components.clone(), // NOTE(Jan): this could be moved behind a InterpreterValue::Strong, in order to avoid copying
                            TypeSymbol::strong(TypeSymbolType::List(Box::new(TypeSymbol::strong(
                                TypeSymbolType::Any,
                            )))),
                            false,
                            false,
                            0..1,
                        )
                        .map_err(|err| ErrorWithRange {
                            err,
                            range: 0..1,
                            file,
                        })?;
                }
            }
            let param_scope = Rc::new(RefCell::new(param_scope));

            with_scope!(self, param_scope, {
                match system_type.execution_body {
                    SystemExecutionStrategy::Buildin(body) => body(Rc::clone(&param_scope))
                        .map_err(|err| ErrorWithRange {
                            err,
                            range: 0..1,
                            file,
                        }),
                    SystemExecutionStrategy::Interpreted(ast_nodes) => {
                        self.eval_nodes(&ast_nodes, world, file).map(|_| ())
                    }
                }
            })?;

            Ok(())
        } else {
            Err(ErrorWithRange {
                err: Error::OperationUnsupported {
                    operation: "call system".to_owned(),
                    type_of: "must be a system".to_owned(),
                },
                range: 0..1,
                file,
            })
        }
    }

    pub fn initialize_pre_run(&mut self, ast: Vec<AstNode>, global_scope: Rc<RefCell<Scope>>) {
        self.ast = ast;
        self.environments = vec![Environment {
            scope: global_scope,
        }];
    }

    pub fn run(&mut self, world: &World, file: &'static str) -> Result<(), ErrorWithRange> {
        let entrypoint_fn = self.entrypoint_fn.clone();
        let main_fn = self
            .get_current_scope()
            .borrow()
            .resolve_value(&entrypoint_fn)
            .map_err(|err| ErrorWithRange {
                err,
                range: 0..1,
                file,
            })?;

        if let InterpreterValue::Function(_) = main_fn {
            let main_fn = self
                .get_current_scope()
                .borrow()
                .resolve_type(&entrypoint_fn)
                .expect("must be present if value is present");
            let current_scope = self.get_current_scope();
            self.call_function(
                &entrypoint_fn,
                &vec![],
                &current_scope,
                main_fn,
                world,
                file,
            )?;
        } else {
            return Err(ErrorWithRange {
                err: Error::WrongType(
                    entrypoint_fn.clone(),
                    TypeSymbolType::Function(FunctionType {
                        name: "main".to_string(),
                        is_method: false,
                        params: vec![],
                        return_type: None,
                        execution_body: FunctionExecutionStrategy::Interpreted(Rc::new(vec![])),
                    })
                    .to_string(),
                    self.get_current_scope()
                        .borrow()
                        .resolve_type(&entrypoint_fn)
                        .expect("must be present if value is presen")
                        .to_string(),
                ),
                range: 0..1,
                file,
            });
        }

        Ok(())
    }
}
