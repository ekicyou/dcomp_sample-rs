//! Parser module for Pasta DSL.
//!
//! This module provides parsing functionality for the Pasta DSL using pest (PEG parser).
//! The parser converts DSL source code into an Abstract Syntax Tree (AST) representation.

mod ast;

pub use ast::*;

use crate::error::PastaError;
use pest::iterators::Pair;
use pest::Parser as PestParser;
use pest_derive::Parser;
use std::path::Path;

#[derive(Parser)]
#[grammar = "parser/pasta.pest"]
struct PastaParser;

/// Parse a Pasta script file
pub fn parse_file(path: &Path) -> Result<PastaFile, PastaError> {
    let source = std::fs::read_to_string(path)?;
    parse_str(&source, path.to_string_lossy().as_ref())
}

/// Parse a Pasta script from a string
pub fn parse_str(source: &str, filename: &str) -> Result<PastaFile, PastaError> {
    let mut pairs = PastaParser::parse(Rule::file, source)
        .map_err(|e| PastaError::PestError(format!("Parse error in {}: {}", filename, e)))?;

    let file_pair = pairs.next().unwrap(); // file rule always produces one pair
    let mut labels = Vec::new();

    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::global_label => {
                labels.push(parse_global_label(pair)?);
            }
            Rule::EOI => {} // End of input, ignore
            _ => {}
        }
    }

    let span = Span::new(1, 1, source.lines().count(), source.len());

    Ok(PastaFile {
        path: Path::new(filename).to_path_buf(),
        labels,
        span,
    })
}

fn parse_global_label(pair: Pair<Rule>) -> Result<LabelDef, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut attributes = Vec::new();
    let mut local_labels = Vec::new();
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label_name => {
                name = inner_pair.as_str().to_string();
                // Validate reserved label pattern: __*__ is reserved for system use
                if name.starts_with("__") && name.ends_with("__") {
                    return Err(PastaError::ParseError {
                        file: "<input>".to_string(),
                        line: start.0,
                        column: start.1,
                        message: format!(
                            "Label name '{}' is reserved for system use. \
                            Label names starting and ending with '__' are not allowed. \
                            Consider using '{}' or '_{}_' instead.",
                            name,
                            name.trim_start_matches('_').trim_end_matches('_'),
                            name.trim_matches('_')
                        ),
                    });
                }
            }
            Rule::attribute_line => {
                attributes.push(parse_attribute(inner_pair)?);
            }
            Rule::local_label => {
                local_labels.push(parse_local_label(inner_pair)?);
            }
            Rule::rune_block => {
                statements.push(parse_rune_block(inner_pair)?);
            }
            Rule::statement => {
                if let Some(stmt) = parse_statement(inner_pair)? {
                    statements.push(stmt);
                }
            }
            _ => {}
        }
    }

    Ok(LabelDef {
        name,
        scope: LabelScope::Global,
        attributes,
        local_labels,
        statements,
        span,
    })
}

fn parse_local_label(pair: Pair<Rule>) -> Result<LabelDef, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut attributes = Vec::new();
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::label_name => {
                name = inner_pair.as_str().to_string();
                // Validate reserved label pattern: __*__ is reserved for system use
                if name.starts_with("__") && name.ends_with("__") {
                    return Err(PastaError::ParseError {
                        file: "<input>".to_string(),
                        line: start.0,
                        column: start.1,
                        message: format!(
                            "Label name '{}' is reserved for system use. \
                            Label names starting and ending with '__' are not allowed. \
                            Consider using '{}' or '_{}_' instead.",
                            name,
                            name.trim_start_matches('_').trim_end_matches('_'),
                            name.trim_matches('_')
                        ),
                    });
                }
            }
            Rule::attribute_line => {
                attributes.push(parse_attribute(inner_pair)?);
            }
            Rule::rune_block => {
                statements.push(parse_rune_block(inner_pair)?);
            }
            Rule::statement => {
                if let Some(stmt) = parse_statement(inner_pair)? {
                    statements.push(stmt);
                }
            }
            _ => {}
        }
    }

    Ok(LabelDef {
        name,
        scope: LabelScope::Local,
        attributes,
        local_labels: Vec::new(), // Local labels cannot have nested locals
        statements,
        span,
    })
}

fn parse_attribute(pair: Pair<Rule>) -> Result<Attribute, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut key = String::new();
    let mut value = AttributeValue::Literal(String::new());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::attribute_key => {
                key = inner_pair.as_str().to_string();
            }
            Rule::attribute_value => {
                value = parse_attribute_value(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Attribute { key, value, span })
}

fn parse_attribute_value(pair: Pair<Rule>) -> Result<AttributeValue, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::var_ref => Ok(AttributeValue::VarRef(
            inner_pair.into_inner().nth(1).unwrap().as_str().to_string(),
        )),
        Rule::literal_value => Ok(AttributeValue::Literal(inner_pair.as_str().to_string())),
        _ => Ok(AttributeValue::Literal(inner_pair.as_str().to_string())),
    }
}

fn parse_statement(pair: Pair<Rule>) -> Result<Option<Statement>, PastaError> {
    let inner_pair = pair.into_inner().next();
    if inner_pair.is_none() {
        return Ok(None); // Empty statement (newline only)
    }
    let inner_pair = inner_pair.unwrap();

    match inner_pair.as_rule() {
        Rule::speech_line => Ok(Some(parse_speech_line(inner_pair)?)),
        Rule::call_stmt => Ok(Some(parse_call_stmt(inner_pair)?)),
        Rule::jump_stmt => Ok(Some(parse_jump_stmt(inner_pair)?)),
        Rule::var_assign => Ok(Some(parse_var_assign(inner_pair)?)),
        _ => Ok(None),
    }
}

fn parse_speech_line(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut speaker = String::new();
    let mut content = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::speaker => {
                speaker = inner_pair.as_str().trim().to_string();
            }
            Rule::speech_content => {
                content.extend(parse_speech_content(inner_pair)?);
            }
            Rule::continuation_line => {
                for cont_inner in inner_pair.into_inner() {
                    if cont_inner.as_rule() == Rule::speech_content {
                        content.extend(parse_speech_content(cont_inner)?);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Statement::Speech {
        speaker,
        content,
        span,
    })
}

fn parse_speech_content(pair: Pair<Rule>) -> Result<Vec<SpeechPart>, PastaError> {
    let mut parts = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::text_part => {
                parts.push(SpeechPart::Text(inner_pair.as_str().to_string()));
            }
            Rule::var_ref => {
                let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
                parts.push(SpeechPart::VarRef(var_name));
            }
            Rule::func_call => {
                let (name, args, scope) = parse_func_call(inner_pair)?;
                parts.push(SpeechPart::FuncCall { name, args, scope });
            }
            Rule::sakura_script => {
                // sakura_script = sakura_escape ~ sakura_command
                // We want the sakura_command part (second element)
                let cmd = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
                parts.push(SpeechPart::SakuraScript(cmd));
            }
            _ => {}
        }
    }

    Ok(parts)
}

fn parse_call_stmt(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut target = JumpTarget::Local(String::new());
    let mut filters = Vec::new();
    let mut args = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::jump_target => {
                target = parse_jump_target(inner_pair)?;
            }
            Rule::filter_list => {
                filters = parse_filter_list(inner_pair)?;
            }
            Rule::arg_list => {
                args = parse_arg_list_as_expr(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::Call {
        target,
        filters,
        args,
        span,
    })
}

fn parse_jump_stmt(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut target = JumpTarget::Local(String::new());
    let mut filters = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::jump_target => {
                target = parse_jump_target(inner_pair)?;
            }
            Rule::filter_list => {
                filters = parse_filter_list(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::Jump {
        target,
        filters,
        span,
    })
}

fn parse_jump_target(pair: Pair<Rule>) -> Result<JumpTarget, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::dynamic_target => {
            let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
            Ok(JumpTarget::Dynamic(var_name))
        }
        Rule::long_jump => {
            let mut parts = inner_pair.into_inner();
            parts.next(); // Skip global marker
            let global = parts.next().unwrap().as_str().to_string();
            parts.next(); // Skip local marker
            let local = parts.next().unwrap().as_str().to_string();
            Ok(JumpTarget::LongJump { global, local })
        }
        Rule::global_target => {
            let mut parts = inner_pair.into_inner();
            parts.next(); // Skip marker
            let name = parts.next().unwrap().as_str().to_string();
            Ok(JumpTarget::Global(name))
        }
        Rule::local_target => Ok(JumpTarget::Local(inner_pair.as_str().to_string())),
        _ => Ok(JumpTarget::Local(inner_pair.as_str().to_string())),
    }
}

fn parse_filter_list(pair: Pair<Rule>) -> Result<Vec<Attribute>, PastaError> {
    let mut filters = Vec::new();

    for inner_pair in pair.into_inner() {
        // Each iteration should process: at_marker, attribute_key, colon, filter_value
        let mut key = String::new();
        let mut value = AttributeValue::Literal(String::new());
        let mut span_start = (1, 1);
        let mut span_end = (1, 1);

        for part in inner_pair.into_inner() {
            match part.as_rule() {
                Rule::attribute_key => {
                    key = part.as_str().to_string();
                    span_start = part.as_span().start_pos().line_col();
                }
                Rule::filter_value => {
                    value = parse_attribute_value(part.clone())?;
                    span_end = part.as_span().end_pos().line_col();
                }
                _ => {}
            }
        }

        if !key.is_empty() {
            filters.push(Attribute {
                key,
                value,
                span: Span::from_pest(span_start, span_end),
            });
        }
    }

    Ok(filters)
}

fn parse_var_assign(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut name = String::new();
    let mut scope = VarScope::Local;
    let mut value = Expr::Literal(Literal::Number(0.0));

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::var_scope => {
                scope = VarScope::Global; // var_scope only appears for global vars
            }
            Rule::var_name => {
                name = inner_pair.as_str().to_string();
            }
            Rule::expr => {
                value = parse_expr(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok(Statement::VarAssign {
        name,
        scope,
        value,
        span,
    })
}

fn parse_rune_block(pair: Pair<Rule>) -> Result<Statement, PastaError> {
    let span_pest = pair.as_span();
    let start = span_pest.start_pos().line_col();
    let end = span_pest.end_pos().line_col();
    let span = Span::from_pest(start, end);

    let mut content = String::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::rune_content {
            content = inner_pair.as_str().to_string();
            break;
        }
    }

    Ok(Statement::RuneBlock { content, span })
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, PastaError> {
    let mut terms = Vec::new();
    let mut ops = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::term => {
                terms.push(parse_term(inner_pair)?);
            }
            Rule::bin_op => {
                ops.push(parse_bin_op(inner_pair)?);
            }
            _ => {}
        }
    }

    if terms.is_empty() {
        return Ok(Expr::Literal(Literal::Number(0.0)));
    }

    // Build left-associative binary operations
    let mut expr = terms.remove(0);
    for (op, rhs) in ops.into_iter().zip(terms.into_iter()) {
        expr = Expr::BinaryOp {
            op,
            lhs: Box::new(expr),
            rhs: Box::new(rhs),
        };
    }

    Ok(expr)
}

fn parse_term(pair: Pair<Rule>) -> Result<Expr, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::paren_expr => {
            let expr_pair = inner_pair.into_inner().nth(1).unwrap(); // Skip lparen, get expr
            Ok(Expr::Paren(Box::new(parse_expr(expr_pair)?)))
        }
        Rule::func_call => {
            let (name, args, scope) = parse_func_call(inner_pair)?;
            Ok(Expr::FuncCall { name, args, scope })
        }
        Rule::var_ref => {
            let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
            Ok(Expr::VarRef {
                name: var_name,
                scope: VarScope::Local, // Default to local, transpiler will resolve
            })
        }
        Rule::number_literal => Ok(Expr::Literal(Literal::Number(
            inner_pair.as_str().parse().unwrap(),
        ))),
        Rule::string_literal => {
            let str_content = parse_string_literal(inner_pair)?;
            Ok(Expr::Literal(Literal::String(str_content)))
        }
        _ => Ok(Expr::Literal(Literal::Number(0.0))),
    }
}

fn parse_bin_op(pair: Pair<Rule>) -> Result<BinOp, PastaError> {
    let op_pair = pair.into_inner().next().unwrap();
    match op_pair.as_rule() {
        Rule::add => Ok(BinOp::Add),
        Rule::sub => Ok(BinOp::Sub),
        Rule::mul => Ok(BinOp::Mul),
        Rule::div => Ok(BinOp::Div),
        Rule::modulo => Ok(BinOp::Mod),
        _ => Ok(BinOp::Add),
    }
}

fn parse_func_call(pair: Pair<Rule>) -> Result<(String, Vec<Argument>, FunctionScope), PastaError> {
    let mut name = String::new();
    let mut args = Vec::new();
    let mut scope = FunctionScope::Auto; // Default to auto-resolution

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::func_name => {
                let func_name_str = inner_pair.as_str();
                // Check if function name starts with * for global-only scope
                if func_name_str.starts_with('*') || func_name_str.starts_with('＊') {
                    scope = FunctionScope::GlobalOnly;
                    name = func_name_str[1..].trim_start().to_string(); // Remove * prefix
                } else {
                    name = func_name_str.to_string();
                }
            }
            Rule::arg_list => {
                args = parse_arg_list(inner_pair)?;
            }
            _ => {}
        }
    }

    Ok((name, args, scope))
}

fn parse_arg_list(pair: Pair<Rule>) -> Result<Vec<Argument>, PastaError> {
    let mut args = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::argument => {
                args.push(parse_argument(inner_pair)?);
            }
            _ => {} // Skip lparen, rparen
        }
    }

    Ok(args)
}

fn parse_arg_list_as_expr(pair: Pair<Rule>) -> Result<Vec<Expr>, PastaError> {
    let args = parse_arg_list(pair)?;
    Ok(args
        .into_iter()
        .map(|arg| match arg {
            Argument::Positional(expr) => expr,
            Argument::Named { value, .. } => value,
        })
        .collect())
}

fn parse_argument(pair: Pair<Rule>) -> Result<Argument, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::named_arg => {
            let mut parts = inner_pair.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            parts.next(); // Skip colon
            let value_pair = parts.next().unwrap();
            let value = parse_arg_value(value_pair)?;
            Ok(Argument::Named { name, value })
        }
        Rule::positional_arg => {
            let value_pair = inner_pair.into_inner().next().unwrap();
            let value = parse_arg_value(value_pair)?;
            Ok(Argument::Positional(value))
        }
        _ => Ok(Argument::Positional(Expr::Literal(Literal::Number(0.0)))),
    }
}

fn parse_arg_value(pair: Pair<Rule>) -> Result<Expr, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::string_literal => {
            let str_content = parse_string_literal(inner_pair)?;
            Ok(Expr::Literal(Literal::String(str_content)))
        }
        Rule::number_literal => Ok(Expr::Literal(Literal::Number(
            inner_pair.as_str().parse().unwrap(),
        ))),
        Rule::var_ref => {
            let var_name = inner_pair.into_inner().nth(1).unwrap().as_str().to_string();
            Ok(Expr::VarRef {
                name: var_name,
                scope: VarScope::Local,
            })
        }
        Rule::func_call => {
            let (name, args, scope) = parse_func_call(inner_pair)?;
            Ok(Expr::FuncCall { name, args, scope })
        }
        _ => Ok(Expr::Literal(Literal::Number(0.0))),
    }
}

fn parse_string_literal(pair: Pair<Rule>) -> Result<String, PastaError> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::ja_string => {
            let content_pair = inner_pair.into_inner().next().unwrap();
            Ok(content_pair.as_str().to_string())
        }
        Rule::en_string => {
            let content_pair = inner_pair.into_inner().next().unwrap();
            // Handle escape sequences
            let content = content_pair.as_str();
            Ok(content
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\"))
        }
        _ => Ok(String::new()),
    }
}
