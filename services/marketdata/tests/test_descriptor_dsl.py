import pytest

from services.marketdata.app.descriptor_dsl import (
    DslError,
    ExecutionContext,
    ExecutionLimits,
    evaluate,
    parse,
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


def test_descriptor_dsl_foreach_emit_outputs_expected_content() -> None:
    source = """
foreach item in events {
  emit item.id
}
"""
    output = evaluate(source, context=ExecutionContext(values={"events": [{"id": "a"}, {"id": "b"}, {"id": "c"}]}))
    assert output == "abc"


def test_descriptor_dsl_output_limit_raises() -> None:
    source = 'emit "abcdef"'
    with pytest.raises(DslError) as exc:
        evaluate(source, limits=ExecutionLimits(max_output_bytes=5))
    assert exc.value.code == "DSL_OUTPUT_LIMIT"


def test_descriptor_dsl_step_limit_raises() -> None:
    source = """
foreach item in events {
  emit item.id
}
"""
    with pytest.raises(DslError) as exc:
        evaluate(
            source,
            context=ExecutionContext(values={"events": [{"id": "x"}, {"id": "y"}, {"id": "z"}]}),
            limits=ExecutionLimits(max_steps=5),
        )
    assert exc.value.code == "DSL_EXEC_LIMIT"
