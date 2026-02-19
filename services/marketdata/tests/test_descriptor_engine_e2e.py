import pytest

from services.marketdata.app.descriptor_dsl import ExecContext, ExecError, ExecutionLimits, execute_descriptor


def test_execute_descriptor_e2e_happy_path() -> None:
    template = '''
emit "market={{symbol}}|"
foreach item in items {
  if item.enabled {
    emit json("/events/0/id")
    emit "-{{item.name}};"
  } else {
    emit "skip;"
  }
}
'''
    result = execute_descriptor(
        template,
        ExecContext(
            values={
                "symbol": "BTC_JPY",
                "items": [
                    {"enabled": True, "name": "alpha"},
                    {"enabled": False, "name": "beta"},
                ],
                "payload": {"events": [{"id": "evt-1"}]},
            }
        ),
    )

    assert result.output == "market=BTC_JPY|evt-1-alpha;skip;"
    assert result.metrics.steps > 0
    assert result.metrics.output_bytes == len(result.output.encode("utf-8"))
    assert result.metrics.loops == 2
    assert result.metrics.emits == 4


@pytest.mark.parametrize(
    ("template", "ctx", "limits", "error_code"),
    [
        (
            'foreach item in items { emit "x" }',
            ExecContext(values={"items": [1, 2, 3]}),
            ExecutionLimits(max_loop_iters=2),
            "DSL_EXEC_LIMIT",
        ),
        (
            'foreach item in items { emit "x" }',
            ExecContext(values={"items": [1, 2, 3]}),
            ExecutionLimits(max_output_bytes=2),
            "DSL_OUTPUT_LIMIT",
        ),
        (
            'emit "{{missing}}"',
            ExecContext(values={}),
            None,
            "DSL_UNKNOWN_PLACEHOLDER",
        ),
        (
            'emit json("bad")',
            ExecContext(values={"payload": {"a": 1}}),
            None,
            "DSL_INVALID_POINTER",
        ),
    ],
)
def test_execute_descriptor_e2e_rejects_malicious_or_invalid_templates(
    template: str,
    ctx: ExecContext,
    limits: ExecutionLimits | None,
    error_code: str,
) -> None:
    with pytest.raises(ExecError) as exc:
        execute_descriptor(template, ctx, limits=limits)
    assert exc.value.error_code == error_code


def test_execute_descriptor_output_cap_boundary_1m() -> None:
    within = execute_descriptor(
        'emit "{{blob}}"',
        ExecContext(values={"blob": "x" * 1_000_000}),
        limits=ExecutionLimits(max_output_bytes=1_000_000, max_string_len=1_000_100),
    )
    assert len(within.output.encode("utf-8")) == 1_000_000

    with pytest.raises(ExecError) as exc:
        execute_descriptor(
            'emit "{{blob}}"',
            ExecContext(values={"blob": "x" * 1_000_001}),
            limits=ExecutionLimits(max_output_bytes=1_000_000, max_string_len=1_000_100),
        )
    assert exc.value.error_code == "DSL_OUTPUT_LIMIT"
