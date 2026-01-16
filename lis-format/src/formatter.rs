//! AST-based formatter
//!
//! Formats LIS programs by traversing the AST and generating pretty-printed output.

use crate::config::FormatConfig;
use crate::printer::Printer;
use lis_core::ast::*;

/// Format a complete program
pub fn format_program(program: &Program, config: &FormatConfig) -> String {
    let mut printer = Printer::new(config.clone());
    format_program_impl(program, &mut printer);
    printer.finish()
}

fn format_program_impl(program: &Program, p: &mut Printer) {
    let blank_lines = p.config().blank_lines_between_items;

    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            p.blank_lines(blank_lines);
        }
        format_item(item, p);
    }
}

fn format_item(item: &Item, p: &mut Printer) {
    match item {
        Item::Function { name, params, ret_ty, body, hardware_hint, is_pub, .. } => {
            if *is_pub { p.write("pub "); }
            format_function(name, params, ret_ty.as_ref(), body, hardware_hint.as_ref(), p, "fn");
        }
        Item::Transform { name, params, ret_ty, body, hardware_hint, is_pub, .. } => {
            if *is_pub { p.write("pub "); }
            format_function(name, params, ret_ty.as_ref(), body, hardware_hint.as_ref(), p, "transform");
        }
        Item::TypeAlias { name, ty, is_pub, .. } => {
            if *is_pub { p.write("pub "); }
            p.write("type ");
            p.write(name);
            p.write(" = ");
            format_type(ty, p);
            p.writeln(";");
        }
        Item::Use(use_stmt) => {
            if use_stmt.is_pub { p.write("pub "); }
            p.write("use ");
            for (i, segment) in use_stmt.path.iter().enumerate() {
                if i > 0 { p.write("::"); }
                p.write(segment);
            }
            if let Some(items) = &use_stmt.items {
                p.write("::{");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { p.write(", "); }
                    p.write(item);
                }
                p.write("}");
            } else if let Some(alias) = &use_stmt.alias {
                p.write(" as ");
                p.write(alias);
            }
            p.writeln(";");
        }
        Item::Module(mod_decl) => {
            if mod_decl.is_pub { p.write("pub "); }
            p.write("mod ");
            p.write(&mod_decl.name);
            if let Some(items) = &mod_decl.items {
                p.writeln(" {");
                p.indent();
                for item in items {
                    format_item(item, p);
                }
                p.dedent();
                p.writeln("}");
            } else {
                p.writeln(";");
            }
        }
        Item::ExternFunction(extern_fn) => {
            p.write("extern fn ");
            p.write(&extern_fn.name);
            p.write("(");
            for (i, param) in extern_fn.params.iter().enumerate() {
                if i > 0 { p.write(", "); }
                format_param(param, p);
            }
            p.write(")");
            if let Some(ret_ty) = &extern_fn.ret_ty {
                p.write(" -> ");
                format_type(ret_ty, p);
            }
            p.writeln(";");
        }
    }
}

fn format_function(
    name: &str,
    params: &[Param],
    ret_ty: Option<&Type>,
    body: &[Stmt],
    hardware_hint: Option<&HardwareHint>,
    p: &mut Printer,
    keyword: &str,
) {
    p.write(keyword);
    p.space();
    p.write(name);
    p.write("(");

    for (i, param) in params.iter().enumerate() {
        if i > 0 {
            p.write(",");
            if p.space_after_comma() {
                p.space();
            }
        }
        format_param(param, p);
    }

    p.write(")");

    // Format return type if present
    if let Some(ty) = ret_ty {
        p.write(" -> ");
        format_type(ty, p);
    }

    // Format hardware hint if present
    if let Some(hint) = hardware_hint {
        p.space();
        match hint {
            HardwareHint::Cpu => p.write("@cpu"),
            HardwareHint::Gpu => p.write("@gpu"),
            HardwareHint::Npu => p.write("@npu"),
            HardwareHint::Simd => p.write("@simd"),
            HardwareHint::Photonic => p.write("@photonic"),
        }
    }

    if p.space_before_brace() {
        p.space();
    }
    p.writeln("{");

    p.indented(|p| {
        for stmt in body {
            format_stmt(stmt, p);
        }
    });

    p.writeln("}");
}

fn format_param(param: &Param, p: &mut Printer) {
    p.write(&param.name);
    if let Some(ty) = &param.ty {
        p.write(": ");
        format_type(ty, p);
    }
}

fn format_type(ty: &Type, p: &mut Printer) {
    match ty {
        Type::ByteSil => p.write("ByteSil"),
        Type::State => p.write("State"),
        Type::Layer(n) => {
            p.write("L");
            p.write(&format!("{:X}", n));
        }
        Type::Hardware(hint) => {
            match hint {
                HardwareHint::Cpu => p.write("@cpu"),
                HardwareHint::Gpu => p.write("@gpu"),
                HardwareHint::Npu => p.write("@npu"),
                HardwareHint::Simd => p.write("@simd"),
                HardwareHint::Photonic => p.write("@photonic"),
            }
        }
        Type::Function { params, ret } => {
            p.write("(");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    p.write(",");
                    if p.space_after_comma() {
                        p.space();
                    }
                }
                format_type(param, p);
            }
            p.write(") -> ");
            format_type(ret, p);
        }
        Type::Tuple(types) => {
            p.write("(");
            for (i, ty) in types.iter().enumerate() {
                if i > 0 {
                    p.write(",");
                    if p.space_after_comma() {
                        p.space();
                    }
                }
                format_type(ty, p);
            }
            p.write(")");
        }
        Type::Named(name) => p.write(name),
    }
}

fn format_stmt(stmt: &Stmt, p: &mut Printer) {
    match stmt {
        Stmt::Let { name, ty, value, .. } => {
            p.write("let ");
            p.write(name);
            if let Some(ty) = ty {
                p.write(": ");
                format_type(ty, p);
            }
            if p.space_around_operators() {
                p.write(" = ");
            } else {
                p.write("=");
            }
            format_expr(value, p, 0);
            p.writeln(";");
        }

        Stmt::Assign { name, value, .. } => {
            p.write(name);
            if p.space_around_operators() {
                p.write(" = ");
            } else {
                p.write("=");
            }
            format_expr(value, p, 0);
            p.writeln(";");
        }

        Stmt::Expr(expr) => {
            format_expr(expr, p, 0);
            p.writeln(";");
        }

        Stmt::Return(expr_opt, _span) => {
            p.write("return");
            if let Some(expr) = expr_opt {
                p.space();
                format_expr(expr, p, 0);
            }
            p.writeln(";");
        }

        Stmt::Loop { body, .. } => {
            p.write("loop");
            if p.space_before_brace() {
                p.space();
            }
            p.writeln("{");
            p.indented(|p| {
                for stmt in body {
                    format_stmt(stmt, p);
                }
            });
            p.writeln("}");
        }

        Stmt::Break(_span) => p.writeln("break;"),
        Stmt::Continue(_span) => p.writeln("continue;"),

        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            p.write("if ");
            format_expr(condition, p, 0);
            if p.space_before_brace() {
                p.space();
            }
            p.writeln("{");
            p.indented(|p| {
                for stmt in then_body {
                    format_stmt(stmt, p);
                }
            });
            p.write("}");

            if let Some(else_stmts) = else_body {
                p.write(" else");
                if p.space_before_brace() {
                    p.space();
                }
                p.writeln("{");
                p.indented(|p| {
                    for stmt in else_stmts {
                        format_stmt(stmt, p);
                    }
                });
                p.writeln("}");
            } else {
                p.newline();
            }
        }
    }
}

fn format_expr(expr: &Expr, p: &mut Printer, precedence: u8) {
    use ExprKind::*;
    match &expr.kind {
        Literal(lit) => format_literal(lit, p),

        Ident(name) => p.write(name),

        Binary { left, op, right } => {
            let op_prec = op_precedence(op);
            let needs_parens = op_prec < precedence;

            if needs_parens {
                p.write("(");
            }

            format_expr(left, p, op_prec);

            if p.space_around_operators() {
                p.write(" ");
            }
            p.write(&format!("{}", op));
            if p.space_around_operators() {
                p.write(" ");
            }

            format_expr(right, p, op_prec);

            if needs_parens {
                p.write(")");
            }
        }

        Unary { op, expr: inner } => {
            match op {
                UnOp::Mag => {
                    // Magnitude uses |expr|
                    p.write("|");
                    format_expr(inner, p, 99);
                    p.write("|");
                }
                _ => {
                    p.write(&format!("{}", op));
                    format_expr(inner, p, 99);
                }
            }
        }

        Call { name, args } => {
            p.write(name);
            p.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    p.write(",");
                    if p.space_after_comma() {
                        p.space();
                    }
                }
                format_expr(arg, p, 0);
            }
            p.write(")");
        }

        LayerAccess { expr: inner, layer } => {
            format_expr(inner, p, 99);
            p.write(".");
            p.write(&format!("L{:X}", layer));
        }

        StateConstruct { layers } => {
            p.write("State");
            if p.space_before_brace() {
                p.space();
            }
            p.write("{");
            if p.space_after_comma() {
                p.space();
            }

            for (i, (layer, layer_expr)) in layers.iter().enumerate() {
                if i > 0 {
                    p.write(",");
                    if p.space_after_comma() {
                        p.space();
                    }
                }
                p.write(&format!("L{:X}:", layer));
                if p.space_after_comma() {
                    p.space();
                }
                format_expr(layer_expr, p, 0);
            }

            if p.space_after_comma() {
                p.space();
            }
            p.write("}");
        }

        Complex { rho, theta } => {
            p.write("(");
            format_expr(rho, p, 0);
            p.write(",");
            if p.space_after_comma() {
                p.space();
            }
            format_expr(theta, p, 0);
            p.write(")");
        }

        Tuple { elements } => {
            p.write("(");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    p.write(",");
                    if p.space_after_comma() {
                        p.space();
                    }
                }
                format_expr(elem, p, 0);
            }
            p.write(")");
        }

        Pipe { expr: inner, transform } => {
            format_expr(inner, p, 0);
            if p.space_around_operators() {
                p.write(" ");
            }
            p.write("|>");
            if p.space_around_operators() {
                p.write(" ");
            }
            p.write(transform);
        }

        Feedback { expr: inner } => {
            p.write("feedback ");
            format_expr(inner, p, 0);
        }

        Emerge { expr: inner } => {
            p.write("emerge ");
            format_expr(inner, p, 0);
        }
    }
}

fn format_literal(lit: &Literal, p: &mut Printer) {
    match lit {
        Literal::Int(n) => p.write(&n.to_string()),
        Literal::Float(f) => {
            let s = f.to_string();
            // Ensure float has decimal point
            if !s.contains('.') && !s.contains('e') {
                p.write(&format!("{}.0", s));
            } else {
                p.write(&s);
            }
        }
        Literal::Bool(b) => p.write(if *b { "true" } else { "false" }),
        Literal::String(s) => {
            p.write("\"");
            // Escape special characters
            for ch in s.chars() {
                match ch {
                    '"' => p.write("\\\""),
                    '\\' => p.write("\\\\"),
                    '\n' => p.write("\\n"),
                    '\r' => p.write("\\r"),
                    '\t' => p.write("\\t"),
                    _ => p.write(&ch.to_string()),
                }
            }
            p.write("\"");
        }
    }
}

/// Get operator precedence (higher = tighter binding)
fn op_precedence(op: &BinOp) -> u8 {
    match op {
        BinOp::Or => 10,
        BinOp::And => 20,
        BinOp::Eq | BinOp::Ne => 30,
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 40,
        BinOp::BitOr => 50,
        BinOp::Xor => 60,
        BinOp::BitAnd => 70,
        BinOp::Add | BinOp::Sub => 80,
        BinOp::Mul | BinOp::Div => 90,
        BinOp::Pow => 100,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lis_core::lexer::Span;

    #[test]
    fn test_format_empty_function() {
        let program = Program {
            items: vec![Item::Function {
                name: "main".to_string(),
                params: vec![],
                ret_ty: None,
                body: vec![],
                hardware_hint: None,
                is_pub: false,
                span: Span::dummy(),
            }],
        };

        let result = format_program(&program, &FormatConfig::default());
        assert!(result.contains("fn main()"));
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_with_params() {
        let program = Program {
            items: vec![Item::Function {
                name: "add".to_string(),
                params: vec![
                    Param {
                        name: "x".to_string(),
                        ty: Some(Type::Named("Int".to_string())),
                    },
                    Param {
                        name: "y".to_string(),
                        ty: Some(Type::Named("Int".to_string())),
                    },
                ],
                ret_ty: None,
                body: vec![],
                hardware_hint: None,
                is_pub: false,
                span: Span::dummy(),
            }],
        };

        let result = format_program(&program, &FormatConfig::default());
        assert!(result.contains("fn add(x: Int, y: Int)"));
    }
}
