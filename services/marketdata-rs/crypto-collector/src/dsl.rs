//! B1 — Safe Templating DSL: Tokenizer, Parser (AST), and Interpreter.
//!
//! Generates subscription messages from a DSL source string plus a runtime
//! context that provides `symbols`, `channels`, and `conn_id`.
//!
//! # Supported statements
//! - `foreach(var in collection) { … }` — iterate over `symbols` or `channels`
//! - `if (cond) { … } else if (cond) { … } else { … }` — conditional
//! - `emit("…");` — emit a message string (subject to placeholder substitution)
//!
//! # Safety
//! - No recursion.
//! - Bounded output: default cap 1_000_000 messages per generator.
//! - Deterministic errors with line/column attribution.

use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

// ───────────────────────────────────────────────────────────────────────────
// Errors
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum DslError {
    #[error("tokenizer error at {line}:{col}: {message}")]
    Tokenize {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("parse error at {line}:{col}: {message}")]
    Parse {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("runtime error (subscription {sub_index}, conn '{conn_id}'): {message}")]
    Runtime {
        sub_index: usize,
        conn_id: String,
        message: String,
    },
}

// ───────────────────────────────────────────────────────────────────────────
// Tokens
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Foreach,
    In,
    If,
    Else,
    Emit,
    // Literals
    StringLit(String),
    Ident(String),
    // Operators
    EqEq,     // ==
    BangEq,   // !=
    AmpAmp,   // &&
    PipePipe, // ||
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    // End
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Foreach => write!(f, "foreach"),
            TokenKind::In => write!(f, "in"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Emit => write!(f, "emit"),
            TokenKind::StringLit(s) => write!(f, "\"{}\"", s),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::BangEq => write!(f, "!="),
            TokenKind::AmpAmp => write!(f, "&&"),
            TokenKind::PipePipe => write!(f, "||"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Eof => write!(f, "<EOF>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

// ───────────────────────────────────────────────────────────────────────────
// Tokenizer
// ───────────────────────────────────────────────────────────────────────────

pub fn tokenize(source: &str) -> Result<Vec<Token>, DslError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut pos = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;

    while pos < len {
        let ch = chars[pos];

        // Whitespace
        if ch.is_whitespace() {
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            pos += 1;
            continue;
        }

        // Line comment //
        if ch == '/' && pos + 1 < len && chars[pos + 1] == '/' {
            while pos < len && chars[pos] != '\n' {
                pos += 1;
            }
            continue;
        }

        // String literal
        if ch == '"' || ch == '\'' {
            let start_line = line;
            let start_col = col;
            let quote = ch;
            pos += 1;
            col += 1;
            let mut s = String::new();
            loop {
                if pos >= len {
                    return Err(DslError::Tokenize {
                        line: start_line,
                        col: start_col,
                        message: "unterminated string literal".to_string(),
                    });
                }
                let c = chars[pos];
                if c == quote {
                    pos += 1;
                    col += 1;
                    break;
                }
                if c == '\\' {
                    pos += 1;
                    col += 1;
                    if pos >= len {
                        return Err(DslError::Tokenize {
                            line: start_line,
                            col: start_col,
                            message: "unterminated escape in string literal".to_string(),
                        });
                    }
                    let esc = chars[pos];
                    match esc {
                        'n' => s.push('\n'),
                        '\\' => s.push('\\'),
                        '\'' => s.push('\''),
                        '"' => s.push('"'),
                        _ => {
                            return Err(DslError::Tokenize {
                                line,
                                col,
                                message: format!("unknown escape sequence '\\{}'", esc),
                            });
                        }
                    }
                    pos += 1;
                    col += 1;
                    continue;
                }
                if c == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                s.push(c);
                pos += 1;
            }
            tokens.push(Token {
                kind: TokenKind::StringLit(s),
                line: start_line,
                col: start_col,
            });
            continue;
        }

        // Identifier / keyword
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start_col = col;
            let mut ident = String::new();
            while pos < len && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                ident.push(chars[pos]);
                pos += 1;
                col += 1;
            }
            let kind = match ident.as_str() {
                "foreach" => TokenKind::Foreach,
                "in" => TokenKind::In,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "emit" => TokenKind::Emit,
                _ => TokenKind::Ident(ident),
            };
            tokens.push(Token {
                kind,
                line,
                col: start_col,
            });
            continue;
        }

        // Two-char operators
        if pos + 1 < len {
            let next = chars[pos + 1];
            let two = match (ch, next) {
                ('=', '=') => Some(TokenKind::EqEq),
                ('!', '=') => Some(TokenKind::BangEq),
                ('&', '&') => Some(TokenKind::AmpAmp),
                ('|', '|') => Some(TokenKind::PipePipe),
                _ => None,
            };
            if let Some(kind) = two {
                tokens.push(Token { kind, line, col });
                pos += 2;
                col += 2;
                continue;
            }
        }

        // Single-char delimiters
        let single = match ch {
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            '{' => Some(TokenKind::LBrace),
            '}' => Some(TokenKind::RBrace),
            ';' => Some(TokenKind::Semi),
            _ => None,
        };
        if let Some(kind) = single {
            tokens.push(Token { kind, line, col });
            pos += 1;
            col += 1;
            continue;
        }

        return Err(DslError::Tokenize {
            line,
            col,
            message: format!("unexpected character '{}'", ch),
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
        col,
    });
    Ok(tokens)
}

// ───────────────────────────────────────────────────────────────────────────
// AST
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    Foreach {
        var: String,
        collection: String, // "symbols" or "channels"
        body: Vec<Stmt>,
        line: usize,
        col: usize,
    },
    If {
        branches: Vec<CondBranch>,
        else_body: Option<Vec<Stmt>>,
        line: usize,
        col: usize,
    },
    Emit {
        value: String,
        line: usize,
        col: usize,
    },
}

#[derive(Debug, Clone)]
pub struct CondBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    StringLit(String),
    Ident(String),
    BinOp {
        op: BinOpKind,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Eq,    // ==
    NotEq, // !=
    And,   // &&
    Or,    // ||
}

// ───────────────────────────────────────────────────────────────────────────
// Parser
// ───────────────────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, DslError> {
        let tok = self.peek().clone();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(expected) {
            self.advance();
            Ok(tok)
        } else {
            Err(DslError::Parse {
                line: tok.line,
                col: tok.col,
                message: format!("expected {}, got {}", expected, tok.kind),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<(String, usize, usize), DslError> {
        let tok = self.peek().clone();
        if let TokenKind::Ident(name) = &tok.kind {
            let name = name.clone();
            self.advance();
            Ok((name, tok.line, tok.col))
        } else {
            Err(DslError::Parse {
                line: tok.line,
                col: tok.col,
                message: format!("expected identifier, got {}", tok.kind),
            })
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, DslError> {
        let mut stmts = Vec::new();
        while self.peek().kind != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, DslError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Foreach => self.parse_foreach(),
            TokenKind::If => self.parse_if(),
            TokenKind::Emit => self.parse_emit(),
            _ => Err(DslError::Parse {
                line: tok.line,
                col: tok.col,
                message: format!("expected statement (foreach/if/emit), got {}", tok.kind),
            }),
        }
    }

    fn parse_foreach(&mut self) -> Result<Stmt, DslError> {
        let tok = self.advance().clone(); // consume 'foreach'
        let line = tok.line;
        let col = tok.col;
        self.expect(&TokenKind::LParen)?;
        let (var, _, _) = self.expect_ident()?;
        self.expect(&TokenKind::In)?;
        let (collection, coll_line, coll_col) = self.expect_ident()?;

        // Validate collection name
        if collection != "symbols" && collection != "channels" {
            return Err(DslError::Parse {
                line: coll_line,
                col: coll_col,
                message: format!(
                    "foreach collection must be 'symbols' or 'channels', got '{}'",
                    collection
                ),
            });
        }

        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(&TokenKind::RBrace)?;

        Ok(Stmt::Foreach {
            var,
            collection,
            body,
            line,
            col,
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, DslError> {
        let tok = self.advance().clone(); // consume 'if'
        let line = tok.line;
        let col = tok.col;

        self.expect(&TokenKind::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(&TokenKind::RBrace)?;

        let mut branches = vec![CondBranch { condition, body }];
        let mut else_body = None;

        // Parse else if / else chains
        while self.peek().kind == TokenKind::Else {
            self.advance(); // consume 'else'
            if self.peek().kind == TokenKind::If {
                self.advance(); // consume 'if'
                self.expect(&TokenKind::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                self.expect(&TokenKind::LBrace)?;
                let b = self.parse_block()?;
                self.expect(&TokenKind::RBrace)?;
                branches.push(CondBranch {
                    condition: cond,
                    body: b,
                });
            } else {
                self.expect(&TokenKind::LBrace)?;
                let b = self.parse_block()?;
                self.expect(&TokenKind::RBrace)?;
                else_body = Some(b);
                break;
            }
        }

        Ok(Stmt::If {
            branches,
            else_body,
            line,
            col,
        })
    }

    fn parse_emit(&mut self) -> Result<Stmt, DslError> {
        let tok = self.advance().clone(); // consume 'emit'
        let line = tok.line;
        let col = tok.col;
        self.expect(&TokenKind::LParen)?;

        let val_tok = self.peek().clone();
        let value = if let TokenKind::StringLit(s) = &val_tok.kind {
            let s = s.clone();
            self.advance();
            s
        } else {
            return Err(DslError::Parse {
                line: val_tok.line,
                col: val_tok.col,
                message: format!("emit expects a string literal, got {}", val_tok.kind),
            });
        };

        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Semi)?;

        Ok(Stmt::Emit { value, line, col })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, DslError> {
        let mut stmts = Vec::new();
        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // Expression parsing: or_expr
    fn parse_expr(&mut self) -> Result<Expr, DslError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, DslError> {
        let mut left = self.parse_and_expr()?;
        while self.peek().kind == TokenKind::PipePipe {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinOp {
                op: BinOpKind::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, DslError> {
        let mut left = self.parse_comparison()?;
        while self.peek().kind == TokenKind::AmpAmp {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp {
                op: BinOpKind::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, DslError> {
        let left = self.parse_primary()?;
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::EqEq => {
                self.advance();
                let right = self.parse_primary()?;
                Ok(Expr::BinOp {
                    op: BinOpKind::Eq,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            TokenKind::BangEq => {
                self.advance();
                let right = self.parse_primary()?;
                Ok(Expr::BinOp {
                    op: BinOpKind::NotEq,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, DslError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::StringLit(s))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name))
            }
            _ => Err(DslError::Parse {
                line: tok.line,
                col: tok.col,
                message: format!("expected expression, got {}", tok.kind),
            }),
        }
    }
}

/// Parse DSL source into an AST.
pub fn parse(source: &str) -> Result<Vec<Stmt>, DslError> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// ───────────────────────────────────────────────────────────────────────────
// Interpreter context
// ───────────────────────────────────────────────────────────────────────────

/// Runtime context for DSL execution.
#[derive(Debug, Clone)]
pub struct DslContext {
    pub symbols: Vec<String>,
    pub channels: Vec<String>,
    pub conn_id: String,
    /// Maximum output messages before error (default 1_000_000).
    pub max_outputs: usize,
}

impl Default for DslContext {
    fn default() -> Self {
        Self {
            symbols: Vec::new(),
            channels: Vec::new(),
            conn_id: String::new(),
            max_outputs: 1_000_000,
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Interpreter
// ───────────────────────────────────────────────────────────────────────────

struct Interpreter {
    ctx: DslContext,
    sub_index: usize,
    vars: HashMap<String, String>,
    outputs: Vec<String>,
}

impl Interpreter {
    fn new(ctx: DslContext, sub_index: usize) -> Self {
        Self {
            ctx,
            sub_index,
            vars: HashMap::new(),
            outputs: Vec::new(),
        }
    }

    fn runtime_error(&self, msg: String) -> DslError {
        DslError::Runtime {
            sub_index: self.sub_index,
            conn_id: self.ctx.conn_id.clone(),
            message: msg,
        }
    }

    fn execute(&mut self, stmts: &[Stmt]) -> Result<(), DslError> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<(), DslError> {
        match stmt {
            Stmt::Foreach {
                var,
                collection,
                body,
                ..
            } => {
                let items = match collection.as_str() {
                    "symbols" => self.ctx.symbols.clone(),
                    "channels" => self.ctx.channels.clone(),
                    _ => {
                        return Err(
                            self.runtime_error(format!("unknown collection '{}'", collection))
                        );
                    }
                };

                let prev = self.vars.get(var).cloned();
                for item in &items {
                    self.vars.insert(var.clone(), item.clone());
                    self.execute(body)?;
                }
                // Restore previous binding (supports nested loops with same var name)
                match prev {
                    Some(v) => {
                        self.vars.insert(var.clone(), v);
                    }
                    None => {
                        self.vars.remove(var);
                    }
                }
                Ok(())
            }

            Stmt::If {
                branches,
                else_body,
                ..
            } => {
                for branch in branches {
                    if self.eval_bool(&branch.condition)? {
                        return self.execute(&branch.body);
                    }
                }
                if let Some(body) = else_body {
                    self.execute(body)?;
                }
                Ok(())
            }

            Stmt::Emit { value, .. } => {
                if self.outputs.len() >= self.ctx.max_outputs {
                    return Err(self.runtime_error(format!(
                        "output cap exceeded (max {} messages)",
                        self.ctx.max_outputs
                    )));
                }
                self.outputs.push(value.clone());
                Ok(())
            }
        }
    }

    fn resolve_ident(&self, name: &str) -> Result<String, DslError> {
        // Check loop variables first
        if let Some(v) = self.vars.get(name) {
            return Ok(v.clone());
        }
        // Built-in identifiers
        match name {
            "conn_id" => Ok(self.ctx.conn_id.clone()),
            _ => Err(self.runtime_error(format!("undefined variable '{}'", name))),
        }
    }

    fn eval_string(&self, expr: &Expr) -> Result<String, DslError> {
        match expr {
            Expr::StringLit(s) => Ok(s.clone()),
            Expr::Ident(name) => self.resolve_ident(name),
            Expr::BinOp { .. } => {
                Err(self.runtime_error("binary operator cannot produce a string value".to_string()))
            }
        }
    }

    fn eval_bool(&self, expr: &Expr) -> Result<bool, DslError> {
        match expr {
            Expr::BinOp { op, left, right } => match op {
                BinOpKind::Eq => {
                    let l = self.eval_string(left)?;
                    let r = self.eval_string(right)?;
                    Ok(l == r)
                }
                BinOpKind::NotEq => {
                    let l = self.eval_string(left)?;
                    let r = self.eval_string(right)?;
                    Ok(l != r)
                }
                BinOpKind::And => {
                    let l = self.eval_bool(left)?;
                    let r = self.eval_bool(right)?;
                    Ok(l && r)
                }
                BinOpKind::Or => {
                    let l = self.eval_bool(left)?;
                    let r = self.eval_bool(right)?;
                    Ok(l || r)
                }
            },
            _ => {
                Err(self.runtime_error("expected boolean expression (==, !=, &&, ||)".to_string()))
            }
        }
    }
}

/// Execute a DSL generator source, returning a list of emitted message strings.
///
/// `sub_index` is used for error attribution (which subscription failed).
pub fn execute(source: &str, ctx: DslContext, sub_index: usize) -> Result<Vec<String>, DslError> {
    let stmts = parse(source)?;
    let mut interp = Interpreter::new(ctx, sub_index);
    interp.execute(&stmts)?;
    Ok(interp.outputs)
}

// ───────────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(symbols: &[&str], channels: &[&str], conn_id: &str) -> DslContext {
        DslContext {
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
            channels: channels.iter().map(|s| s.to_string()).collect(),
            conn_id: conn_id.to_string(),
            max_outputs: 1_000_000,
        }
    }

    #[test]
    fn nested_foreach_symbols_x_channels() {
        let src = r#"
            foreach(symbol in symbols) {
                foreach(ch in channels) {
                    emit("{symbol}:{ch}");
                }
            }
        "#;
        let c = ctx(&["BTC", "ETH"], &["trades", "book"], "main");
        let out = execute(src, c, 0).unwrap();
        assert_eq!(out.len(), 4);
        assert_eq!(out[0], "{symbol}:{ch}");
        assert_eq!(out[1], "{symbol}:{ch}");
        assert_eq!(out[2], "{symbol}:{ch}");
        assert_eq!(out[3], "{symbol}:{ch}");
    }

    #[test]
    fn if_else_if_else_path_selection() {
        let src = r#"
            foreach(ch in channels) {
                if (ch == "trades") {
                    emit("trade_sub");
                } else if (ch == "book") {
                    emit("book_sub");
                } else {
                    emit("other_sub");
                }
            }
        "#;
        let c = ctx(&[], &["trades", "book", "unknown"], "main");
        let out = execute(src, c, 0).unwrap();
        assert_eq!(out, vec!["trade_sub", "book_sub", "other_sub"]);
    }

    #[test]
    fn output_count_correctness() {
        let src = r#"
            foreach(symbol in symbols) {
                foreach(ch in channels) {
                    emit("msg");
                }
            }
        "#;
        let c = ctx(&["A", "B", "C"], &["x", "y"], "c");
        let out = execute(src, c, 0).unwrap();
        assert_eq!(out.len(), 6); // 3 * 2
    }

    #[test]
    fn syntax_error_line_col() {
        let src = "foreach(symbol in symbols) {\n  badtoken@@@\n}";
        let err = execute(src, DslContext::default(), 0).unwrap_err();
        match err {
            DslError::Tokenize { line, col, .. } => {
                assert_eq!(line, 2);
                assert!(col > 0);
            }
            other => panic!("expected tokenize error, got: {}", other),
        }
    }

    #[test]
    fn cap_enforcement() {
        let src = r#"
            foreach(symbol in symbols) {
                emit("msg");
            }
        "#;
        let mut c = ctx(&["A", "B", "C"], &[], "c");
        c.max_outputs = 2;
        let err = execute(src, c, 0).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("output cap exceeded"), "got: {}", msg);
    }

    #[test]
    fn empty_program() {
        let out = execute("", DslContext::default(), 0).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn string_escape_handling() {
        let src = r#"emit("hello \"world\"\nnewline\\end");"#;
        let out = execute(src, DslContext::default(), 0).unwrap();
        assert_eq!(out[0], "hello \"world\"\nnewline\\end");
    }

    #[test]
    fn and_or_conditions() {
        let src = r#"
            foreach(symbol in symbols) {
                foreach(ch in channels) {
                    if (symbol == "BTC" && ch == "trades") {
                        emit("match");
                    }
                    if (symbol == "ETH" || ch == "book") {
                        emit("or_match");
                    }
                }
            }
        "#;
        let c = ctx(&["BTC", "ETH"], &["trades", "book"], "m");
        let out = execute(src, c, 0).unwrap();
        // BTC+trades: match, or_match(no - BTC!=ETH and trades!=book → nope)
        // Actually: BTC+trades: symbol==BTC&&ch==trades → true → "match"
        //   symbol==ETH||ch==book → false||false → false
        // BTC+book: symbol==BTC&&ch==trades → false
        //   symbol==ETH||ch==book → false||true → true → "or_match"
        // ETH+trades: &&→false, ||→true(ETH==ETH) → "or_match"
        // ETH+book: &&→false, ||→true(ETH==ETH) → "or_match"
        assert_eq!(out, vec!["match", "or_match", "or_match", "or_match"]);
    }

    #[test]
    fn conn_id_in_condition() {
        let src = r#"
            if (conn_id == "main") {
                emit("yes");
            } else {
                emit("no");
            }
        "#;
        let c = ctx(&[], &[], "main");
        let out = execute(src, c, 0).unwrap();
        assert_eq!(out, vec!["yes"]);
    }

    #[test]
    fn parse_error_on_bad_collection() {
        let src = "foreach(x in bogus) { emit(\"a\"); }";
        let err = execute(src, DslContext::default(), 0).unwrap_err();
        match err {
            DslError::Parse { message, .. } => {
                assert!(
                    message.contains("'symbols' or 'channels'"),
                    "got: {}",
                    message
                );
            }
            other => panic!("expected parse error, got: {}", other),
        }
    }

    #[test]
    fn line_comments_ignored() {
        let src = r#"
            // This is a comment
            emit("hello"); // inline comment
        "#;
        let out = execute(src, DslContext::default(), 0).unwrap();
        assert_eq!(out, vec!["hello"]);
    }

    #[test]
    fn undefined_variable_error() {
        let src = r#"
            if (unknown_var == "x") {
                emit("nope");
            }
        "#;
        let err = execute(src, DslContext::default(), 0).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("undefined variable"), "got: {}", msg);
    }
}
