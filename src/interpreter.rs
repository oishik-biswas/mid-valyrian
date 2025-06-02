use std::collections::HashMap;
use std::io::{ self, Write };
use crate::ast::*;
use crate::error::ValyrianError;

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Vec<Statement>)>,
    debug: bool,
}

impl Interpreter {
    pub fn new(debug: bool) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            debug,
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<(), ValyrianError> {
        if self.debug {
            println!("🐉 AST: {:#?}", program);
        }

        for statement in &program.statements {
            if let Statement::FunctionDeclaration { name, parameters, body } = statement {
                self.functions.insert(name.clone(), (parameters.clone(), body.clone()));
            }
        }

        for statement in &program.statements {
            match statement {
                Statement::MainBlock(statements) => {
                    for stmt in statements {
                        self.execute_statement(stmt)?;
                    }
                }
                Statement::FunctionDeclaration { .. } => {}
                _ => {
                    self.execute_statement(statement)?;
                }
            }
        }

        Ok(())
    }

    fn execute_statement(
        &mut self,
        statement: &Statement
    ) -> Result<Option<ControlFlow>, ValyrianError> {
        if self.debug {
            println!("🏰 Executing: {:?}", statement);
        }

        match statement {
            Statement::Return(expr_opt) => {
                let value = if let Some(expr) = expr_opt {
                    self.evaluate_expression(expr)?
                } else {
                    Value::Void
                };
                return Ok(Some(ControlFlow::Return(value)));
            }
            Statement::VariableDeclaration { name, data_type: _, value } => {
                let val = self.evaluate_expression(value)?;
                self.variables.insert(name.clone(), val);
                Ok(None)
            }
            Statement::Assignment { name, value } => {
                if !self.variables.contains_key(name) {
                    return Err(ValyrianError::UndefinedVariable(name.clone()));
                }
                let val = self.evaluate_expression(value)?;
                self.variables.insert(name.clone(), val);
                Ok(None)
            }
            Statement::FunctionCall { name, arguments } => {
                let _ = self.call_function(name, arguments)?;
                Ok(None)
            }
            Statement::Conditional { condition, then_branch, else_branch } => {
                let condition_value = self.evaluate_expression(condition)?;
                let should_execute = match condition_value {
                    Value::Boolean(b) => b,
                    _ => {
                        return Err(
                            ValyrianError::type_error("boolean", &self.type_name(&condition_value))
                        );
                    }
                };

                let branch = if should_execute { Some(then_branch) } else { else_branch.as_ref() };

                if let Some(stmts) = branch {
                    for stmt in stmts {
                        if let Some(flow) = self.execute_statement(stmt)? {
                            return Ok(Some(flow));
                        }
                    }
                }

                Ok(None)
            }
            Statement::ForLoop { count, body } => {
                for _ in 0..*count {
                    for stmt in body {
                        if let Some(flow) = self.execute_statement(stmt)? {
                            return Ok(Some(flow));
                        }
                    }
                }
                Ok(None)
            }
            Statement::WhileLoop { condition, body } => {
                loop {
                    let condition_value = self.evaluate_expression(condition)?;
                    let should_continue = match condition_value {
                        Value::Boolean(b) => b,
                        _ => {
                            return Err(
                                ValyrianError::type_error(
                                    "boolean",
                                    &self.type_name(&condition_value)
                                )
                            );
                        }
                    };

                    if !should_continue {
                        break;
                    }

                    for stmt in body {
                        if let Some(flow) = self.execute_statement(stmt)? {
                            return Ok(Some(flow));
                        }
                    }
                }
                Ok(None)
            }
            Statement::Speak(expression) => {
                let value = self.evaluate_expression(expression)?;
                println!("{}", value);
                Ok(None)
            }
            Statement::MainBlock(statements) => {
                for stmt in statements {
                    if let Some(flow) = self.execute_statement(stmt)? {
                        return Ok(Some(flow));
                    }
                }
                Ok(None)
            }
            Statement::FunctionDeclaration { .. } => Ok(None),
        }
    }

    fn call_function(
        &mut self,
        name: &str,
        arguments: &[Expression]
    ) -> Result<Value, ValyrianError> {
        let (params, body) = self.functions
            .get(name)
            .ok_or_else(|| ValyrianError::UndefinedFunction(name.to_string()))?
            .clone();

        if arguments.len() != params.len() {
            return Err(ValyrianError::ArgumentMismatch);
        }

        let old_vars: Vec<_> = params
            .iter()
            .map(|p| (p.clone(), self.variables.get(p).cloned()))
            .collect();

        for (param, arg_expr) in params.iter().zip(arguments.iter()) {
            let value = self.evaluate_expression(arg_expr)?;
            self.variables.insert(param.clone(), value);
        }

        for stmt in &body {
            if let Some(ControlFlow::Return(val)) = self.execute_statement(stmt)? {
                for (param, old_val) in old_vars {
                    match old_val {
                        Some(v) => {
                            self.variables.insert(param, v);
                        }
                        None => {
                            self.variables.remove(&param);
                        }
                    }
                }
                return Ok(val);
            }
        }

        for (param, old_val) in old_vars {
            match old_val {
                Some(v) => {
                    self.variables.insert(param, v);
                }
                None => {
                    self.variables.remove(&param);
                }
            }
        }

        Ok(Value::Void)
    }

    fn evaluate_expression(&mut self, expression: &Expression) -> Result<Value, ValyrianError> {
        match expression {
            Expression::Literal(literal) =>
                match literal {
                    Literal::String(s) => Ok(Value::String(s.clone())),
                    Literal::Integer(i) => Ok(Value::Integer(*i)),
                    Literal::Float(f) => Ok(Value::Float(*f)),
                    Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                    Literal::Char(c) => Ok(Value::Char(*c)),
                }
            Expression::Identifier(name) => {
                self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| ValyrianError::UndefinedVariable(name.clone()))
            }
            Expression::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.apply_binary_operator(operator, &left_val, &right_val)
            }
            Expression::Unary { operator, operand } => {
                let operand_val = self.evaluate_expression(operand)?;
                self.apply_unary_operator(operator, &operand_val)
            }
            Expression::Input(_) => {
                print!("🗣️ Speak your words: ");
                io::stdout().flush().map_err(ValyrianError::from)?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).map_err(ValyrianError::from)?;
                Ok(Value::String(input.trim().to_string()))
            }
            Expression::FunctionCall { name, arguments } => { self.call_function(name, arguments) }
        }
    }

    fn apply_binary_operator(
        &self,
        op: &BinaryOperator,
        left: &Value,
        right: &Value
    ) -> Result<Value, ValyrianError> {
        use BinaryOperator::*;
        match (op, left, right) {
            // Arithmetic operators
            (Add, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
            (Add, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
            (Add, Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
            (Add, Value::Integer(l), Value::Float(r)) => Ok(Value::Float((*l as f64) + r)),
            (Add, Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + (*r as f64))),

            (Subtract, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
            (Subtract, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
            (Subtract, Value::Integer(l), Value::Float(r)) => Ok(Value::Float((*l as f64) - r)),
            (Subtract, Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l - (*r as f64))),

            (Multiply, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
            (Multiply, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
            (Multiply, Value::Integer(l), Value::Float(r)) => Ok(Value::Float((*l as f64) * r)),
            (Multiply, Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * (*r as f64))),

            (Divide, _, Value::Integer(r)) if *r == 0 => Err(ValyrianError::DivisionByZero),
            (Divide, _, Value::Float(r)) if *r == 0.0 => Err(ValyrianError::DivisionByZero),
            (Divide, Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l / r)),
            (Divide, Value::Float(l), Value::Float(r)) => Ok(Value::Float(l / r)),
            (Divide, Value::Integer(l), Value::Float(r)) => Ok(Value::Float((*l as f64) / r)),
            (Divide, Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l / (*r as f64))),

            // Boolean operators
            (And, Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l && *r)),
            (Or, Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(*l || *r)),

            // Numeric comparisons - **put these before Equals/NotEquals**
            (Greater, Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l > r)),
            (Less, Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l < r)),
            (GreaterEqual, Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l >= r)),
            (LessEqual, Value::Integer(l), Value::Integer(r)) => Ok(Value::Boolean(l <= r)),

            // General equality checks (catch all variants)
            (Equals, l, r) => Ok(Value::Boolean(l == r)),
            (NotEquals, l, r) => Ok(Value::Boolean(l != r)),

            // Catch-all fallback for unsupported operations
            _ =>
                Err(
                    ValyrianError::invalid_operation(
                        &format!("{:?}", op),
                        &self.type_name(left),
                        &self.type_name(right)
                    )
                ),
        }
    }

    fn apply_unary_operator(
        &self,
        op: &UnaryOperator,
        operand: &Value
    ) -> Result<Value, ValyrianError> {
        match (op, operand) {
            (UnaryOperator::Minus, Value::Integer(n)) => Ok(Value::Integer(-n)),
            (UnaryOperator::Minus, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            _ =>
                Err(
                    ValyrianError::ParseError(
                        format!("Invalid unary operation: {:?} on {:?}", op, operand)
                    )
                ),
        }
    }

    fn type_name(&self, value: &Value) -> String {
        match value {
            Value::Integer(_) => "integer".to_string(),
            Value::Float(_) => "float".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Boolean(_) => "boolean".to_string(),
            Value::Char(_) => "char".to_string(),
            Value::Void => "void".to_string(),
        }
    }
}
