//! Safe templating DSL — tokenizer, parser (AST), and bounded interpreter.
//!
//! Grammar (supported subset):
//! ```text
//! program     = statement*
//! statement   = foreach_stmt | if_stmt | emit_stmt
//! foreach_stmt = "foreach" "(" IDENT "in" IDENT ")" "{" statement* "}"
//! if_stmt     = "if" "(" condition ")" "{" statement* "}"
//!               ("else" "if" "(" condition ")" "{" statement* "}")*
//!               ("else" "{" statement* "}")?
//! emit_stmt   = "emit" "(" string_literal ")" ";"
//! condition   = or_expr
//! or_expr     = and_expr ("||" and_expr)*
//! and_expr    = cmp_expr ("&&" cmp_expr)*
//! cmp_expr    = "(" condition ")" | operand ("==" | "!=") operand
//! operand     = IDENT | string_literal
//! ```
//!
//! Safety: no recursion in execution, bounded output, deterministic errors
//! with line/column.

use crate::placeholder::{self, PlaceholderContext};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DslError {
    #[error("DSL syntax error at {line}:{col}: {message}")]
    Syntax {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("DSL runtime error (subscription {sub_index}, connection '{conn_id}'): {message}")]
    Runtime {
        sub_index: usize,
        conn_id: String,
        message: String,
    },

    #[error("DSL output limit exceeded: generated {count} messages (max {max})")]
    OutputLimitExceeded { count: usize, max: usize },

    #[error("placeholder error in emit: {0}")]
    Placeholder(#[from] placeholder::PlaceholderError),
}

// ---------------------------------------------------------------------------
// Tokens
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Foreach,
    In,
    If,
    Else,
    Emit,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Eq,  // ==
    Neq, // !=
    And, // &&
    Or,  // ||
    Ident(String),
    StringLit(String),
}

#[derive(Debug, Clone)]
struct Located<T> {
    val: T,
    line: usize,
    col: usize,
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

fn tokenize(source: &str) -> Result<Vec<Located<Token>>, DslError> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut line = 1usize;
    let mut col = 1usize;

    while i < len {
        // Skip whitespace
        if bytes[i].is_ascii_whitespace() {
            if bytes[i] == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            i += 1;
            continue;
        }

        // Skip line comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        let start_line = line;
        let start_col = col;

        match bytes[i] {
            b'(' => {
                tokens.push(Located {
                    val: Token::LParen,
                    line: start_line,
                    col: start_col,
                });
                i += 1;
                col += 1;
            }
            b')' => {
                tokens.push(Located {
                    val: Token::RParen,
                    line: start_line,
                    col: start_col,
                });
                i += 1;
                col += 1;
            }
            b'{' => {
                tokens.push(Located {
                    val: Token::LBrace,
                    line: start_line,
                    col: start_col,
                });
                i += 1;
                col += 1;
            }
            b'}' => {
                tokens.push(Located {
                    val: Token::RBrace,
                    line: start_line,
                    col: start_col,
                });
                i += 1;
                col += 1;
            }
            b';' => {
                tokens.push(Located {
                    val: Token::Semi,
                    line: start_line,
                    col: start_col,
                });
                i += 1;
                col += 1;
            }
            b'=' if i + 1 < len && bytes[i + 1] == b'=' => {
                tokens.push(Located {
                    val: Token::Eq,
                    line: start_line,
                    col: start_col,
                });
                i += 2;
                col += 2;
            }
            b'!' if i + 1 < len && bytes[i + 1] == b'=' => {
                tokens.push(Located {
                    val: Token::Neq,
                    line: start_line,
                    col: start_col,
                });
                i += 2;
                col += 2;
            }
            b'&' if i + 1 < len && bytes[i + 1] == b'&' => {
                tokens.push(Located {
                    val: Token::And,
                    line: start_line,
                    col: start_col,
                });
                i += 2;
                col += 2;
            }
            b'|' if i + 1 < len && bytes[i + 1] == b'|' => {
                tokens.push(Located {
                    val: Token::Or,
                    line: start_line,
                    col: start_col,
                });
                i += 2;
                col += 2;
            }
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                col += 1;
                let mut s = String::new();
                loop {
                    if i >= len {
                        return Err(DslError::Syntax {
                            line: start_line,
                            col: start_col,
                            message: "unterminated string literal".to_string(),
                        });
                    }
                    if bytes[i] == b'\\' && i + 1 < len {
                        match bytes[i + 1] {
                            b'n' => s.push('\n'),
                            b'\\' => s.push('\\'),
                            b'\'' => s.push('\''),
                            b'"' => s.push('"'),
                            other => {
                                s.push('\\');
                                s.push(other as char);
                            }
                        }
                        i += 2;
                        col += 2;
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        col += 1;
                        break;
                    }
                    s.push(bytes[i] as char);
                    if bytes[i] == b'\n' {
                        line += 1;
                        col = 1;
                    } else {
                        col += 1;
                    }
                    i += 1;
                }
                tokens.push(Located {
                    val: Token::StringLit(s),
                    line: start_line,
                    col: start_col,
                });
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                    col += 1;
                }
                let word = &source[start..i];
                let tok = match word {
                    "foreach" => Token::Foreach,
                    "in" => Token::In,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "emit" => Token::Emit,
                    _ => Token::Ident(word.to_string()),
                };
                tokens.push(Located {
                    val: tok,
                    line: start_line,
                    col: start_col,
                });
            }
            _ => {
                return Err(DslError::Syntax {
                    line: start_line,
                    col: start_col,
                    message: format!("unexpected character '{}'", bytes[i] as char),
                });
            }
        }
    }

    Ok(tokens)
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Stmt {
    Foreach {
        var: String,
        collection: String,
        body: Vec<Stmt>,
    },
    If {
        branches: Vec<(Condition, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    Emit(String),
}

#[derive(Debug, Clone)]
enum Condition {
    Compare {
        left: Operand,
        op: CmpOp,
        right: Operand,
    },
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CmpOp {
    Eq,
    Neq,
}

#[derive(Debug, Clone)]
enum Operand {
    Ident(String),
    Literal(String),
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    tokens: Vec<Located<Token>>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Located<Token>>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.val)
    }

    fn loc(&self) -> (usize, usize) {
        self.tokens
            .get(self.pos)
            .map(|t| (t.line, t.col))
            .unwrap_or((0, 0))
    }

    fn expect(&mut self, expected: &Token) -> Result<(), DslError> {
        let (line, col) = self.loc();
        match self.peek() {
            Some(t) if t == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(t) => Err(DslError::Syntax {
                line,
                col,
                message: format!("expected {expected:?}, found {t:?}"),
            }),
            None => Err(DslError::Syntax {
                line,
                col,
                message: format!("expected {expected:?}, found end of input"),
            }),
        }
    }

    fn advance(&mut self) -> Option<&Located<Token>> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn parse_program(&mut self) -> Result<Vec<Stmt>, DslError> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Stmt, DslError> {
        let (line, col) = self.loc();
        match self.peek() {
            Some(Token::Foreach) => self.parse_foreach(),
            Some(Token::If) => self.parse_if(),
            Some(Token::Emit) => self.parse_emit(),
            Some(t) => Err(DslError::Syntax {
                line,
                col,
                message: format!("expected 'foreach', 'if', or 'emit', found {t:?}"),
            }),
            None => Err(DslError::Syntax {
                line,
                col,
                message: "unexpected end of input".to_string(),
            }),
        }
    }

    fn parse_foreach(&mut self) -> Result<Stmt, DslError> {
        self.expect(&Token::Foreach)?;
        self.expect(&Token::LParen)?;
        let var = self.expect_ident()?;
        self.expect(&Token::In)?;
        let collection = self.expect_ident()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::Foreach {
            var,
            collection,
            body,
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, DslError> {
        self.expect(&Token::If)?;
        self.expect(&Token::LParen)?;
        let cond = self.parse_condition()?;
        self.expect(&Token::RParen)?;
        let body = self.parse_block()?;

        let mut branches = vec![(cond, body)];
        let mut else_body = None;

        while self.peek() == Some(&Token::Else) {
            self.pos += 1; // consume 'else'
            if self.peek() == Some(&Token::If) {
                self.pos += 1; // consume 'if'
                self.expect(&Token::LParen)?;
                let cond = self.parse_condition()?;
                self.expect(&Token::RParen)?;
                let body = self.parse_block()?;
                branches.push((cond, body));
            } else {
                else_body = Some(self.parse_block()?);
                break;
            }
        }

        Ok(Stmt::If {
            branches,
            else_body,
        })
    }

    fn parse_emit(&mut self) -> Result<Stmt, DslError> {
        self.expect(&Token::Emit)?;
        self.expect(&Token::LParen)?;
        let (line, col) = self.loc();
        let s = match self.advance() {
            Some(Located {
                val: Token::StringLit(s),
                ..
            }) => s.clone(),
            _ => {
                return Err(DslError::Syntax {
                    line,
                    col,
                    message: "emit expects a string literal argument".to_string(),
                })
            }
        };
        self.expect(&Token::RParen)?;
        self.expect(&Token::Semi)?;
        Ok(Stmt::Emit(s))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, DslError> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            if self.at_end() {
                let (line, col) = self.loc();
                return Err(DslError::Syntax {
                    line,
                    col,
                    message: "unclosed block (missing '}')".to_string(),
                });
            }
            stmts.push(self.parse_statement()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_condition(&mut self) -> Result<Condition, DslError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Condition, DslError> {
        let mut left = self.parse_and_expr()?;
        while self.peek() == Some(&Token::Or) {
            self.pos += 1;
            let right = self.parse_and_expr()?;
            left = Condition::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Condition, DslError> {
        let mut left = self.parse_cmp_expr()?;
        while self.peek() == Some(&Token::And) {
            self.pos += 1;
            let right = self.parse_cmp_expr()?;
            left = Condition::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_cmp_expr(&mut self) -> Result<Condition, DslError> {
        if self.peek() == Some(&Token::LParen) {
            self.pos += 1;
            let cond = self.parse_condition()?;
            self.expect(&Token::RParen)?;
            return Ok(cond);
        }
        let left = self.parse_operand()?;
        let (line, col) = self.loc();
        let op = match self.peek() {
            Some(Token::Eq) => CmpOp::Eq,
            Some(Token::Neq) => CmpOp::Neq,
            _ => {
                return Err(DslError::Syntax {
                    line,
                    col,
                    message: "expected '==' or '!=' in condition".to_string(),
                })
            }
        };
        self.pos += 1;
        let right = self.parse_operand()?;
        Ok(Condition::Compare { left, op, right })
    }

    fn parse_operand(&mut self) -> Result<Operand, DslError> {
        let (line, col) = self.loc();
        match self.advance() {
            Some(Located {
                val: Token::Ident(s),
                ..
            }) => Ok(Operand::Ident(s.clone())),
            Some(Located {
                val: Token::StringLit(s),
                ..
            }) => Ok(Operand::Literal(s.clone())),
            _ => Err(DslError::Syntax {
                line,
                col,
                message: "expected identifier or string literal".to_string(),
            }),
        }
    }

    fn expect_ident(&mut self) -> Result<String, DslError> {
        let (line, col) = self.loc();
        match self.advance() {
            Some(Located {
                val: Token::Ident(s),
                ..
            }) => Ok(s.clone()),
            _ => Err(DslError::Syntax {
                line,
                col,
                message: "expected identifier".to_string(),
            }),
        }
    }
}

/// Parse DSL source into an AST.
fn parse(source: &str) -> Result<Vec<Stmt>, DslError> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// ---------------------------------------------------------------------------
// Interpreter
// ---------------------------------------------------------------------------

/// Runtime context for DSL execution.
pub struct DslContext {
    pub symbols: Vec<String>,
    pub channels: Vec<String>,
    pub conn_id: String,
    pub args: std::collections::HashMap<String, String>,
    pub max_outputs: usize,
}

impl Default for DslContext {
    fn default() -> Self {
        Self {
            symbols: Vec::new(),
            channels: Vec::new(),
            conn_id: String::new(),
            args: std::collections::HashMap::new(),
            max_outputs: 1_000_000,
        }
    }
}

struct Interpreter<'a> {
    ctx: &'a DslContext,
    outputs: Vec<String>,
    /// Scoped variables: name -> value
    vars: std::collections::HashMap<String, String>,
    sub_index: usize,
}

impl<'a> Interpreter<'a> {
    fn new(ctx: &'a DslContext, sub_index: usize) -> Self {
        Self {
            ctx,
            outputs: Vec::new(),
            vars: std::collections::HashMap::new(),
            sub_index,
        }
    }

    fn run(&mut self, stmts: &[Stmt]) -> Result<(), DslError> {
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
            } => {
                let items = match collection.as_str() {
                    "symbols" => self.ctx.symbols.clone(),
                    "channels" => self.ctx.channels.clone(),
                    other => {
                        return Err(DslError::Runtime {
                            sub_index: self.sub_index,
                            conn_id: self.ctx.conn_id.clone(),
                            message: format!(
                                "unknown collection '{other}' (allowed: symbols, channels)"
                            ),
                        });
                    }
                };
                let prev = self.vars.get(var).cloned();
                for item in &items {
                    self.vars.insert(var.clone(), item.clone());
                    self.run(body)?;
                }
                // Restore previous value (scope cleanup)
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
            } => {
                for (cond, body) in branches {
                    if self.eval_condition(cond)? {
                        return self.run(body);
                    }
                }
                if let Some(body) = else_body {
                    self.run(body)?;
                }
                Ok(())
            }
            Stmt::Emit(template) => {
                if self.outputs.len() >= self.ctx.max_outputs {
                    return Err(DslError::OutputLimitExceeded {
                        count: self.outputs.len(),
                        max: self.ctx.max_outputs,
                    });
                }
                let pctx = PlaceholderContext {
                    symbol: self
                        .vars
                        .get("symbol")
                        .or_else(|| self.vars.get("s"))
                        .cloned(),
                    channel: self
                        .vars
                        .get("ch")
                        .or_else(|| self.vars.get("channel"))
                        .cloned(),
                    conn_id: Some(self.ctx.conn_id.clone()),
                    args: self.ctx.args.clone(),
                };
                let msg = placeholder::substitute(template, &pctx)?;
                self.outputs.push(msg);
                Ok(())
            }
        }
    }

    fn eval_condition(&self, cond: &Condition) -> Result<bool, DslError> {
        match cond {
            Condition::Compare { left, op, right } => {
                let l = self.resolve_operand(left)?;
                let r = self.resolve_operand(right)?;
                Ok(match op {
                    CmpOp::Eq => l == r,
                    CmpOp::Neq => l != r,
                })
            }
            Condition::And(a, b) => Ok(self.eval_condition(a)? && self.eval_condition(b)?),
            Condition::Or(a, b) => Ok(self.eval_condition(a)? || self.eval_condition(b)?),
        }
    }

    fn resolve_operand(&self, op: &Operand) -> Result<String, DslError> {
        match op {
            Operand::Literal(s) => Ok(s.clone()),
            Operand::Ident(name) => {
                // Check loop vars first, then built-in identifiers
                if let Some(v) = self.vars.get(name) {
                    return Ok(v.clone());
                }
                match name.as_str() {
                    "conn_id" => Ok(self.ctx.conn_id.clone()),
                    _ => Err(DslError::Runtime {
                        sub_index: self.sub_index,
                        conn_id: self.ctx.conn_id.clone(),
                        message: format!("undefined identifier '{name}' in condition"),
                    }),
                }
            }
        }
    }
}

/// Execute a DSL generator source and return the generated messages.
///
/// - `source`: DSL source code string
/// - `ctx`: runtime context (symbols, channels, conn_id)
/// - `sub_index`: subscription index for error attribution
pub fn execute(source: &str, ctx: &DslContext, sub_index: usize) -> Result<Vec<String>, DslError> {
    let stmts = parse(source)?;
    let mut interp = Interpreter::new(ctx, sub_index);
    interp.run(&stmts)?;
    Ok(interp.outputs)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_ctx() -> DslContext {
        DslContext {
            symbols: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            channels: vec!["trades".to_string(), "orderbook".to_string()],
            conn_id: "main".to_string(),
            args: std::collections::HashMap::new(),
            max_outputs: 1_000_000,
        }
    }

    #[test]
    fn simple_emit() {
        let src = r#"emit("hello");"#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["hello"]);
    }

    #[test]
    fn foreach_symbols() {
        let src = r#"
            foreach(symbol in symbols) {
                emit("{symbol}");
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["BTC/USDT", "ETH/USDT"]);
    }

    #[test]
    fn foreach_channels() {
        let src = r#"
            foreach(ch in channels) {
                emit("{ch}");
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["trades", "orderbook"]);
    }

    #[test]
    fn nested_foreach() {
        let src = r#"
            foreach(symbol in symbols) {
                foreach(ch in channels) {
                    emit("{symbol}:{ch}");
                }
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(
            msgs,
            vec![
                "BTC/USDT:trades",
                "BTC/USDT:orderbook",
                "ETH/USDT:trades",
                "ETH/USDT:orderbook"
            ]
        );
    }

    #[test]
    fn if_else_branching() {
        let src = r#"
            foreach(ch in channels) {
                if (ch == "trades") {
                    emit("TRADE:{channel}");
                } else if (ch == "orderbook") {
                    emit("BOOK:{channel}");
                } else {
                    emit("OTHER:{channel}");
                }
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["TRADE:trades", "BOOK:orderbook"]);
    }

    #[test]
    fn if_with_neq() {
        let src = r#"
            foreach(symbol in symbols) {
                if (symbol != "BTC/USDT") {
                    emit("{symbol}");
                }
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["ETH/USDT"]);
    }

    #[test]
    fn if_with_and_or() {
        let src = r#"
            foreach(symbol in symbols) {
                foreach(ch in channels) {
                    if (symbol == "BTC/USDT" && ch == "trades") {
                        emit("match:{symbol}:{ch}");
                    }
                }
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["match:BTC/USDT:trades"]);
    }

    #[test]
    fn conn_id_in_condition() {
        let src = r#"
            if (conn_id == "main") {
                emit("on_main");
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["on_main"]);
    }

    #[test]
    fn output_count_correctness() {
        // 2 symbols * 2 channels = 4 messages
        let src = r#"
            foreach(symbol in symbols) {
                foreach(ch in channels) {
                    emit("msg");
                }
            }
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs.len(), 4);
    }

    #[test]
    fn output_cap_enforcement() {
        let src = r#"
            foreach(symbol in symbols) {
                emit("{symbol}");
            }
        "#;
        let mut ctx = default_ctx();
        ctx.max_outputs = 1; // Allow only 1 message
        let err = execute(src, &ctx, 0).unwrap_err();
        assert!(
            matches!(err, DslError::OutputLimitExceeded { .. }),
            "got: {err}"
        );
    }

    #[test]
    fn syntax_error_line_col() {
        // The tokenizer rejects '4' as an unexpected character (DSL has no numeric tokens).
        let src = "emit(42);";
        let err = execute(src, &default_ctx(), 0).unwrap_err();
        match &err {
            DslError::Syntax { line, col, message } => {
                assert_eq!(*line, 1);
                assert!(*col > 0);
                assert!(
                    message.contains("unexpected character"),
                    "message should report unexpected char: {message}"
                );
            }
            _ => panic!("expected Syntax error, got: {err}"),
        }
    }

    #[test]
    fn syntax_error_unterminated_string() {
        let src = r#"emit("hello);"#;
        let err = execute(src, &default_ctx(), 0).unwrap_err();
        assert!(err.to_string().contains("unterminated"));
    }

    #[test]
    fn syntax_error_missing_semicolon() {
        let src = r#"emit("hello")"#;
        let err = execute(src, &default_ctx(), 0).unwrap_err();
        match &err {
            DslError::Syntax { message, .. } => {
                assert!(message.contains("Semi"), "got: {message}");
            }
            _ => panic!("expected Syntax error, got: {err}"),
        }
    }

    #[test]
    fn unknown_collection_error() {
        let src = r#"
            foreach(x in unknown) {
                emit("x");
            }
        "#;
        let err = execute(src, &default_ctx(), 0).unwrap_err();
        assert!(err.to_string().contains("unknown collection"));
    }

    #[test]
    fn escape_sequences() {
        let src = r#"emit("line1\nline2\\end\"quoted");"#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs[0], "line1\nline2\\end\"quoted");
    }

    #[test]
    fn single_quoted_string() {
        let src = "emit('hello \"world\"');";
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs[0], "hello \"world\"");
    }

    #[test]
    fn line_comments_ignored() {
        let src = r#"
            // This is a comment
            emit("hello"); // inline comment
        "#;
        let msgs = execute(src, &default_ctx(), 0).unwrap();
        assert_eq!(msgs, vec!["hello"]);
    }

    #[test]
    fn empty_program() {
        let msgs = execute("", &default_ctx(), 0).unwrap();
        assert!(msgs.is_empty());
    }
}
