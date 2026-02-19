from __future__ import annotations

from dataclasses import dataclass
import json
import re
from typing import Any

MAX_TOKENS = 100_000
MAX_AST_NODES = 100_000
MAX_AST_DEPTH = 32


@dataclass(frozen=True)
class Token:
    type: str
    value: str
    line: int
    column: int


class DslError(Exception):
    def __init__(
        self,
        code: str,
        message: str,
        *,
        line: int | None = None,
        column: int | None = None,
        context_tokens: list[str] | None = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.line = line
        self.column = column
        self.context_tokens = context_tokens or []

    def to_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "error_code": self.code,
            "message": self.message,
        }
        if self.line is not None:
            payload["line"] = self.line
        if self.column is not None:
            payload["column"] = self.column
        if self.context_tokens:
            payload["context_tokens"] = self.context_tokens
        return payload


@dataclass(frozen=True)
class Expr:
    kind: str
    value: Any
    line: int
    column: int


@dataclass(frozen=True)
class AstNode:
    kind: str
    line: int
    column: int
    data: dict[str, Any]


@dataclass(frozen=True)
class Ast:
    root: AstNode


@dataclass(frozen=True)
class ExecutionLimits:
    max_output_bytes: int = 1_000_000
    max_steps: int = 2_000_000
    max_loop_iters: int = 100_000
    max_emit_count: int = 100_000
    max_string_len: int = 1_000_000


@dataclass(frozen=True)
class ExecutionContext:
    values: dict[str, Any]
    trace_id: str | None = None
    request_id: str | None = None
    run_id: str | None = None


class _Executor:
    def __init__(self, *, ast: Ast, context: ExecutionContext, limits: ExecutionLimits) -> None:
        self._ast = ast
        self._context = context
        self._limits = limits
        self._steps = 0
        self._output: list[str] = []
        self._output_bytes = 0
        self._emit_count = 0
        self._scope_stack: list[dict[str, Any]] = [dict(context.values)]

    def run(self) -> str:
        self._exec_node(self._ast.root)
        return "".join(self._output)

    def _exec_node(self, node: AstNode) -> None:
        self._step(node)
        if node.kind == "Program":
            self._exec_node(node.data["body"])
            return
        if node.kind == "Seq":
            for item in node.data["items"]:
                self._exec_node(item)
            return
        if node.kind == "Text":
            self._append_output(self._eval_expr(node.data["value"]))
            return
        if node.kind == "Emit":
            self._append_output(_to_string(self._eval_expr(node.data["expr"])))
            return
        if node.kind == "If":
            cond_value = self._eval_expr(node.data["condition"])
            branch = node.data["then"] if _truthy(cond_value) else node.data["else"]
            if branch is not None:
                self._exec_node(branch)
            return
        if node.kind == "Foreach":
            iterable_value = self._eval_expr(node.data["iterable"])
            if not isinstance(iterable_value, list):
                raise DslError(
                    "DSL_TYPE_MISMATCH",
                    "foreach iterable must evaluate to an array",
                    line=node.line,
                    column=node.column,
                )
            var_name = node.data["var"]
            loop_count = 0
            for item in iterable_value:
                loop_count += 1
                if loop_count > self._limits.max_loop_iters:
                    raise DslError("DSL_EXEC_LIMIT", "Loop iteration limit exceeded", line=node.line, column=node.column)
                self._scope_stack.append({var_name: item})
                try:
                    self._exec_node(node.data["body"])
                finally:
                    self._scope_stack.pop()
            return
        raise DslError("DSL_EXEC_ERROR", f"Unsupported AST node '{node.kind}'", line=node.line, column=node.column)

    def _eval_expr(self, expr: Expr) -> Any:
        self._step(expr)
        if expr.kind == "String":
            return self._resolve_string(expr)
        if expr.kind in {"Number", "Bool", "Null"}:
            return expr.value
        if expr.kind == "Expr":
            return self._resolve_path(expr)
        if expr.kind == "UnaryOp":
            operand = self._eval_expr(expr.value["operand"])
            if expr.value["op"] == "not":
                if not isinstance(operand, bool):
                    raise DslError("DSL_TYPE_MISMATCH", "'not' expects bool", line=expr.line, column=expr.column)
                return not operand
            raise DslError("DSL_EXEC_ERROR", f"Unsupported unary op '{expr.value['op']}'", line=expr.line, column=expr.column)
        if expr.kind == "BinaryOp":
            return self._eval_binary(expr)
        if expr.kind == "Call":
            return self._eval_call(expr)
        if expr.kind == "MapLiteral":
            return {entry["key"]: self._eval_expr(entry["value"]) for entry in expr.value}
        raise DslError("DSL_EXEC_ERROR", f"Unsupported expression '{expr.kind}'", line=expr.line, column=expr.column)

    def _resolve_string(self, expr: Expr) -> str:
        text = expr.value["text"]
        placeholders = expr.value["placeholders"]
        for placeholder in placeholders:
            resolved = self._resolve_path_expr(placeholder.split("."), expr.line, expr.column)
            text = text.replace("{{" + placeholder + "}}", _to_string(resolved))
            text = text.replace("{{ " + placeholder + " }}", _to_string(resolved))
        if len(text) > self._limits.max_string_len:
            raise DslError("DSL_EXEC_LIMIT", "String length exceeds limit", line=expr.line, column=expr.column)
        return text

    def _resolve_path(self, expr: Expr) -> Any:
        return self._resolve_path_expr(expr.value["path"], expr.line, expr.column)

    def _resolve_path_expr(self, path: list[str], line: int, column: int) -> Any:
        base = self._lookup_var(path[0])
        if base is _MISSING:
            raise DslError("DSL_EXEC_ERROR", f"Unknown variable '{path[0]}'", line=line, column=column)
        current = base
        for segment in path[1:]:
            if isinstance(current, dict) and segment in current:
                current = current[segment]
                continue
            raise DslError("DSL_EXEC_ERROR", f"Path not found: {'.'.join(path)}", line=line, column=column)
        return current

    def _lookup_var(self, name: str) -> Any:
        for scope in reversed(self._scope_stack):
            if name in scope:
                return scope[name]
        return _MISSING

    def _append_output(self, value: str) -> None:
        self._emit_count += 1
        if self._emit_count > self._limits.max_emit_count:
            raise DslError("DSL_OUTPUT_LIMIT", "Emit/output item limit exceeded")
        value_bytes = len(value.encode("utf-8"))
        new_size = self._output_bytes + value_bytes
        if new_size > self._limits.max_output_bytes:
            raise DslError("DSL_OUTPUT_LIMIT", "Output buffer limit exceeded")
        self._output.append(value)
        self._output_bytes = new_size

    def _step(self, source: AstNode | Expr) -> None:
        self._steps += 1
        if self._steps > self._limits.max_steps:
            raise DslError("DSL_EXEC_LIMIT", "Execution step limit exceeded", line=source.line, column=source.column)

    def _eval_binary(self, expr: Expr) -> Any:
        op = expr.value["op"]
        lhs = self._eval_expr(expr.value["left"])
        rhs = self._eval_expr(expr.value["right"])
        if op in {"and", "or"}:
            if not isinstance(lhs, bool) or not isinstance(rhs, bool):
                raise DslError("DSL_TYPE_MISMATCH", f"'{op}' expects bool operands", line=expr.line, column=expr.column)
            return lhs and rhs if op == "and" else lhs or rhs
        if op in {"==", "!=", "<", "<=", ">", ">="}:
            if type(lhs) is not type(rhs):
                raise DslError("DSL_TYPE_MISMATCH", "Comparison operands must have the same type", line=expr.line, column=expr.column)
            if op == "==":
                return lhs == rhs
            if op == "!=":
                return lhs != rhs
            if not isinstance(lhs, (int, float, str)) or isinstance(lhs, bool):
                raise DslError("DSL_TYPE_MISMATCH", f"'{op}' expects int/float/string operands", line=expr.line, column=expr.column)
            if op == "<":
                return lhs < rhs
            if op == "<=":
                return lhs <= rhs
            if op == ">":
                return lhs > rhs
            return lhs >= rhs
        raise DslError("DSL_EXEC_ERROR", f"Unsupported binary op '{op}'", line=expr.line, column=expr.column)

    def _eval_call(self, expr: Expr) -> Any:
        name = expr.value["name"]
        args = [self._eval_expr(arg) for arg in expr.value["args"]]
        if name == "len":
            if len(args) != 1 or not isinstance(args[0], (str, list, dict)):
                raise DslError("DSL_TYPE_MISMATCH", "len expects one string/array/object argument", line=expr.line, column=expr.column)
            return len(args[0])
        if name == "contains":
            if len(args) != 2:
                raise DslError("DSL_TYPE_MISMATCH", "contains expects 2 arguments", line=expr.line, column=expr.column)
            haystack, needle = args
            if isinstance(haystack, str):
                if not isinstance(needle, str):
                    raise DslError("DSL_TYPE_MISMATCH", "contains(string, x) requires string x", line=expr.line, column=expr.column)
                return needle in haystack
            if isinstance(haystack, list):
                return needle in haystack
            raise DslError("DSL_TYPE_MISMATCH", "contains expects array or string as first arg", line=expr.line, column=expr.column)
        if name == "default":
            if len(args) != 2:
                raise DslError("DSL_TYPE_MISMATCH", "default expects 2 arguments", line=expr.line, column=expr.column)
            value, fallback = args
            if value is None:
                return fallback
            if isinstance(value, str) and value == "":
                return fallback
            if isinstance(value, (list, dict)) and len(value) == 0:
                return fallback
            return value
        raise DslError("DSL_NOT_SUPPORTED", f"Unsupported function '{name}'", line=expr.line, column=expr.column)


class _Missing:
    pass


_MISSING = _Missing()


_KEYWORDS = {
    "foreach": "FOREACH",
    "in": "IN",
    "if": "IF",
    "else": "ELSE",
    "emit": "EMIT",
    "true": "TRUE",
    "false": "FALSE",
    "null": "NULL",
    "and": "AND",
    "or": "OR",
    "not": "NOT",
}

_UNSUPPORTED_KEYWORDS = {"while", "for", "function", "import", "eval"}
_PLACEHOLDER_RE = re.compile(r"\{\{\s*([A-Za-z_][A-Za-z0-9_\.]*)\s*\}\}")
_ALLOWED_PLACEHOLDERS = {
    "symbol",
    "venue",
    "venue_symbol",
    "market_kind",
    "trace_id",
    "request_id",
    "run_id",
}


class _Parser:
    def __init__(self, tokens: list[Token]):
        self.tokens = tokens
        self.i = 0

    def parse(self) -> Ast:
        seq = self._parse_seq(stop_at_rbrace=False)
        eof = self._peek()
        if eof.type != "EOF":
            raise self._err("DSL_PARSE_ERROR", f"Unexpected token {eof.type}", eof)
        return Ast(root=AstNode(kind="Program", line=1, column=1, data={"body": seq}))

    def _parse_seq(self, *, stop_at_rbrace: bool) -> AstNode:
        body: list[AstNode] = []
        start = self._peek()
        while True:
            tok = self._peek()
            if tok.type == "EOF":
                if stop_at_rbrace:
                    raise self._err("DSL_PARSE_ERROR", "Unexpected EOF; missing closing '}'", tok)
                break
            if stop_at_rbrace and tok.type == "RBRACE":
                break
            body.append(self._parse_stmt())
        return AstNode(kind="Seq", line=start.line, column=start.column, data={"items": body})

    def _parse_stmt(self) -> AstNode:
        tok = self._peek()
        if tok.type == "FOREACH":
            return self._parse_foreach()
        if tok.type == "IF":
            return self._parse_if()
        if tok.type == "EMIT":
            return self._parse_emit()
        if tok.type == "ELSE":
            raise self._err("DSL_UNEXPECTED_TOKEN", "'else' without matching 'if'", tok)
        if tok.type == "STRING":
            s = self._advance()
            expr = _string_expr_from_token(s)
            return AstNode(kind="Text", line=s.line, column=s.column, data={"value": expr})
        if tok.type == "IDENT" and tok.value in _UNSUPPORTED_KEYWORDS:
            raise self._err("DSL_NOT_SUPPORTED", f"Unsupported syntax '{tok.value}'", tok)
        raise self._err("DSL_UNEXPECTED_TOKEN", f"Unexpected token {tok.type}", tok)

    def _parse_foreach(self) -> AstNode:
        start = self._expect("FOREACH")
        var = self._expect("IDENT")
        self._expect("IN")
        iterable = self._parse_expr(until={"LBRACE"})
        body = self._parse_block()
        return AstNode(
            kind="Foreach",
            line=start.line,
            column=start.column,
            data={"var": var.value, "iterable": iterable, "body": body},
        )

    def _parse_if(self) -> AstNode:
        start = self._expect("IF")
        cond = self._parse_expr(until={"LBRACE"})
        then_branch = self._parse_block()
        else_branch = None
        if self._peek().type == "ELSE":
            self._advance()
            else_branch = self._parse_block()
        return AstNode(
            kind="If",
            line=start.line,
            column=start.column,
            data={"condition": cond, "then": then_branch, "else": else_branch},
        )

    def _parse_emit(self) -> AstNode:
        start = self._expect("EMIT")
        expr = self._parse_expr(until={"RBRACE", "EOF", "FOREACH", "IF", "ELSE", "EMIT"})
        return AstNode(kind="Emit", line=start.line, column=start.column, data={"expr": expr})

    def _parse_block(self) -> AstNode:
        self._expect("LBRACE")
        seq = self._parse_seq(stop_at_rbrace=True)
        self._expect("RBRACE")
        return seq

    def _parse_expr(self, *, until: set[str]) -> Expr:
        return self._parse_or_expr(until=until)

    def _parse_or_expr(self, *, until: set[str]) -> Expr:
        left = self._parse_and_expr(until=until)
        while self._peek().type not in until and self._peek().type == "OR":
            op = self._advance()
            right = self._parse_and_expr(until=until)
            left = Expr(kind="BinaryOp", value={"op": "or", "left": left, "right": right}, line=op.line, column=op.column)
        return left

    def _parse_and_expr(self, *, until: set[str]) -> Expr:
        left = self._parse_not_expr(until=until)
        while self._peek().type not in until and self._peek().type == "AND":
            op = self._advance()
            right = self._parse_not_expr(until=until)
            left = Expr(kind="BinaryOp", value={"op": "and", "left": left, "right": right}, line=op.line, column=op.column)
        return left

    def _parse_not_expr(self, *, until: set[str]) -> Expr:
        if self._peek().type not in until and self._peek().type == "NOT":
            op = self._advance()
            operand = self._parse_not_expr(until=until)
            return Expr(kind="UnaryOp", value={"op": "not", "operand": operand}, line=op.line, column=op.column)
        return self._parse_cmp_expr(until=until)

    def _parse_cmp_expr(self, *, until: set[str]) -> Expr:
        left = self._parse_primary_expr(until=until)
        cmp_tokens = {"EQ", "NE", "LT", "LE", "GT", "GE"}
        if self._peek().type not in until and self._peek().type in cmp_tokens:
            op_tok = self._advance()
            op_map = {"EQ": "==", "NE": "!=", "LT": "<", "LE": "<=", "GT": ">", "GE": ">="}
            right = self._parse_primary_expr(until=until)
            return Expr(kind="BinaryOp", value={"op": op_map[op_tok.type], "left": left, "right": right}, line=op_tok.line, column=op_tok.column)
        return left

    def _parse_primary_expr(self, *, until: set[str]) -> Expr:
        tok = self._peek()
        if tok.type in until:
            raise self._err("DSL_PARSE_ERROR", "Expected expression", tok)
        if tok.type == "STRING":
            self._advance()
            return _string_expr_from_token(tok)
        if tok.type == "NUMBER":
            self._advance()
            return Expr(kind="Number", value=float(tok.value) if "." in tok.value else int(tok.value), line=tok.line, column=tok.column)
        if tok.type == "TRUE":
            self._advance()
            return Expr(kind="Bool", value=True, line=tok.line, column=tok.column)
        if tok.type == "FALSE":
            self._advance()
            return Expr(kind="Bool", value=False, line=tok.line, column=tok.column)
        if tok.type == "NULL":
            self._advance()
            return Expr(kind="Null", value=None, line=tok.line, column=tok.column)
        if tok.type == "LBRACE":
            return self._parse_map_literal()
        if tok.type != "IDENT":
            raise self._err("DSL_PARSE_ERROR", f"Invalid expression token {tok.type}", tok)
        ident = self._advance()
        if self._peek().type == "LPAREN":
            return self._parse_call_expr(ident)
        path = [ident.value]
        while self._peek().type == "DOT":
            self._advance()
            nxt = self._expect("IDENT")
            path.append(nxt.value)
        return Expr(kind="Expr", value={"path": path}, line=ident.line, column=ident.column)

    def _parse_call_expr(self, ident: Token) -> Expr:
        self._expect("LPAREN")
        args: list[Expr] = []
        if self._peek().type != "RPAREN":
            while True:
                args.append(self._parse_expr(until={"COMMA", "RPAREN"}))
                if self._peek().type == "COMMA":
                    self._advance()
                    continue
                break
        self._expect("RPAREN")
        if ident.value in {"len", "contains", "default"}:
            return Expr(kind="Call", value={"name": ident.value, "args": args}, line=ident.line, column=ident.column)
        if ident.value not in {"json", "get"}:
            raise self._err("DSL_NOT_SUPPORTED", f"Unsupported function '{ident.value}'", ident)
        if len(args) != 1:
            raise self._err("DSL_TYPE_MISMATCH", f"{ident.value} expects 1 argument", ident)
        arg = args[0]
        if arg.kind != "String":
            raise self._err("DSL_TYPE_MISMATCH", f"{ident.value} expects a string pointer", Token("IDENT", ident.value, arg.line, arg.column))
        return Expr(
            kind="JsonPointer",
            value={"pointer": arg.value["text"]},
            line=ident.line,
            column=ident.column,
        )

    def _parse_map_literal(self) -> Expr:
        start = self._expect("LBRACE")
        entries: list[dict[str, Any]] = []
        while self._peek().type != "RBRACE":
            key_tok = self._peek()
            if key_tok.type == "STRING":
                self._advance()
                key = key_tok.value
            else:
                key = self._expect("IDENT").value
            self._expect("COLON")
            value_expr = self._parse_expr(until={"COMMA", "RBRACE"})
            entries.append({"key": key, "value": value_expr})
            if self._peek().type == "COMMA":
                self._advance()
            elif self._peek().type != "RBRACE":
                raise self._err("DSL_PARSE_ERROR", "Expected ',' or '}' in map literal", self._peek())
        self._expect("RBRACE")
        return Expr(kind="MapLiteral", value=entries, line=start.line, column=start.column)

    def _peek(self) -> Token:
        return self.tokens[self.i]

    def _advance(self) -> Token:
        tok = self.tokens[self.i]
        self.i += 1
        return tok

    def _expect(self, kind: str) -> Token:
        tok = self._peek()
        if tok.type != kind:
            raise self._err("DSL_UNEXPECTED_TOKEN", f"Expected {kind}, got {tok.type}", tok)
        return self._advance()

    def _err(self, code: str, msg: str, tok: Token) -> DslError:
        left = max(0, self.i - 2)
        right = min(len(self.tokens), self.i + 3)
        context = [f"{t.type}:{t.value}" for t in self.tokens[left:right] if t.type != "EOF"]
        return DslError(code, msg, line=tok.line, column=tok.column, context_tokens=context)


def tokenize(input: str) -> list[Token]:
    tokens: list[Token] = []
    i = 0
    line = 1
    col = 1

    def push(kind: str, value: str, t_line: int, t_col: int) -> None:
        if len(tokens) >= MAX_TOKENS:
            raise DslError("DSL_TOO_LARGE", "Token count exceeds limit", line=t_line, column=t_col)
        tokens.append(Token(type=kind, value=value, line=t_line, column=t_col))

    while i < len(input):
        ch = input[i]
        if ch in " \t\r":
            i += 1
            col += 1
            continue
        if ch == "\n":
            i += 1
            line += 1
            col = 1
            continue
        if ch == "#":
            while i < len(input) and input[i] != "\n":
                i += 1
                col += 1
            continue

        t_line, t_col = line, col

        if ch.isalpha() or ch == "_":
            start = i
            while i < len(input) and (input[i].isalnum() or input[i] == "_"):
                i += 1
                col += 1
            word = input[start:i]
            push(_KEYWORDS.get(word, "IDENT"), word, t_line, t_col)
            continue

        if ch.isdigit() or (ch == "-" and i + 1 < len(input) and input[i + 1].isdigit()):
            start = i
            i += 1
            col += 1
            while i < len(input) and input[i].isdigit():
                i += 1
                col += 1
            if i < len(input) and input[i] == ".":
                i += 1
                col += 1
                while i < len(input) and input[i].isdigit():
                    i += 1
                    col += 1
            push("NUMBER", input[start:i], t_line, t_col)
            continue

        if ch == '"':
            i += 1
            col += 1
            buf: list[str] = []
            escaped = False
            while i < len(input):
                curr = input[i]
                if escaped:
                    buf.append(curr)
                    escaped = False
                    i += 1
                    col += 1
                    continue
                if curr == "\\":
                    escaped = True
                    i += 1
                    col += 1
                    continue
                if curr == '"':
                    break
                if curr == "\n":
                    raise DslError("DSL_PARSE_ERROR", "Unterminated string literal", line=t_line, column=t_col)
                buf.append(curr)
                i += 1
                col += 1
            if i >= len(input) or input[i] != '"':
                raise DslError("DSL_PARSE_ERROR", "Unterminated string literal", line=t_line, column=t_col)
            i += 1
            col += 1
            push("STRING", "".join(buf), t_line, t_col)
            continue

        punct = {
            "{": "LBRACE",
            "}": "RBRACE",
            "(": "LPAREN",
            ")": "RPAREN",
            ",": "COMMA",
            ":": "COLON",
            ".": "DOT",
        }
        ops2 = {
            "==": "EQ",
            "!=": "NE",
            "<=": "LE",
            ">=": "GE",
        }
        two = input[i : i + 2]
        if two in ops2:
            push(ops2[two], two, t_line, t_col)
            i += 2
            col += 2
            continue
        if ch == "<":
            push("LT", ch, t_line, t_col)
            i += 1
            col += 1
            continue
        if ch == ">":
            push("GT", ch, t_line, t_col)
            i += 1
            col += 1
            continue
        if ch in punct:
            push(punct[ch], ch, t_line, t_col)
            i += 1
            col += 1
            continue

        raise DslError("DSL_PARSE_ERROR", f"Unexpected character '{ch}'", line=t_line, column=t_col)

    tokens.append(Token(type="EOF", value="", line=line, column=col))
    return tokens


def parse(tokens: list[Token]) -> Ast:
    if len(tokens) > MAX_TOKENS + 1:
        tok = tokens[MAX_TOKENS]
        raise DslError("DSL_TOO_LARGE", "Token count exceeds limit", line=tok.line, column=tok.column)
    return _Parser(tokens).parse()


def validate_ast(ast: Ast) -> None:
    count = 0

    def visit_node(node: AstNode, depth: int) -> None:
        nonlocal count
        count += 1
        if count > MAX_AST_NODES:
            raise DslError("DSL_TOO_LARGE", "AST node count exceeds limit", line=node.line, column=node.column)
        if depth > MAX_AST_DEPTH:
            raise DslError("DSL_MAX_DEPTH_EXCEEDED", "AST nesting depth exceeds limit", line=node.line, column=node.column)

        if node.kind == "Foreach":
            visit_expr(node.data["iterable"], depth + 1)
            visit_node(node.data["body"], depth + 1)
            return
        if node.kind == "If":
            visit_expr(node.data["condition"], depth + 1)
            visit_node(node.data["then"], depth + 1)
            else_branch = node.data["else"]
            if else_branch is not None:
                visit_node(else_branch, depth + 1)
            return
        if node.kind == "Emit":
            visit_expr(node.data["expr"], depth + 1)
            return
        if node.kind in {"Program", "Seq"}:
            items = node.data["body"].data["items"] if node.kind == "Program" else node.data["items"]
            for child in items:
                visit_node(child, depth + 1)
            return
        if node.kind == "Text":
            visit_expr(node.data["value"], depth + 1)
            return
        raise DslError("DSL_NOT_SUPPORTED", f"Unsupported AST node '{node.kind}'", line=node.line, column=node.column)

    def visit_expr(expr: Expr, depth: int) -> None:
        nonlocal count
        count += 1
        if count > MAX_AST_NODES:
            raise DslError("DSL_TOO_LARGE", "AST node count exceeds limit", line=expr.line, column=expr.column)
        if depth > MAX_AST_DEPTH:
            raise DslError("DSL_MAX_DEPTH_EXCEEDED", "AST nesting depth exceeds limit", line=expr.line, column=expr.column)
        if expr.kind == "MapLiteral":
            for entry in expr.value:
                visit_expr(entry["value"], depth + 1)
        if expr.kind == "UnaryOp":
            visit_expr(expr.value["operand"], depth + 1)
        if expr.kind == "BinaryOp":
            visit_expr(expr.value["left"], depth + 1)
            visit_expr(expr.value["right"], depth + 1)
        if expr.kind == "Call":
            for arg in expr.value["args"]:
                visit_expr(arg, depth + 1)

    visit_node(ast.root, 1)


def get_json_pointer(value: Any, pointer_str: str) -> Any | None:
    if pointer_str == "":
        return value
    if not pointer_str.startswith("/"):
        raise DslError("DSL_INVALID_POINTER", "JSON pointer must start with '/'")

    current = value
    for raw in pointer_str.split("/")[1:]:
        token = _decode_json_pointer_segment(raw)
        if isinstance(current, dict):
            if token not in current:
                return None
            current = current[token]
            continue
        if isinstance(current, list):
            if not token.isdigit():
                return None
            index = int(token)
            if index >= len(current):
                return None
            current = current[index]
            continue
        return None
    return current


def extract_json_pointer(value: Any, pointer_str: str, expected_type: str) -> Any | None:
    extracted = get_json_pointer(value, pointer_str)
    if extracted is None:
        return None

    type_checks = {
        "string": lambda v: isinstance(v, str),
        "int": lambda v: isinstance(v, int) and not isinstance(v, bool),
        "bool": lambda v: isinstance(v, bool),
        "array": lambda v: isinstance(v, list),
        "object": lambda v: isinstance(v, dict),
    }
    if expected_type not in type_checks:
        raise DslError("DSL_NOT_SUPPORTED", f"Unsupported expected type '{expected_type}'")
    if not type_checks[expected_type](extracted):
        raise DslError("DSL_TYPE_MISMATCH", f"Expected {expected_type}, got {type(extracted).__name__}")
    return extracted


def render_placeholders(template: str, values: dict[str, str], *, max_output_bytes: int) -> str:
    parts: list[str] = []
    i = 0
    while i < len(template):
        ch = template[i]
        if ch != "{":
            parts.append(ch)
            i += 1
            continue
        end = template.find("}", i + 1)
        if end == -1:
            raise DslError("DSL_PARSE_ERROR", "Unclosed placeholder")
        key = template[i + 1 : end].strip()
        if key not in _ALLOWED_PLACEHOLDERS:
            raise DslError("DSL_UNKNOWN_PLACEHOLDER", f"Unknown placeholder '{key}'")
        parts.append(values.get(key, ""))
        i = end + 1
    rendered = "".join(parts)
    if len(rendered.encode("utf-8")) > max_output_bytes:
        raise DslError("DSL_OUTPUT_TOO_LARGE", "Rendered output exceeds max_output_bytes")
    return rendered


def _decode_json_pointer_segment(segment: str) -> str:
    out: list[str] = []
    i = 0
    while i < len(segment):
        ch = segment[i]
        if ch != "~":
            out.append(ch)
            i += 1
            continue
        if i + 1 >= len(segment):
            raise DslError("DSL_INVALID_POINTER", "Invalid JSON pointer escape")
        nxt = segment[i + 1]
        if nxt == "0":
            out.append("~")
        elif nxt == "1":
            out.append("/")
        else:
            raise DslError("DSL_INVALID_POINTER", "Invalid JSON pointer escape")
        i += 2
    return "".join(out)


def _string_expr_from_token(tok: Token) -> Expr:
    placeholders = _PLACEHOLDER_RE.findall(tok.value)
    return Expr(
        kind="String",
        value={
            "text": tok.value,
            "has_placeholders": bool(placeholders),
            "placeholders": placeholders,
        },
        line=tok.line,
        column=tok.column,
    )


def execute_ast(ast: Ast, *, context: ExecutionContext | None = None, limits: ExecutionLimits | None = None) -> str:
    exec_ctx = context or ExecutionContext(values={})
    exec_limits = limits or ExecutionLimits()
    return _Executor(ast=ast, context=exec_ctx, limits=exec_limits).run()


def evaluate(source: str, *, context: ExecutionContext | None = None, limits: ExecutionLimits | None = None) -> str:
    ast = parse(tokenize(source))
    validate_ast(ast)
    return execute_ast(ast, context=context, limits=limits)


def _truthy(value: Any) -> bool:
    if isinstance(value, bool):
        return value
    if value is None:
        return False
    if isinstance(value, (int, float)):
        return value != 0
    if isinstance(value, str):
        return value != ""
    if isinstance(value, (list, dict)):
        return len(value) > 0
    raise DslError("DSL_TYPE_MISMATCH", f"Unsupported condition type '{type(value).__name__}'")


def _to_string(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float, str)):
        return str(value)
    if isinstance(value, (list, dict)):
        return json.dumps(value, separators=(",", ":"), ensure_ascii=False)
    raise DslError("DSL_TYPE_MISMATCH", f"Value cannot be stringified: {type(value).__name__}")
