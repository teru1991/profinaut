import pytest

from services.marketdata.app.descriptor_dsl import (
    DslError,
    extract_json_pointer,
    get_json_pointer,
    parse,
    render_placeholders,
    tokenize,
    validate_ast,
)


def test_descriptor_dsl_pipeline_happy_path() -> None:
    source = '''
foreach item in events {
  if item.enabled {
    emit "event={{item.id}}"
  } else {
    emit {kind: "skipped", id: item.id}
  }
}
'''
    tokens = tokenize(source)
    ast = parse(tokens)
    validate_ast(ast)


def test_descriptor_dsl_brace_mismatch_raises_parse_error() -> None:
    source = 'if flag { emit "ok"'
    with pytest.raises(DslError) as exc:
        validate_ast(parse(tokenize(source)))
    assert exc.value.code == "DSL_PARSE_ERROR"


def test_descriptor_dsl_else_without_if_raises_unexpected_token() -> None:
    source = 'else { emit "nope" }'
    with pytest.raises(DslError) as exc:
        validate_ast(parse(tokenize(source)))
    assert exc.value.code == "DSL_UNEXPECTED_TOKEN"


def test_descriptor_dsl_too_deep_nesting_rejected() -> None:
    depth = 35
    prefix = "\n".join(["if cond {"] * depth)
    suffix = "\n".join(["}"] * depth)
    source = f"{prefix}\nemit \"x\"\n{suffix}"

    with pytest.raises(DslError) as exc:
        validate_ast(parse(tokenize(source)))
    assert exc.value.code == "DSL_MAX_DEPTH_EXCEEDED"


def test_get_json_pointer_success_and_missing() -> None:
    payload = {"a": {"b/c": [0, {"ok": True}]}}
    assert get_json_pointer(payload, "/a/b~1c/1/ok") is True
    assert get_json_pointer(payload, "/a/missing") is None


def test_get_json_pointer_invalid_pointer_rejected() -> None:
    with pytest.raises(DslError) as exc:
        get_json_pointer({"a": 1}, "a")
    assert exc.value.code == "DSL_INVALID_POINTER"


def test_extract_json_pointer_type_mismatch_rejected() -> None:
    with pytest.raises(DslError) as exc:
        extract_json_pointer({"a": "1"}, "/a", "int")
    assert exc.value.code == "DSL_TYPE_MISMATCH"


def test_unknown_placeholder_rejected() -> None:
    with pytest.raises(DslError) as exc:
        render_placeholders("symbol={symbol},x={unknown}", {"symbol": "BTC"}, max_output_bytes=128)
    assert exc.value.code == "DSL_UNKNOWN_PLACEHOLDER"


def test_render_placeholder_applies_output_limit_after_substitution() -> None:
    with pytest.raises(DslError) as exc:
        render_placeholders("{symbol}", {"symbol": "X" * 20}, max_output_bytes=8)
    assert exc.value.code == "DSL_OUTPUT_TOO_LARGE"


def test_parse_json_pointer_expr_function() -> None:
    ast = parse(tokenize('emit json("/a/b")'))
    emit_expr = ast.root.data["body"].data["items"][0].data["expr"]
    assert emit_expr.kind == "JsonPointer"
    assert emit_expr.value["pointer"] == "/a/b"


def test_parse_json_pointer_expr_requires_string_arg() -> None:
    with pytest.raises(DslError) as exc:
        parse(tokenize("emit json(1)"))
    assert exc.value.code == "DSL_TYPE_MISMATCH"
