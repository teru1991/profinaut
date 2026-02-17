//! Safe mini-expression evaluator for `descriptor.parse.expr`.
//!
//! Supported features:
//! - Dot access: `a.b.c`
//! - Array indexing: `a[0]`
//! - Fallback operator: `x ?? y`
//! - Whitelisted functions: `to_number(x)`, `to_string(x)`
//!
//! Strict prohibitions: no arithmetic, no user-defined functions, no loops,
//! no recursion, no external access (IO/network/env).
//!
//! Complexity constraints:
//! - Maximum expression length: configurable (default 4096 bytes)
//! - Maximum AST nodes: 1000
//! - Evaluation step bound: 10000

use serde_json::Value;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_MAX_EXPR_LEN: usize = 4096;
const MAX_AST_NODES: usize = 1000;
const MAX_EVAL_STEPS: usize = 10_000;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ExprError {
    #[error("expression too long: {len} bytes (max {max})")]
    TooLong { len: usize, max: usize },

    #[error("AST too complex: {count} nodes (max {max})")]
    TooComplex { count: usize, max: usize },

    #[error("evaluation step limit exceeded (max {max})")]
    StepLimitExceeded { max: usize },

    #[error("parse error at position {pos}: {message}")]
    Parse { pos: usize, message: String },

    #[error("unknown function '{name}' (allowed: to_number, to_string)")]
    UnknownFunction { name: String },

    #[error("to_number: cannot convert {value_desc} to number")]
    CastToNumber { value_desc: String },
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Expr {
    /// An identifier: `foo`
    Ident(String),
    /// String literal: `"hello"`
    StringLit(String),
    /// Numeric literal: `42`
    NumberLit(f64),
    /// Dot access: `expr.field`
    Dot(Box<Expr>, String),
    /// Array index: `expr[index]`
    Index(Box<Expr>, usize),
    /// Fallback: `expr ?? expr`
    Fallback(Box<Expr>, Box<Expr>),
    /// Function call: `func(expr)`
    FnCall(String, Box<Expr>),
}

impl Expr {
    fn count_nodes(&self) -> usize {
        match self {
            Expr::Ident(_) | Expr::StringLit(_) | Expr::NumberLit(_) => 1,
            Expr::Dot(inner, _) | Expr::Index(inner, _) | Expr::FnCall(_, inner) => {
                1 + inner.count_nodes()
            }
            Expr::Fallback(a, b) => 1 + a.count_nodes() + b.count_nodes(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum ExprToken {
    Ident(String),
    StringLit(String),
    NumberLit(f64),
    Dot,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Fallback, // ??
}

fn tokenize_expr(source: &str) -> Result<Vec<(ExprToken, usize)>, ExprError> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < len {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        let pos = i;
        match bytes[i] {
            b'.' => {
                tokens.push((ExprToken::Dot, pos));
                i += 1;
            }
            b'[' => {
                tokens.push((ExprToken::LBracket, pos));
                i += 1;
            }
            b']' => {
                tokens.push((ExprToken::RBracket, pos));
                i += 1;
            }
            b'(' => {
                tokens.push((ExprToken::LParen, pos));
                i += 1;
            }
            b')' => {
                tokens.push((ExprToken::RParen, pos));
                i += 1;
            }
            b'?' if i + 1 < len && bytes[i + 1] == b'?' => {
                tokens.push((ExprToken::Fallback, pos));
                i += 2;
            }
            b'"' => {
                i += 1;
                let start = i;
                while i < len && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i >= len {
                    return Err(ExprError::Parse {
                        pos,
                        message: "unterminated string literal".to_string(),
                    });
                }
                let s = source[start..i].replace("\\\"", "\"").replace("\\\\", "\\");
                tokens.push((ExprToken::StringLit(s), pos));
                i += 1; // skip closing quote
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                    i += 1;
                }
                let num_str = &source[start..i];
                let n: f64 = num_str.parse().map_err(|_| ExprError::Parse {
                    pos,
                    message: format!("invalid number '{num_str}'"),
                })?;
                tokens.push((ExprToken::NumberLit(n), pos));
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = source[start..i].to_string();
                tokens.push((ExprToken::Ident(word), pos));
            }
            _ => {
                return Err(ExprError::Parse {
                    pos,
                    message: format!("unexpected character '{}'", bytes[i] as char),
                });
            }
        }
    }
    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct ExprParser {
    tokens: Vec<(ExprToken, usize)>,
    pos: usize,
}

impl ExprParser {
    fn new(tokens: Vec<(ExprToken, usize)>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&ExprToken> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn cur_pos(&self) -> usize {
        self.tokens
            .get(self.pos)
            .map(|(_, p)| *p)
            .unwrap_or(usize::MAX)
    }

    fn advance(&mut self) -> Option<(ExprToken, usize)> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    /// Parse the top-level expression (with fallback).
    fn parse_expr(&mut self) -> Result<Expr, ExprError> {
        let left = self.parse_access()?;

        if self.peek() == Some(&ExprToken::Fallback) {
            self.advance();
            let right = self.parse_expr()?; // right-associative
            return Ok(Expr::Fallback(Box::new(left), Box::new(right)));
        }

        Ok(left)
    }

    /// Parse dot access and array indexing (left-to-right).
    fn parse_access(&mut self) -> Result<Expr, ExprError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                Some(ExprToken::Dot) => {
                    self.advance();
                    let pos = self.cur_pos();
                    match self.advance() {
                        Some((ExprToken::Ident(name), _)) => {
                            expr = Expr::Dot(Box::new(expr), name);
                        }
                        _ => {
                            return Err(ExprError::Parse {
                                pos,
                                message: "expected field name after '.'".to_string(),
                            });
                        }
                    }
                }
                Some(ExprToken::LBracket) => {
                    self.advance();
                    let pos = self.cur_pos();
                    match self.advance() {
                        Some((ExprToken::NumberLit(n), _)) => {
                            let idx = n as usize;
                            let close_pos = self.cur_pos();
                            match self.advance() {
                                Some((ExprToken::RBracket, _)) => {}
                                _ => {
                                    return Err(ExprError::Parse {
                                        pos: close_pos,
                                        message: "expected ']' after index".to_string(),
                                    });
                                }
                            }
                            expr = Expr::Index(Box::new(expr), idx);
                        }
                        _ => {
                            return Err(ExprError::Parse {
                                pos,
                                message: "expected numeric index in brackets".to_string(),
                            });
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ExprError> {
        let pos = self.cur_pos();
        match self.advance() {
            Some((ExprToken::Ident(name), _)) => {
                // Check if this is a function call
                if self.peek() == Some(&ExprToken::LParen) {
                    self.advance(); // consume '('
                    let arg = self.parse_expr()?;
                    let close_pos = self.cur_pos();
                    match self.advance() {
                        Some((ExprToken::RParen, _)) => {}
                        _ => {
                            return Err(ExprError::Parse {
                                pos: close_pos,
                                message: "expected ')' after function argument".to_string(),
                            })
                        }
                    }
                    Ok(Expr::FnCall(name, Box::new(arg)))
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Some((ExprToken::StringLit(s), _)) => Ok(Expr::StringLit(s)),
            Some((ExprToken::NumberLit(n), _)) => Ok(Expr::NumberLit(n)),
            Some((ExprToken::LParen, _)) => {
                let inner = self.parse_expr()?;
                let close_pos = self.cur_pos();
                match self.advance() {
                    Some((ExprToken::RParen, _)) => Ok(inner),
                    _ => Err(ExprError::Parse {
                        pos: close_pos,
                        message: "expected ')' after grouped expression".to_string(),
                    }),
                }
            }
            _ => Err(ExprError::Parse {
                pos,
                message: "expected identifier, string, number, or '('".to_string(),
            }),
        }
    }
}

fn parse_expr(source: &str, max_len: usize) -> Result<Expr, ExprError> {
    if source.len() > max_len {
        return Err(ExprError::TooLong {
            len: source.len(),
            max: max_len,
        });
    }
    let tokens = tokenize_expr(source)?;
    let mut parser = ExprParser::new(tokens);
    let expr = parser.parse_expr()?;

    if !parser.at_end() {
        let pos = parser.cur_pos();
        return Err(ExprError::Parse {
            pos,
            message: "unexpected token after expression".to_string(),
        });
    }

    let node_count = expr.count_nodes();
    if node_count > MAX_AST_NODES {
        return Err(ExprError::TooComplex {
            count: node_count,
            max: MAX_AST_NODES,
        });
    }

    Ok(expr)
}

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

struct Evaluator<'a> {
    payload: &'a Value,
    steps: usize,
}

impl<'a> Evaluator<'a> {
    fn new(payload: &'a Value) -> Self {
        Self { payload, steps: 0 }
    }

    fn step(&mut self) -> Result<(), ExprError> {
        self.steps += 1;
        if self.steps > MAX_EVAL_STEPS {
            return Err(ExprError::StepLimitExceeded {
                max: MAX_EVAL_STEPS,
            });
        }
        Ok(())
    }

    fn eval(&mut self, expr: &Expr) -> Result<Value, ExprError> {
        self.step()?;
        match expr {
            Expr::Ident(name) => {
                // Look up from payload root
                Ok(self.payload.get(name).cloned().unwrap_or(Value::Null))
            }
            Expr::StringLit(s) => Ok(Value::String(s.clone())),
            Expr::NumberLit(n) => Ok(serde_json::Number::from_f64(*n)
                .map(Value::Number)
                .unwrap_or(Value::Null)),
            Expr::Dot(inner, field) => {
                let val = self.eval(inner)?;
                Ok(val.get(field).cloned().unwrap_or(Value::Null))
            }
            Expr::Index(inner, idx) => {
                let val = self.eval(inner)?;
                // Array index out-of-range => null (not error)
                Ok(val.get(*idx).cloned().unwrap_or(Value::Null))
            }
            Expr::Fallback(left, right) => {
                let val = self.eval(left)?;
                if val.is_null() {
                    self.eval(right)
                } else {
                    Ok(val)
                }
            }
            Expr::FnCall(name, arg) => {
                let val = self.eval(arg)?;
                match name.as_str() {
                    "to_number" => match &val {
                        Value::Number(_) => Ok(val),
                        Value::String(s) => {
                            let n: f64 = s.parse().map_err(|_| ExprError::CastToNumber {
                                value_desc: format!("string \"{s}\""),
                            })?;
                            Ok(serde_json::Number::from_f64(n)
                                .map(Value::Number)
                                .unwrap_or(Value::Null))
                        }
                        Value::Bool(b) => {
                            let n = if *b { 1.0 } else { 0.0 };
                            Ok(serde_json::Number::from_f64(n)
                                .map(Value::Number)
                                .unwrap_or(Value::Null))
                        }
                        _ => Err(ExprError::CastToNumber {
                            value_desc: format!("{val}"),
                        }),
                    },
                    "to_string" => match &val {
                        Value::String(_) => Ok(val),
                        Value::Number(n) => Ok(Value::String(n.to_string())),
                        Value::Bool(b) => Ok(Value::String(b.to_string())),
                        Value::Null => Ok(Value::String("null".to_string())),
                        other => Ok(Value::String(other.to_string())),
                    },
                    _ => Err(ExprError::UnknownFunction { name: name.clone() }),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evaluate a mini-expression against a JSON payload.
///
/// Returns the resulting `serde_json::Value`.
pub fn evaluate(source: &str, payload: &Value) -> Result<Value, ExprError> {
    evaluate_with_max_len(source, payload, DEFAULT_MAX_EXPR_LEN)
}

/// Evaluate with a custom maximum expression length.
pub fn evaluate_with_max_len(
    source: &str,
    payload: &Value,
    max_len: usize,
) -> Result<Value, ExprError> {
    let ast = parse_expr(source, max_len)?;
    let mut evaluator = Evaluator::new(payload);
    evaluator.eval(&ast)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn payload() -> Value {
        json!({
            "data": {
                "symbol": "BTC/USDT",
                "price": "42000.50",
                "trades": [
                    {"id": 1, "amount": 0.5},
                    {"id": 2, "amount": 1.0}
                ]
            },
            "channel": "trade",
            "seq": 999
        })
    }

    #[test]
    fn dot_access() {
        let val = evaluate("data.symbol", &payload()).unwrap();
        assert_eq!(val, json!("BTC/USDT"));
    }

    #[test]
    fn deep_dot_access() {
        let val = evaluate("data.trades", &payload()).unwrap();
        assert!(val.is_array());
    }

    #[test]
    fn array_indexing() {
        let val = evaluate("data.trades[0].id", &payload()).unwrap();
        assert_eq!(val, json!(1));
    }

    #[test]
    fn array_indexing_second() {
        let val = evaluate("data.trades[1].amount", &payload()).unwrap();
        assert_eq!(val, json!(1.0));
    }

    #[test]
    fn array_out_of_range_is_null() {
        let val = evaluate("data.trades[99]", &payload()).unwrap();
        assert!(val.is_null());
    }

    #[test]
    fn missing_field_is_null() {
        let val = evaluate("data.nonexistent", &payload()).unwrap();
        assert!(val.is_null());
    }

    #[test]
    fn fallback_operator_present() {
        let val = evaluate("data.symbol ?? \"default\"", &payload()).unwrap();
        assert_eq!(val, json!("BTC/USDT"));
    }

    #[test]
    fn fallback_operator_missing() {
        let val = evaluate("data.missing ?? \"default\"", &payload()).unwrap();
        assert_eq!(val, json!("default"));
    }

    #[test]
    fn fallback_chain() {
        let val = evaluate("data.a ?? data.b ?? \"final\"", &payload()).unwrap();
        assert_eq!(val, json!("final"));
    }

    #[test]
    fn to_number_from_string() {
        let val = evaluate("to_number(data.price)", &payload()).unwrap();
        assert_eq!(val, json!(42000.50));
    }

    #[test]
    fn to_number_from_number() {
        let val = evaluate("to_number(seq)", &payload()).unwrap();
        assert_eq!(val, json!(999));
    }

    #[test]
    fn to_string_from_number() {
        let val = evaluate("to_string(seq)", &payload()).unwrap();
        assert_eq!(val, json!("999"));
    }

    #[test]
    fn to_string_from_string() {
        let val = evaluate("to_string(data.symbol)", &payload()).unwrap();
        assert_eq!(val, json!("BTC/USDT"));
    }

    #[test]
    fn unknown_function_error() {
        let err = evaluate("unknown_fn(data.symbol)", &payload()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown_fn"), "got: {msg}");
        assert!(msg.contains("unknown function"), "got: {msg}");
    }

    #[test]
    fn expression_too_long() {
        let long = "a".repeat(5000);
        let err = evaluate_with_max_len(&long, &payload(), 4096).unwrap_err();
        assert!(matches!(err, ExprError::TooLong { .. }));
    }

    #[test]
    fn root_level_field() {
        let val = evaluate("channel", &payload()).unwrap();
        assert_eq!(val, json!("trade"));
    }

    #[test]
    fn string_literal() {
        let val = evaluate("\"hello\"", &payload()).unwrap();
        assert_eq!(val, json!("hello"));
    }

    #[test]
    fn number_literal() {
        let val = evaluate("42", &payload()).unwrap();
        assert_eq!(val, json!(42.0));
    }
}
