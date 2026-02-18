//! B4 — Safe Mini-Expr Evaluator.
//!
//! A tiny, safe expression engine for `descriptor.parse.expr`.
//! Enabled only when `expr.enabled = true`.
//!
//! # Supported features
//! - Dot access: `a.b.c`
//! - Array indexing: `a[0]`
//! - Fallback operator: `x ?? y`
//! - Functions (whitelist only): `to_number(x)`, `to_string(x)`
//!
//! # Strict prohibitions
//! - No arithmetic (+, -, *, /)
//! - No user-defined functions
//! - No loops/recursion
//! - No external access (IO/network/env)
//!
//! # Complexity constraints
//! - Maximum expression length: configurable (default 4096 bytes)
//! - Maximum AST nodes: 1000
//! - Evaluation step bound: 10_000
//! - Array index out-of-range → null (not error)

use serde_json::Value;
use thiserror::Error;

// ───────────────────────────────────────────────────────────────────────────
// Errors
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ExprError {
    #[error("expression too long: {len} bytes (max {max})")]
    TooLong { len: usize, max: usize },

    #[error("AST too complex: {nodes} nodes (max {max})")]
    TooManyNodes { nodes: usize, max: usize },

    #[error("evaluation step limit exceeded ({max} steps)")]
    StepLimitExceeded { max: usize },

    #[error("parse error at position {pos}: {message}")]
    Parse { pos: usize, message: String },

    #[error("unknown function '{name}' (allowed: to_number, to_string)")]
    UnknownFunction { name: String },

    #[error("runtime error: {0}")]
    Runtime(String),
}

// ───────────────────────────────────────────────────────────────────────────
// Configuration
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExprConfig {
    pub max_expression_length: usize,
    pub max_ast_nodes: usize,
    pub max_eval_steps: usize,
}

impl Default for ExprConfig {
    fn default() -> Self {
        Self {
            max_expression_length: 4096,
            max_ast_nodes: 1_000,
            max_eval_steps: 10_000,
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// AST
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ExprNode {
    /// Root/field identifier: `a`
    Ident(String),
    /// Dot access: `expr.field`
    DotAccess {
        object: Box<ExprNode>,
        field: String,
    },
    /// Array index: `expr[index]`
    IndexAccess { object: Box<ExprNode>, index: usize },
    /// Fallback: `left ?? right`
    Fallback {
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },
    /// Function call: `func_name(arg)`
    FunctionCall { name: String, arg: Box<ExprNode> },
    /// String literal
    StringLit(String),
    /// Number literal
    NumberLit(f64),
}

// ───────────────────────────────────────────────────────────────────────────
// Parser
// ───────────────────────────────────────────────────────────────────────────

struct ExprParser {
    chars: Vec<char>,
    pos: usize,
    node_count: usize,
    max_nodes: usize,
}

impl ExprParser {
    fn new(input: &str, max_nodes: usize) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            node_count: 0,
            max_nodes,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn add_node(&mut self) -> Result<(), ExprError> {
        self.node_count += 1;
        if self.node_count > self.max_nodes {
            return Err(ExprError::TooManyNodes {
                nodes: self.node_count,
                max: self.max_nodes,
            });
        }
        Ok(())
    }

    fn parse_expr(&mut self) -> Result<ExprNode, ExprError> {
        self.parse_fallback()
    }

    fn parse_fallback(&mut self) -> Result<ExprNode, ExprError> {
        let mut left = self.parse_postfix()?;
        loop {
            self.skip_ws();
            if self.pos + 1 < self.chars.len()
                && self.chars[self.pos] == '?'
                && self.chars[self.pos + 1] == '?'
            {
                self.pos += 2; // consume ??
                self.skip_ws();
                let right = self.parse_postfix()?;
                self.add_node()?;
                left = ExprNode::Fallback {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_postfix(&mut self) -> Result<ExprNode, ExprError> {
        let mut node = self.parse_primary()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('.') => {
                    self.advance(); // consume '.'
                    let field = self.parse_ident_str()?;
                    self.add_node()?;
                    node = ExprNode::DotAccess {
                        object: Box::new(node),
                        field,
                    };
                }
                Some('[') => {
                    self.advance(); // consume '['
                    self.skip_ws();
                    let idx = self.parse_usize()?;
                    self.skip_ws();
                    if self.advance() != Some(']') {
                        return Err(ExprError::Parse {
                            pos: self.pos,
                            message: "expected ']'".to_string(),
                        });
                    }
                    self.add_node()?;
                    node = ExprNode::IndexAccess {
                        object: Box::new(node),
                        index: idx,
                    };
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<ExprNode, ExprError> {
        self.skip_ws();
        match self.peek() {
            Some('"') | Some('\'') => self.parse_string_lit(),
            Some(c) if c.is_ascii_digit() => self.parse_number_lit(),
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let name = self.parse_ident_str()?;
                self.skip_ws();
                // Check if it's a function call
                if self.peek() == Some('(') {
                    self.advance(); // consume '('
                    self.skip_ws();
                    let arg = self.parse_expr()?;
                    self.skip_ws();
                    if self.advance() != Some(')') {
                        return Err(ExprError::Parse {
                            pos: self.pos,
                            message: "expected ')' after function argument".to_string(),
                        });
                    }
                    // Whitelist check
                    if name != "to_number" && name != "to_string" {
                        return Err(ExprError::UnknownFunction { name });
                    }
                    self.add_node()?;
                    Ok(ExprNode::FunctionCall {
                        name,
                        arg: Box::new(arg),
                    })
                } else {
                    self.add_node()?;
                    Ok(ExprNode::Ident(name))
                }
            }
            Some(c) => Err(ExprError::Parse {
                pos: self.pos,
                message: format!("unexpected character '{}'", c),
            }),
            None => Err(ExprError::Parse {
                pos: self.pos,
                message: "unexpected end of expression".to_string(),
            }),
        }
    }

    fn parse_ident_str(&mut self) -> Result<String, ExprError> {
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_ascii_alphanumeric() || self.chars[self.pos] == '_')
        {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ExprError::Parse {
                pos: self.pos,
                message: "expected identifier".to_string(),
            });
        }
        Ok(self.chars[start..self.pos].iter().collect())
    }

    fn parse_usize(&mut self) -> Result<usize, ExprError> {
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ExprError::Parse {
                pos: self.pos,
                message: "expected number".to_string(),
            });
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<usize>().map_err(|_| ExprError::Parse {
            pos: start,
            message: format!("invalid index '{}'", s),
        })
    }

    fn parse_string_lit(&mut self) -> Result<ExprNode, ExprError> {
        let quote = self.advance().unwrap(); // consume opening quote
        let mut s = String::new();
        loop {
            match self.advance() {
                None => {
                    return Err(ExprError::Parse {
                        pos: self.pos,
                        message: "unterminated string literal".to_string(),
                    });
                }
                Some(c) if c == quote => break,
                Some('\\') => match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('\\') => s.push('\\'),
                    Some(c) if c == quote => s.push(c),
                    _ => {
                        return Err(ExprError::Parse {
                            pos: self.pos,
                            message: "invalid escape sequence".to_string(),
                        });
                    }
                },
                Some(c) => s.push(c),
            }
        }
        self.add_node()?;
        Ok(ExprNode::StringLit(s))
    }

    fn parse_number_lit(&mut self) -> Result<ExprNode, ExprError> {
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_ascii_digit() || self.chars[self.pos] == '.')
        {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        let n: f64 = s.parse().map_err(|_| ExprError::Parse {
            pos: start,
            message: format!("invalid number '{}'", s),
        })?;
        self.add_node()?;
        Ok(ExprNode::NumberLit(n))
    }
}

/// Parse an expression string into an AST.
pub fn parse_expr(input: &str, config: &ExprConfig) -> Result<ExprNode, ExprError> {
    if input.len() > config.max_expression_length {
        return Err(ExprError::TooLong {
            len: input.len(),
            max: config.max_expression_length,
        });
    }
    let mut parser = ExprParser::new(input, config.max_ast_nodes);
    let node = parser.parse_expr()?;
    parser.skip_ws();
    if parser.pos < parser.chars.len() {
        return Err(ExprError::Parse {
            pos: parser.pos,
            message: format!("unexpected trailing input at position {}", parser.pos),
        });
    }
    Ok(node)
}

// ───────────────────────────────────────────────────────────────────────────
// Evaluator
// ───────────────────────────────────────────────────────────────────────────

struct Evaluator<'a> {
    root: &'a Value,
    steps: usize,
    max_steps: usize,
}

impl<'a> Evaluator<'a> {
    fn new(root: &'a Value, max_steps: usize) -> Self {
        Self {
            root,
            steps: 0,
            max_steps,
        }
    }

    fn step(&mut self) -> Result<(), ExprError> {
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(ExprError::StepLimitExceeded {
                max: self.max_steps,
            });
        }
        Ok(())
    }

    fn eval(&mut self, node: &ExprNode) -> Result<Value, ExprError> {
        self.step()?;
        match node {
            ExprNode::Ident(name) => Ok(self.root.get(name).cloned().unwrap_or(Value::Null)),
            ExprNode::DotAccess { object, field } => {
                let obj = self.eval(object)?;
                Ok(obj.get(field).cloned().unwrap_or(Value::Null))
            }
            ExprNode::IndexAccess { object, index } => {
                let obj = self.eval(object)?;
                match &obj {
                    Value::Array(arr) => Ok(arr.get(*index).cloned().unwrap_or(Value::Null)),
                    _ => Ok(Value::Null),
                }
            }
            ExprNode::Fallback { left, right } => {
                let l = self.eval(left)?;
                if l.is_null() {
                    self.eval(right)
                } else {
                    Ok(l)
                }
            }
            ExprNode::FunctionCall { name, arg } => {
                let val = self.eval(arg)?;
                match name.as_str() {
                    "to_number" => self.fn_to_number(&val),
                    "to_string" => self.fn_to_string(&val),
                    _ => Err(ExprError::UnknownFunction { name: name.clone() }),
                }
            }
            ExprNode::StringLit(s) => Ok(Value::String(s.clone())),
            ExprNode::NumberLit(n) => Ok(serde_json::Number::from_f64(*n)
                .map(Value::Number)
                .unwrap_or(Value::Null)),
        }
    }

    fn fn_to_number(&self, val: &Value) -> Result<Value, ExprError> {
        match val {
            Value::Number(_) => Ok(val.clone()),
            Value::String(s) => {
                let n: f64 = s.parse().map_err(|_| {
                    ExprError::Runtime(format!("to_number: cannot convert '{}' to number", s))
                })?;
                Ok(serde_json::Number::from_f64(n)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            }
            _ => Err(ExprError::Runtime(format!(
                "to_number: unsupported type '{}'",
                value_type(val)
            ))),
        }
    }

    fn fn_to_string(&self, val: &Value) -> Result<Value, ExprError> {
        match val {
            Value::String(_) => Ok(val.clone()),
            Value::Number(n) => Ok(Value::String(n.to_string())),
            Value::Bool(b) => Ok(Value::String(b.to_string())),
            Value::Null => Ok(Value::String("null".to_string())),
            _ => Err(ExprError::Runtime(format!(
                "to_string: unsupported type '{}'",
                value_type(val)
            ))),
        }
    }
}

fn value_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Evaluate an expression against a JSON payload.
///
/// Returns a `serde_json::Value` (which can then be cast using B3 rules).
pub fn evaluate(
    expression: &str,
    payload: &Value,
    config: &ExprConfig,
) -> Result<Value, ExprError> {
    let ast = parse_expr(expression, config)?;
    let mut evaluator = Evaluator::new(payload, config.max_eval_steps);
    evaluator.eval(&ast)
}

// ───────────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cfg() -> ExprConfig {
        ExprConfig::default()
    }

    #[test]
    fn dot_access() {
        let payload = json!({"data": {"price": 42.5}});
        let r = evaluate("data.price", &payload, &cfg()).unwrap();
        assert_eq!(r, json!(42.5));
    }

    #[test]
    fn array_indexing() {
        let payload = json!({"items": [10, 20, 30]});
        let r = evaluate("items[1]", &payload, &cfg()).unwrap();
        assert_eq!(r, json!(20));
    }

    #[test]
    fn array_index_out_of_range_returns_null() {
        let payload = json!({"items": [10]});
        let r = evaluate("items[99]", &payload, &cfg()).unwrap();
        assert_eq!(r, Value::Null);
    }

    #[test]
    fn missing_field_returns_null() {
        let payload = json!({"a": 1});
        let r = evaluate("b", &payload, &cfg()).unwrap();
        assert_eq!(r, Value::Null);
    }

    #[test]
    fn fallback_operator_on_null() {
        let payload = json!({"a": null, "b": "fallback"});
        let r = evaluate("a ?? b", &payload, &cfg()).unwrap();
        assert_eq!(r, json!("fallback"));
    }

    #[test]
    fn fallback_operator_on_present() {
        let payload = json!({"a": "present", "b": "fallback"});
        let r = evaluate("a ?? b", &payload, &cfg()).unwrap();
        assert_eq!(r, json!("present"));
    }

    #[test]
    fn fallback_chain() {
        let payload = json!({"c": "deep"});
        let r = evaluate("a ?? b ?? c", &payload, &cfg()).unwrap();
        assert_eq!(r, json!("deep"));
    }

    #[test]
    fn fn_to_number() {
        let payload = json!({"price": "42.5"});
        let r = evaluate("to_number(price)", &payload, &cfg()).unwrap();
        assert_eq!(r, json!(42.5));
    }

    #[test]
    fn fn_to_string() {
        let payload = json!({"count": 7});
        let r = evaluate("to_string(count)", &payload, &cfg()).unwrap();
        assert_eq!(r, json!("7"));
    }

    #[test]
    fn unknown_function_rejected() {
        let payload = json!({});
        let err = evaluate("unknown_fn(x)", &payload, &cfg()).unwrap_err();
        match err {
            ExprError::UnknownFunction { name } => assert_eq!(name, "unknown_fn"),
            other => panic!("expected UnknownFunction, got: {}", other),
        }
    }

    #[test]
    fn expression_too_long() {
        let long = "a".repeat(5000);
        let payload = json!({});
        let err = evaluate(&long, &payload, &cfg()).unwrap_err();
        match err {
            ExprError::TooLong { .. } => {}
            other => panic!("expected TooLong, got: {}", other),
        }
    }

    #[test]
    fn ast_node_limit() {
        // Build a deeply nested expression that exceeds 1000 nodes
        // Use short field names to stay under the byte-length limit
        let expr = (0..1100)
            .map(|i| format!("f{}", i))
            .collect::<Vec<_>>()
            .join(".");
        let payload = json!({});
        // Raise max_expression_length so TooLong doesn't fire first
        let mut c = cfg();
        c.max_expression_length = 100_000;
        let err = evaluate(&expr, &payload, &c).unwrap_err();
        match err {
            ExprError::TooManyNodes { .. } => {}
            other => panic!("expected TooManyNodes, got: {}", other),
        }
    }

    #[test]
    fn combined_dot_index_fallback() {
        let payload = json!({
            "data": {
                "bids": [[100, 1], [99, 2]],
            }
        });
        let r = evaluate("data.bids[0][0]", &payload, &cfg()).unwrap();
        assert_eq!(r, json!(100));
    }

    #[test]
    fn string_literal_in_fallback() {
        let payload = json!({});
        let r = evaluate("missing ?? \"default\"", &payload, &cfg()).unwrap();
        assert_eq!(r, json!("default"));
    }
}
