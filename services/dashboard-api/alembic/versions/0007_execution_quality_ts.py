"""execution quality timeseries

Revision ID: 0007_execution_quality_ts
Revises: 0006_cost_ledger
Create Date: 2026-02-11 04:00:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0007_execution_quality_ts"
down_revision: Union[str, Sequence[str], None] = "0006_cost_ledger"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "execution_quality_ts",
        sa.Column("eq_id", sa.String(length=36), nullable=False),
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("symbol", sa.String(length=64), nullable=False),
        sa.Column("slippage_bps", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("latency_ms", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("fill_ratio", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("timestamp", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("eq_id"),
    )


def downgrade() -> None:
    op.drop_table("execution_quality_ts")
