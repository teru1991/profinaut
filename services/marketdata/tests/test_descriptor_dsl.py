import pytest

from services.marketdata.app.descriptor_dsl import DslError, parse, tokenize, validate_ast


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
