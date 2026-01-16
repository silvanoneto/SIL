//! Parser for LIS language
//!
//! Parses token stream into an Abstract Syntax Tree (AST).

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::{Token, Span, SpannedToken};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

/// Legacy parser constructor that accepts plain tokens (for backwards compatibility)
pub fn parse_tokens(tokens: Vec<Token>) -> Result<Program> {
    let spanned: Vec<SpannedToken> = tokens.into_iter()
        .map(|token| SpannedToken { token, span: Span::dummy() })
        .collect();
    Parser::new(spanned).parse()
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Create parser from plain tokens (backwards compatible)
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        let spanned: Vec<SpannedToken> = tokens.into_iter()
            .map(|token| SpannedToken { token, span: Span::dummy() })
            .collect();
        Self::new(spanned)
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        Ok(Program { items })
    }

    // ===== Item Parsing =====

    fn parse_item(&mut self) -> Result<Item> {
        // Check for pub modifier
        let is_pub = if matches!(self.peek_token(), Some(Token::Pub)) {
            self.advance();
            true
        } else {
            false
        };

        match self.peek_token() {
            Some(Token::Fn) => self.parse_function(is_pub),
            Some(Token::Transform) => self.parse_transform(is_pub),
            Some(Token::Type) => self.parse_type_alias(is_pub),
            Some(Token::Use) => self.parse_use_statement(is_pub),
            Some(Token::Mod) => self.parse_module_decl(is_pub),
            Some(Token::Extern) => self.parse_extern_fn(),
            _ => Err(self.error("Expected item (fn, transform, type, use, mod, or extern)")),
        }
    }

    fn parse_function(&mut self, is_pub: bool) -> Result<Item> {
        let start_span = self.current_span();
        self.expect(Token::Fn)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        // Parse optional return type: -> Type
        let ret_ty = if matches!(self.peek_token(), Some(Token::Arrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse optional hardware hint: @cpu, @gpu, @npu, @simd, @photonic
        let hardware_hint = self.parse_hardware_hint();

        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        let end_span = self.current_span();
        self.expect(Token::RBrace)?;

        Ok(Item::Function { name, params, ret_ty, body, hardware_hint, is_pub, span: start_span.merge(&end_span) })
    }

    fn parse_transform(&mut self, is_pub: bool) -> Result<Item> {
        let start_span = self.current_span();
        self.expect(Token::Transform)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        // Parse optional return type: -> Type
        let ret_ty = if matches!(self.peek_token(), Some(Token::Arrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse optional hardware hint: @cpu, @gpu, @npu, @simd, @photonic
        let hardware_hint = self.parse_hardware_hint();

        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        let end_span = self.current_span();
        self.expect(Token::RBrace)?;

        Ok(Item::Transform { name, params, ret_ty, body, hardware_hint, is_pub, span: start_span.merge(&end_span) })
    }

    fn parse_hardware_hint(&mut self) -> Option<HardwareHint> {
        match self.peek_token() {
            Some(Token::HintCpu) => { self.advance(); Some(HardwareHint::Cpu) }
            Some(Token::HintGpu) => { self.advance(); Some(HardwareHint::Gpu) }
            Some(Token::HintNpu) => { self.advance(); Some(HardwareHint::Npu) }
            Some(Token::HintSimd) => { self.advance(); Some(HardwareHint::Simd) }
            Some(Token::HintPhotonic) => { self.advance(); Some(HardwareHint::Photonic) }
            _ => None,
        }
    }

    fn parse_type_alias(&mut self, is_pub: bool) -> Result<Item> {
        let start_span = self.current_span();
        self.expect(Token::Type)?;
        let name = self.expect_ident()?;
        self.expect(Token::Eq)?;
        let ty = self.parse_type()?;
        let end_span = self.current_span();
        self.expect(Token::Semi)?;

        Ok(Item::TypeAlias { name, ty, is_pub, span: start_span.merge(&end_span) })
    }

    /// Parse use statement: `use path::to::module;` or `use path::to::module as alias;`
    fn parse_use_statement(&mut self, is_pub: bool) -> Result<Item> {
        let start_span = self.current_span();
        self.expect(Token::Use)?;

        // Parse path: ident (:: ident)*
        let mut path = vec![self.expect_ident()?];
        while matches!(self.peek_token(), Some(Token::ColonColon)) {
            self.advance(); // consume ::

            // Check for { item1, item2 } syntax
            if matches!(self.peek_token(), Some(Token::LBrace)) {
                self.advance(); // consume {
                let items = self.parse_use_items()?;
                let end_span = self.current_span();
                self.expect(Token::RBrace)?;
                self.expect(Token::Semi)?;
                return Ok(Item::Use(UseStatement {
                    path,
                    alias: None,
                    items: Some(items),
                    is_pub,
                    span: start_span.merge(&end_span),
                }));
            }

            path.push(self.expect_ident()?);
        }

        // Parse optional alias: as name
        let alias = if matches!(self.peek_token(), Some(Token::As)) {
            self.advance();
            Some(self.expect_ident()?)
        } else {
            None
        };

        let end_span = self.current_span();
        self.expect(Token::Semi)?;

        Ok(Item::Use(UseStatement {
            path,
            alias,
            items: None,
            is_pub,
            span: start_span.merge(&end_span),
        }))
    }

    /// Parse items in `use path::{item1, item2}`
    fn parse_use_items(&mut self) -> Result<Vec<String>> {
        let mut items = Vec::new();

        if matches!(self.peek_token(), Some(Token::RBrace)) {
            return Ok(items);
        }

        loop {
            items.push(self.expect_ident()?);

            if matches!(self.peek_token(), Some(Token::Comma)) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(items)
    }

    /// Parse module declaration: `mod name;` or `mod name { items }`
    fn parse_module_decl(&mut self, is_pub: bool) -> Result<Item> {
        let start_span = self.current_span();
        self.expect(Token::Mod)?;
        let name = self.expect_ident()?;

        // Check for inline module: mod name { ... }
        let items = if matches!(self.peek_token(), Some(Token::LBrace)) {
            self.advance(); // consume {
            let mut module_items = Vec::new();
            while !matches!(self.peek_token(), Some(Token::RBrace)) && !self.is_at_end() {
                module_items.push(self.parse_item()?);
            }
            self.expect(Token::RBrace)?;
            Some(module_items)
        } else {
            // External module: mod name;
            self.expect(Token::Semi)?;
            None
        };

        let end_span = self.current_span();

        Ok(Item::Module(ModuleDecl {
            name,
            items,
            is_pub,
            span: start_span.merge(&end_span),
        }))
    }

    /// Parse extern function: `extern fn name(params) -> Type;`
    fn parse_extern_fn(&mut self) -> Result<Item> {
        let start_span = self.current_span();
        self.expect(Token::Extern)?;
        self.expect(Token::Fn)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        // Parse optional return type: -> Type
        let ret_ty = if matches!(self.peek_token(), Some(Token::Arrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let end_span = self.current_span();
        self.expect(Token::Semi)?;

        Ok(Item::ExternFunction(ExternFn {
            name,
            params,
            ret_ty,
            span: start_span.merge(&end_span),
        }))
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();

        if matches!(self.peek_token(), Some(Token::RParen)) {
            return Ok(params);
        }

        loop {
            let name = self.expect_ident()?;
            let ty = if matches!(self.peek_token(), Some(Token::Colon)) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };

            params.push(Param { name, ty });

            if !matches!(self.peek_token(), Some(Token::Comma)) {
                break;
            }
            self.advance();
        }

        Ok(params)
    }

    // ===== Type Parsing =====

    fn parse_type(&mut self) -> Result<Type> {
        match self.peek_token() {
            Some(Token::TByteSil) => {
                self.advance();
                Ok(Type::ByteSil)
            }
            Some(Token::TState) => {
                self.advance();
                Ok(Type::State)
            }
            Some(Token::Layer(n)) => {
                let layer = *n;
                self.advance();
                Ok(Type::Layer(layer))
            }
            Some(Token::HintCpu) => {
                self.advance();
                Ok(Type::Hardware(HardwareHint::Cpu))
            }
            Some(Token::HintGpu) => {
                self.advance();
                Ok(Type::Hardware(HardwareHint::Gpu))
            }
            Some(Token::HintNpu) => {
                self.advance();
                Ok(Type::Hardware(HardwareHint::Npu))
            }
            Some(Token::LParen) => {
                // Tuple type: (T1, T2, ...)
                self.advance();
                let mut types = Vec::new();

                if !matches!(self.peek_token(), Some(Token::RParen)) {
                    loop {
                        types.push(self.parse_type()?);
                        if !matches!(self.peek_token(), Some(Token::Comma)) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
                Ok(Type::Tuple(types))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Type::Named(name))
            }
            _ => Err(self.error("Expected type")),
        }
    }

    // ===== Statement Parsing =====

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();

        while !matches!(self.peek_token(), Some(Token::RBrace)) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek_token() {
            Some(Token::Let) => self.parse_let(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Loop) => self.parse_loop(),
            Some(Token::Break) => {
                let span = self.current_span();
                self.advance();
                self.expect(Token::Semi)?;
                Ok(Stmt::Break(span))
            }
            Some(Token::Continue) => {
                let span = self.current_span();
                self.advance();
                self.expect(Token::Semi)?;
                Ok(Stmt::Continue(span))
            }
            Some(Token::If) => self.parse_if(),
            _ => {
                // Try to parse as assignment or expression statement
                let expr = self.parse_expr()?;

                if matches!(self.peek_token(), Some(Token::Eq)) {
                    // Assignment
                    if let ExprKind::Ident(name) = &expr.kind {
                        let span = expr.span;
                        let name = name.clone();
                        self.expect(Token::Eq)?;
                        let value = self.parse_expr()?;
                        let end_span = self.current_span();
                        self.expect(Token::Semi)?;
                        Ok(Stmt::Assign { name, value, span: span.merge(&end_span) })
                    } else {
                        Err(self.error("Expected identifier on left side of assignment"))
                    }
                } else {
                    // Expression statement
                    self.expect(Token::Semi)?;
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt> {
        let start_span = self.current_span();
        self.expect(Token::Let)?;
        let name = self.expect_ident()?;

        let ty = if matches!(self.peek_token(), Some(Token::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        let end_span = self.current_span();
        self.expect(Token::Semi)?;

        Ok(Stmt::Let { name, ty, value, span: start_span.merge(&end_span) })
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        let start_span = self.current_span();
        self.expect(Token::Return)?;

        let value = if matches!(self.peek_token(), Some(Token::Semi)) {
            None
        } else {
            Some(self.parse_expr()?)
        };

        let end_span = self.current_span();
        self.expect(Token::Semi)?;
        Ok(Stmt::Return(value, start_span.merge(&end_span)))
    }

    fn parse_loop(&mut self) -> Result<Stmt> {
        let start_span = self.current_span();
        self.expect(Token::Loop)?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        let end_span = self.current_span();
        self.expect(Token::RBrace)?;

        Ok(Stmt::Loop { body, span: start_span.merge(&end_span) })
    }

    fn parse_if(&mut self) -> Result<Stmt> {
        let start_span = self.current_span();
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let then_body = self.parse_block()?;
        self.expect(Token::RBrace)?;

        let else_body = if matches!(self.peek_token(), Some(Token::Else)) {
            self.advance();
            self.expect(Token::LBrace)?;
            let body = self.parse_block()?;
            self.expect(Token::RBrace)?;
            Some(body)
        } else {
            None
        };

        let end_span = self.current_span();
        Ok(Stmt::If {
            condition,
            then_body,
            else_body,
            span: start_span.merge(&end_span),
        })
    }

    // ===== Expression Parsing =====

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_or()?;

        while matches!(self.peek_token(), Some(Token::PipeRight)) {
            self.advance();
            let transform = self.expect_ident()?;
            let span = expr.span.merge(&self.prev_span());
            expr = Expr::new(ExprKind::Pipe {
                expr: Box::new(expr),
                transform,
            }, span);
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_logical_and()?;

        while matches!(self.peek_token(), Some(Token::OrOr)) {
            self.advance();
            let right = self.parse_logical_and()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(ExprKind::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            }, span);
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;

        while matches!(self.peek_token(), Some(Token::AndAnd)) {
            self.advance();
            let right = self.parse_equality()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(ExprKind::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            }, span);
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;

        while let Some(op_token) = self.peek_token() {
            let op = match op_token {
                Token::EqEq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                _ => break,
            };

            self.advance();
            let right = self.parse_comparison()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }, span);
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;

        while let Some(op_token) = self.peek_token() {
            let op = match op_token {
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };

            self.advance();
            let right = self.parse_term()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }, span);
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut left = self.parse_factor()?;

        while let Some(op_token) = self.peek_token() {
            let op = match op_token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };

            self.advance();
            let right = self.parse_factor()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }, span);
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        while let Some(op_token) = self.peek_token() {
            let op = match op_token {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::StarStar => BinOp::Pow,
                Token::Caret => BinOp::Xor,
                _ => break,
            };

            self.advance();
            let right = self.parse_unary()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }, span);
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek_token() {
            Some(Token::Minus) => {
                let start_span = self.current_span();
                self.advance();
                let expr = self.parse_unary()?;
                let span = start_span.merge(&expr.span);
                Ok(Expr::new(ExprKind::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(expr),
                }, span))
            }
            Some(Token::Bang) => {
                let start_span = self.current_span();
                self.advance();
                let expr = self.parse_unary()?;
                let span = start_span.merge(&expr.span);
                Ok(Expr::new(ExprKind::Unary {
                    op: UnOp::Not,
                    expr: Box::new(expr),
                }, span))
            }
            Some(Token::Tilde) => {
                let start_span = self.current_span();
                self.advance();
                let expr = self.parse_unary()?;
                let span = start_span.merge(&expr.span);
                Ok(Expr::new(ExprKind::Unary {
                    op: UnOp::Conj,
                    expr: Box::new(expr),
                }, span))
            }
            Some(Token::Pipe) => {
                // Magnitude operator |expr|
                let start_span = self.current_span();
                self.advance();

                // Parse inner expression (everything except pipe operator)
                let expr = self.parse_logical_or()?;

                // Expect closing |
                if matches!(self.peek_token(), Some(Token::Pipe)) {
                    let end_span = self.current_span();
                    self.advance();
                    Ok(Expr::new(ExprKind::Unary {
                        op: UnOp::Mag,
                        expr: Box::new(expr),
                    }, start_span.merge(&end_span)))
                } else {
                    Err(self.error("Expected closing | for magnitude operator"))
                }
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_token() {
                Some(Token::Dot) => {
                    self.advance();
                    if let Some(Token::Layer(layer)) = self.peek_token() {
                        let layer = *layer;
                        let end_span = self.current_span();
                        self.advance();
                        let span = expr.span.merge(&end_span);
                        expr = Expr::new(ExprKind::LayerAccess {
                            expr: Box::new(expr),
                            layer,
                        }, span);
                    } else {
                        return Err(self.error("Expected layer after '.'"));
                    }
                }
                Some(Token::LParen) => {
                    // Function call
                    if let ExprKind::Ident(name) = &expr.kind {
                        let name = name.clone();
                        let start_span = expr.span;
                        self.advance();
                        let args = self.parse_args()?;
                        let end_span = self.current_span();
                        self.expect(Token::RParen)?;
                        expr = Expr::new(ExprKind::Call { name, args }, start_span.merge(&end_span));
                    } else {
                        return Err(self.error("Expected identifier before '('"));
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let span = self.current_span();
        match self.peek_token() {
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Int(n)), span))
            }
            Some(Token::Float(f)) => {
                let f = *f;
                self.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Float(f)), span))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Bool(true)), span))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Bool(false)), span))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::String(s)), span))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::new(ExprKind::Ident(name), span))
            }
            Some(Token::Feedback) => {
                self.advance();
                let inner = self.parse_expr()?;
                let end_span = inner.span;
                Ok(Expr::new(ExprKind::Feedback {
                    expr: Box::new(inner),
                }, span.merge(&end_span)))
            }
            Some(Token::Emerge) => {
                self.advance();
                let inner = self.parse_expr()?;
                let end_span = inner.span;
                Ok(Expr::new(ExprKind::Emerge {
                    expr: Box::new(inner),
                }, span.merge(&end_span)))
            }
            Some(Token::LParen) => {
                self.advance();

                // Check if empty tuple ()
                if matches!(self.peek_token(), Some(Token::RParen)) {
                    let end_span = self.current_span();
                    self.advance();
                    return Ok(Expr::new(ExprKind::Tuple { elements: vec![] }, span.merge(&end_span)));
                }

                // Parse first expression
                let first_expr = self.parse_expr()?;

                if matches!(self.peek_token(), Some(Token::Comma)) {
                    // Tuple or Complex number
                    self.advance(); // consume comma

                    // Check if there's a closing paren (2-element tuple/complex)
                    if matches!(self.peek_token(), Some(Token::RParen)) {
                        // This shouldn't happen, but handle gracefully
                        let end_span = self.current_span();
                        self.advance();
                        return Ok(Expr::new(ExprKind::Tuple {
                            elements: vec![first_expr]
                        }, span.merge(&end_span)));
                    }

                    let second_expr = self.parse_expr()?;

                    // Check if there are more elements (tuple with 3+ elements)
                    if matches!(self.peek_token(), Some(Token::Comma)) {
                        // Definitely a tuple with 3+ elements
                        let mut elements = vec![first_expr, second_expr];
                        while matches!(self.peek_token(), Some(Token::Comma)) {
                            self.advance();
                            if matches!(self.peek_token(), Some(Token::RParen)) {
                                break; // trailing comma
                            }
                            elements.push(self.parse_expr()?);
                        }
                        let end_span = self.current_span();
                        self.expect(Token::RParen)?;
                        Ok(Expr::new(ExprKind::Tuple { elements }, span.merge(&end_span)))
                    } else {
                        // Two elements - could be complex or 2-tuple
                        // Use complex for numeric literals, tuple otherwise
                        let end_span = self.current_span();
                        self.expect(Token::RParen)?;

                        let is_numeric = |e: &Expr| matches!(
                            e.kind,
                            ExprKind::Literal(Literal::Int(_)) |
                            ExprKind::Literal(Literal::Float(_)) |
                            ExprKind::Unary { op: UnOp::Neg, .. }
                        );

                        if is_numeric(&first_expr) && is_numeric(&second_expr) {
                            Ok(Expr::new(ExprKind::Complex {
                                rho: Box::new(first_expr),
                                theta: Box::new(second_expr),
                            }, span.merge(&end_span)))
                        } else {
                            Ok(Expr::new(ExprKind::Tuple {
                                elements: vec![first_expr, second_expr],
                            }, span.merge(&end_span)))
                        }
                    }
                } else {
                    // Just parentheses for grouping
                    let end_span = self.current_span();
                    self.expect(Token::RParen)?;
                    // Preserve the inner expression's span but update to include parens
                    Ok(Expr::new(first_expr.kind, span.merge(&end_span)))
                }
            }
            Some(Token::TState) => {
                // State construction: State { L0: expr, L1: expr, ... }
                self.advance();
                self.expect(Token::LBrace)?;
                let layers = self.parse_state_layers()?;
                let end_span = self.current_span();
                self.expect(Token::RBrace)?;
                Ok(Expr::new(ExprKind::StateConstruct { layers }, span.merge(&end_span)))
            }
            _ => Err(self.error("Expected expression")),
        }
    }

    fn parse_state_layers(&mut self) -> Result<Vec<(u8, Expr)>> {
        let mut layers = Vec::new();

        if matches!(self.peek_token(), Some(Token::RBrace)) {
            return Ok(layers);
        }

        loop {
            // Expect layer identifier (L0, L1, etc.)
            match self.peek_token() {
                Some(Token::Layer(n)) => {
                    let layer = *n;
                    self.advance();
                    self.expect(Token::Colon)?;
                    let expr = self.parse_expr()?;
                    layers.push((layer, expr));
                }
                _ => return Err(self.error("Expected layer identifier (L0-LF)")),
            }

            if !matches!(self.peek_token(), Some(Token::Comma)) {
                break;
            }
            self.advance();
        }

        Ok(layers)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();

        if matches!(self.peek_token(), Some(Token::RParen)) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expr()?);

            if !matches!(self.peek_token(), Some(Token::Comma)) {
                break;
            }
            self.advance();
        }

        Ok(args)
    }

    // ===== Helper Methods =====

    /// Peek at the current token (without span)
    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|st| &st.token)
    }

    /// Peek at the current spanned token
    #[allow(dead_code)]
    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    /// Get the span of the current token
    fn current_span(&self) -> Span {
        self.tokens.get(self.pos).map(|st| st.span).unwrap_or(Span::dummy())
    }

    /// Get the span of the previous token
    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens.get(self.pos - 1).map(|st| st.span).unwrap_or(Span::dummy())
        } else {
            Span::dummy()
        }
    }

    fn advance(&mut self) -> Option<&SpannedToken> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        match self.peek_token() {
            Some(token) if std::mem::discriminant(token) == std::mem::discriminant(&expected) => {
                self.advance();
                Ok(())
            }
            _ => Err(self.error(&format!("Expected {:?}", expected))),
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        match self.peek_token() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(self.error("Expected identifier")),
        }
    }

    fn error(&self, message: &str) -> Error {
        let span = self.current_span();
        Error::ParseError {
            message: message.to_string(),
            line: span.line,
            col: span.col,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Result<Program> {
        let tokens = Lexer::new(source).tokenize_with_spans()?;
        Parser::new(tokens).parse()
    }

    #[test]
    fn test_parse_empty_function() {
        let source = "fn main() {}";
        let program = parse(source).unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Function { name, params, body, .. } => {
                assert_eq!(name, "main");
                assert_eq!(params.len(), 0);
                assert_eq!(body.len(), 0);
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_function_with_return_type() {
        let source = "fn add(a: Int, b: Int) -> Int { return a; }";
        let program = parse(source).unwrap();
        match &program.items[0] {
            Item::Function { name, params, ret_ty, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert!(matches!(ret_ty, Some(Type::Named(n)) if n == "Int"));
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_let_statement() {
        let source = "fn main() { let x = 42; }";
        let program = parse(source).unwrap();
        match &program.items[0] {
            Item::Function { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Stmt::Let { name, value, .. } => {
                        assert_eq!(name, "x");
                        assert!(matches!(value.kind, ExprKind::Literal(Literal::Int(42))));
                    }
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_binary_expr() {
        let source = "fn main() { let x = 1 + 2 * 3; }";
        let program = parse(source).unwrap();
        match &program.items[0] {
            Item::Function { body, .. } => match &body[0] {
                Stmt::Let { value, .. } => {
                    // Should parse as: 1 + (2 * 3)
                    assert!(matches!(
                        value.kind,
                        ExprKind::Binary {
                            op: BinOp::Add,
                            ..
                        }
                    ));
                }
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let source = "fn main() { sense(); }";
        let program = parse(source).unwrap();
        match &program.items[0] {
            Item::Function { body, .. } => match &body[0] {
                Stmt::Expr(expr) => {
                    assert!(matches!(&expr.kind, ExprKind::Call { name, args } if name == "sense" && args.is_empty()));
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_tuple() {
        let source = "fn main() { let t = (a, b, c); }";
        let program = parse(source).unwrap();
        match &program.items[0] {
            Item::Function { body, .. } => match &body[0] {
                Stmt::Let { value, .. } => {
                    assert!(matches!(&value.kind, ExprKind::Tuple { elements } if elements.len() == 3));
                }
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_complex_number() {
        let source = "fn main() { let c = (1.0, 2.0); }";
        let program = parse(source).unwrap();
        match &program.items[0] {
            Item::Function { body, .. } => match &body[0] {
                Stmt::Let { value, .. } => {
                    assert!(matches!(&value.kind, ExprKind::Complex { .. }));
                }
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }
}
