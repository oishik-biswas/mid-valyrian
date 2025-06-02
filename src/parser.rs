use pest::Parser;
use pest_derive::Parser;
use crate::ast::*;
use crate::error::ValyrianError;

#[derive(Parser)]
#[grammar = "mid_valyrian.pest"]
pub struct MidValyrianParser;

pub fn parse_program(input: &str) -> Result<Program, ValyrianError> {
    let pairs = MidValyrianParser::parse(Rule::program, input).map_err(|e|
        ValyrianError::ParseError(format!("The Maester failed to decipher your scroll: {}", e))
    )?;

    let mut statements = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner().filter(|p| p.as_rule() == Rule::statement) {
                statements.push(parse_statement(inner)?);
            }
        }
    }

    Ok(Program { statements })
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Statement, ValyrianError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| ValyrianError::ParseError("Empty statement found in the scroll".into()))?;

    match inner.as_rule() {
        Rule::main_block => {
            // inner contains NEWLINEs, WHITESPACE, and one block pair
            let mut body = Vec::new();

            for p in inner.into_inner() {
                match p.as_rule() {
                    Rule::block => {
                        // The block contains statements, comments, and newlines
                        for inner_p in p.into_inner() {
                            if inner_p.as_rule() == Rule::statement {
                                body.push(parse_statement(inner_p)?);
                            }
                            // ignore comments and newlines here
                        }
                    }
                    _ => {
                        // ignore other tokens (e.g. NEWLINE, WHITESPACE)
                    }
                }
            }

            Ok(Statement::MainBlock(body))
        }

        Rule::variable_declaration => {
            let mut inner_rules = inner.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();
            let data_type_str = inner_rules.next().unwrap().as_str();
            let value_expr = inner_rules
                .next()
                .ok_or_else(|| {
                    ValyrianError::ParseError("Missing expression in variable declaration".into())
                })?;
            let value = parse_expression(value_expr)?;
            let data_type = DataType::from_str(data_type_str).ok_or_else(|| {
                ValyrianError::ParseError(format!("Unknown type: {}", data_type_str))
            })?;
            Ok(Statement::VariableDeclaration {
                name,
                data_type,
                value,
            })
        }

        Rule::function_declaration => {
            let mut inner_rules = inner.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();

            // Collect parameters from the appropriate pair (should be first after name)
            let params_pair = inner_rules.next().unwrap();
            let parameters = params_pair
                .into_inner()
                .filter(|p| p.as_rule() == Rule::identifier)
                .map(|p| p.as_str().to_string())
                .collect::<Vec<_>>();

            // The rest are statements (body)
            let body = inner_rules
                .filter(|p| p.as_rule() == Rule::statement)
                .map(parse_statement)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            })
        }

        Rule::function_call => {
            let mut inner_rules = inner.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();
            let arguments = inner_rules
                .filter(|p| p.as_rule() == Rule::expression)
                .map(parse_expression)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Statement::FunctionCall { name, arguments })
        }

        Rule::assignment => {
            let mut inner_rules = inner.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();
            let value = parse_expression(inner_rules.next().unwrap())?;
            Ok(Statement::Assignment { name, value })
        }

        Rule::conditional => {
            let mut inner_rules = inner.into_inner();
            let condition = parse_expression(inner_rules.next().unwrap())?;

            let mut then_branch = Vec::new();
            let mut else_branch = Vec::new();
            let mut in_else = false;

            for stmt in inner_rules {
                match stmt.as_rule() {
                    Rule::ELSE => {
                        in_else = true;
                    }
                    Rule::statement => {
                        let parsed = parse_statement(stmt)?;
                        if in_else {
                            else_branch.push(parsed);
                        } else {
                            then_branch.push(parsed);
                        }
                    }
                    _ => {}
                }
            }

            Ok(Statement::Conditional {
                condition,
                then_branch,
                else_branch: if else_branch.is_empty() {
                    None
                } else {
                    Some(else_branch)
                },
            })
        }

        Rule::for_loop => {
            let mut inner_rules = inner.into_inner();
            let count = inner_rules
                .next()
                .unwrap()
                .as_str()
                .parse::<i64>()
                .map_err(|_| ValyrianError::ParseError("Invalid loop count".into()))?;
            let body = inner_rules
                .filter(|p| p.as_rule() == Rule::statement)
                .map(parse_statement)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Statement::ForLoop { count, body })
        }

        Rule::while_loop => {
            let mut inner_rules = inner.into_inner();
            let condition = parse_expression(inner_rules.next().unwrap())?;
            let body = inner_rules
                .filter(|p| p.as_rule() == Rule::statement)
                .map(parse_statement)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Statement::WhileLoop { condition, body })
        }

        Rule::speak_statement => {
            let expr = inner
                .into_inner()
                .next()
                .ok_or_else(|| ValyrianError::ParseError("speak() is empty".into()))?;
            Ok(Statement::Speak(parse_expression(expr)?))
        }

        _ =>
            Err(
                ValyrianError::ParseError(format!("Unknown statement type: {:?}", inner.as_rule()))
            ),
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, ValyrianError> {
    match pair.as_rule() {
        Rule::expression => parse_expression(pair.into_inner().next().unwrap()),

        Rule::binary_expr => {
            let mut inner = pair.into_inner();
            let mut left = parse_expression(inner.next().unwrap())?;

            while let Some(op) = inner.next() {
                let operator = BinaryOperator::from_str(op.as_str()).ok_or_else(||
                    ValyrianError::ParseError(format!("Unknown binary operator: {}", op.as_str()))
                )?;
                let right = parse_expression(inner.next().unwrap())?;
                left = Expression::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            }

            Ok(left)
        }

        // Rule::unary_expr => {
        //     let mut inner = pair.into_inner();
        //     let op_str = inner.next().unwrap().as_str();
        //     let operator = match op_str {
        //         "-" => UnaryOperator::Minus,
        //         "!" => UnaryOperator::Not,
        //         _ => {
        //             return Err(
        //                 ValyrianError::ParseError(format!("Unknown unary operator: {}", op_str))
        //             );
        //         }
        //     };
        //     let operand = parse_expression(inner.next().unwrap())?;
        //     Ok(Expression::Unary {
        //         operator,
        //         operand: Box::new(operand),
        //     })
        // }

        Rule::unary_expr => {
            let mut inner = pair.into_inner();

            // Peek at first token
            let first = inner.next().unwrap();
            let op = match first.as_rule() {
                Rule::unary_op => {
                    let op_str = first.as_str();
                    let operator = match op_str {
                        "-" => UnaryOperator::Minus,
                        "!" => UnaryOperator::Not,
                        _ => {
                            return Err(
                                ValyrianError::ParseError(
                                    format!("Unknown unary operator: {}", op_str)
                                )
                            );
                        }
                    };
                    let operand = parse_expression(inner.next().unwrap())?;
                    return Ok(Expression::Unary {
                        operator,
                        operand: Box::new(operand),
                    });
                }
                _ => {
                    // no unary operator, so the whole unary_expr is just a primary
                    // parse_expression on the pair directly
                    return parse_expression(first);
                }
            };
        }

        Rule::primary => parse_expression(pair.into_inner().next().unwrap()),

        Rule::string_literal =>
            Ok(Expression::Literal(Literal::String(pair.as_str().trim_matches('"').to_string()))),
        Rule::integer_literal => {
            let value = pair
                .as_str()
                .trim()
                .parse::<i64>()
                .map_err(|_|
                    ValyrianError::ParseError(format!("Invalid integer: {}", pair.as_str()))
                )?;
            Ok(Expression::Literal(Literal::Integer(value)))
        }
        Rule::float_literal => {
            let value = pair
                .as_str()
                .trim()
                .parse::<f64>()
                .map_err(|_|
                    ValyrianError::ParseError(format!("Invalid float: {}", pair.as_str()))
                )?;
            Ok(Expression::Literal(Literal::Float(value)))
        }
        Rule::boolean_literal => {
            let value = match pair.as_str() {
                "aye" => true,
                "nay" => false,
                _ => {
                    return Err(
                        ValyrianError::ParseError(format!("Invalid boolean: {}", pair.as_str()))
                    );
                }
            };
            Ok(Expression::Literal(Literal::Boolean(value)))
        }
        Rule::char_literal => {
            let chars: Vec<char> = pair.as_str().chars().collect();
            if chars.len() < 3 {
                return Err(ValyrianError::ParseError("Invalid character literal".into()));
            }
            Ok(Expression::Literal(Literal::Char(chars[1])))
        }
        Rule::identifier => Ok(Expression::Identifier(pair.as_str().to_string())),

        Rule::input_statement => {
            let name = pair.into_inner().next().unwrap().as_str().to_string();
            Ok(Expression::Input(name))
        }

        _ =>
            Err(
                ValyrianError::ParseError(format!("Unknown expression type: {:?}", pair.as_rule()))
            ),
    }
}
