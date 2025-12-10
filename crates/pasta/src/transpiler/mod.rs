//! Transpiler module for converting Pasta AST to Rune source code.
//!
//! This module converts the Pasta AST into Rune source code that can be executed
//! by the Rune VM to generate ScriptEvent IR.

use crate::{
    Argument, BinOp, Expr, JumpTarget, LabelDef, LabelScope, Literal, PastaError,
    PastaFile, SpeechPart, Statement, VarScope,
};
use std::collections::HashMap;

/// Transpiler that converts Pasta AST to Rune source code.
pub struct Transpiler;

impl Transpiler {
    /// Transpile a Pasta file AST to Rune source code.
    pub fn transpile(file: &PastaFile) -> Result<String, PastaError> {
        let mut output = String::new();

        // Add imports for standard library functions
        output.push_str("use pasta_stdlib::*;\n\n");

        // Track label counters to generate unique function names for duplicates
        let mut label_counters: HashMap<String, usize> = HashMap::new();

        // Transpile each global label
        for label in &file.labels {
            let counter = label_counters.entry(label.name.clone()).or_insert(0);
            Self::transpile_label_with_counter(&mut output, label, None, *counter)?;
            *counter += 1;
        }

        Ok(output)
    }

    /// Transpile a single label definition to a Rune function with a counter for duplicates.
    fn transpile_label_with_counter(
        output: &mut String,
        label: &LabelDef,
        parent_name: Option<&str>,
        counter: usize,
    ) -> Result<(), PastaError> {
        let fn_name = Self::label_to_fn_name_with_counter(label, parent_name, counter);

        // Function signature - generators don't need async keyword in Rune
        output.push_str(&format!("pub fn {}() {{\n", fn_name));

        // Transpile statements
        for stmt in &label.statements {
            Self::transpile_statement(output, stmt)?;
        }

        // Transpile local labels (with their own counter tracking)
        let mut local_counters: HashMap<String, usize> = HashMap::new();
        for local_label in &label.local_labels {
            let counter = local_counters.entry(local_label.name.clone()).or_insert(0);
            Self::transpile_label_with_counter(output, local_label, Some(&label.name), *counter)?;
            *counter += 1;
        }

        output.push_str("}\n\n");
        Ok(())
    }

    /// Generate a function name from a label definition with counter for duplicates.
    fn label_to_fn_name_with_counter(label: &LabelDef, parent_name: Option<&str>, counter: usize) -> String {
        let base_name = match label.scope {
            LabelScope::Global => {
                // Global labels use their name directly
                Self::sanitize_identifier(&label.name)
            }
            LabelScope::Local => {
                // Local labels are prefixed with parent name
                if let Some(parent) = parent_name {
                    format!("{}__{}", Self::sanitize_identifier(parent), Self::sanitize_identifier(&label.name))
                } else {
                    Self::sanitize_identifier(&label.name)
                }
            }
        };
        
        // Append counter if this is a duplicate (counter > 0)
        if counter > 0 {
            format!("{}_{}", base_name, counter)
        } else {
            base_name
        }
    }

    /// Sanitize identifier to be valid Rune function name.
    fn sanitize_identifier(name: &str) -> String {
        // For now, just replace invalid characters with underscores
        // In the future, this might need more sophisticated handling
        name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }

    /// Transpile a statement to Rune code.
    fn transpile_statement(output: &mut String, stmt: &Statement) -> Result<(), PastaError> {
        match stmt {
            Statement::Speech {
                speaker,
                content,
                span: _,
            } => {
                // Emit change speaker
                output.push_str(&format!("    yield change_speaker(\"{}\");\n", speaker));

                // Emit each content part
                for part in content {
                    Self::transpile_speech_part(output, part)?;
                }
            }
            Statement::Call {
                target,
                filters: _,
                args: _,
                span: _,
            } => {
                // Generate call statement
                let target_fn = Self::transpile_jump_target(target);
                output.push_str(&format!("    {}();\n", target_fn));
            }
            Statement::Jump {
                target,
                filters: _,
                span: _,
            } => {
                // Generate jump (return from current function and call target)
                let target_fn = Self::transpile_jump_target(target);
                output.push_str(&format!("    return {}();\n", target_fn));
            }
            Statement::VarAssign {
                name,
                scope,
                value,
                span: _,
            } => {
                let value_expr = Self::transpile_expr(value)?;
                match scope {
                    VarScope::Local => {
                        output.push_str(&format!("    let {} = {};\n", name, value_expr));
                    }
                    VarScope::Global => {
                        output.push_str(&format!(
                            "    set_global(\"{}\", {});\n",
                            name, value_expr
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Transpile a speech part to Rune code.
    fn transpile_speech_part(output: &mut String, part: &SpeechPart) -> Result<(), PastaError> {
        match part {
            SpeechPart::Text(text) => {
                output.push_str(&format!("    yield emit_text(\"{}\");\n", Self::escape_string(text)));
            }
            SpeechPart::VarRef(var_name) => {
                output.push_str(&format!(
                    "    yield emit_text(&format!(\"{{}}\", get_variable(\"{}\")));\n",
                    var_name
                ));
            }
            SpeechPart::FuncCall { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| match arg {
                        Argument::Positional(expr) => Self::transpile_expr(expr),
                        Argument::Named { name, value } => {
                            Ok(format!("{}={}", name, Self::transpile_expr(value)?))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                output.push_str(&format!("    yield {}({});\n", name, args_str));
            }
            SpeechPart::SakuraScript(script) => {
                output.push_str(&format!(
                    "    yield emit_sakura_script(\"{}\");\n",
                    Self::escape_string(script)
                ));
            }
        }
        Ok(())
    }

    /// Transpile a jump target to a function name.
    fn transpile_jump_target(target: &JumpTarget) -> String {
        match target {
            JumpTarget::Local(name) => Self::sanitize_identifier(name),
            JumpTarget::Global(name) => Self::sanitize_identifier(name),
            JumpTarget::LongJump { global, local } => {
                format!(
                    "{}_{}",
                    Self::sanitize_identifier(global),
                    Self::sanitize_identifier(local)
                )
            }
            JumpTarget::Dynamic(var_name) => {
                // Dynamic targets need to be resolved at runtime
                format!("resolve_label(\"{}\")", var_name)
            }
        }
    }

    /// Transpile an expression to Rune code.
    fn transpile_expr(expr: &Expr) -> Result<String, PastaError> {
        match expr {
            Expr::Literal(lit) => Ok(Self::transpile_literal(lit)),
            Expr::VarRef { name, scope } => match scope {
                VarScope::Local => Ok(name.clone()),
                VarScope::Global => Ok(format!("get_global(\"{}\")", name)),
            },
            Expr::FuncCall { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| match arg {
                        Argument::Positional(expr) => Self::transpile_expr(expr),
                        Argument::Named { name, value } => {
                            Ok(format!("{}={}", name, Self::transpile_expr(value)?))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", name, args_str))
            }
            Expr::BinaryOp { op, lhs, rhs } => {
                let lhs_str = Self::transpile_expr(lhs)?;
                let rhs_str = Self::transpile_expr(rhs)?;
                let op_str = Self::transpile_binop(*op);
                Ok(format!("({} {} {})", lhs_str, op_str, rhs_str))
            }
            Expr::Paren(inner) => {
                let inner_str = Self::transpile_expr(inner)?;
                Ok(format!("({})", inner_str))
            }
        }
    }

    /// Transpile a literal to Rune code.
    fn transpile_literal(lit: &Literal) -> String {
        match lit {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => format!("\"{}\"", Self::escape_string(s)),
        }
    }

    /// Transpile a binary operator to Rune code.
    fn transpile_binop(op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
        }
    }

    /// Escape a string for use in Rune code.
    fn escape_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Span;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(Transpiler::sanitize_identifier("hello"), "hello");
        assert_eq!(Transpiler::sanitize_identifier("hello-world"), "hello_world");
        assert_eq!(Transpiler::sanitize_identifier("＊挨拶"), "_挨拶"); // Full-width asterisk replaced, Japanese kept
        assert_eq!(Transpiler::sanitize_identifier("挨拶"), "挨拶"); // Pure Japanese unchanged
    }

    #[test]
    fn test_transpile_simple_label() {
        let file = PastaFile {
            path: "test.pasta".into(),
            labels: vec![LabelDef {
                name: "greeting".to_string(),
                scope: LabelScope::Global,
                attributes: vec![],
                local_labels: vec![],
                statements: vec![Statement::Speech {
                    speaker: "sakura".to_string(),
                    content: vec![SpeechPart::Text("Hello!".to_string())],
                    span: Span::new(1, 1, 1, 10),
                }],
                span: Span::new(1, 1, 2, 1),
            }],
            span: Span::new(1, 1, 2, 1),
        };

        let result = Transpiler::transpile(&file).unwrap();
        assert!(result.contains("pub fn greeting()"));
        assert!(result.contains("yield change_speaker(\"sakura\")"));
        assert!(result.contains("yield emit_text(\"Hello!\")"));
    }

    #[test]
    fn test_transpile_expr() {
        let expr = Expr::BinaryOp {
            op: BinOp::Add,
            lhs: Box::new(Expr::Literal(Literal::Number(1.0))),
            rhs: Box::new(Expr::Literal(Literal::Number(2.0))),
        };
        let result = Transpiler::transpile_expr(&expr).unwrap();
        assert_eq!(result, "(1 + 2)");
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(Transpiler::escape_string("hello"), "hello");
        assert_eq!(Transpiler::escape_string("hello\"world"), "hello\\\"world");
        assert_eq!(Transpiler::escape_string("hello\\world"), "hello\\\\world");
        assert_eq!(Transpiler::escape_string("hello\nworld"), "hello\\nworld");
    }
}
