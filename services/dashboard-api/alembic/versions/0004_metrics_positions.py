"""metrics and positions foundation

Revision ID: 0004_metrics_positions
Revises: 0003_alerts
Create Date: 2026-02-11 01:00:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0004_metrics_positions"
down_revision: Union[str, Sequence[str], None] = "0003_alerts"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "metrics_ts",
        sa.Column("metric_id", sa.String(length=36), nullable=False),
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("symbol", sa.String(length=64), nullable=False),
        sa.Column("metric_type", sa.String(length=64), nullable=False),
        sa.Column("value", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("timestamp", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("metric_id"),
    )

    op.create_table(
        "positions_current",
        sa.Column("position_id", sa.String(length=36), nullable=False),
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("symbol", sa.String(length=64), nullable=False),
        sa.Column("net_exposure", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("gross_exposure", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("position_id"),
    )


def downgrade() -> None:
    op.drop_table("positions_current")
    op.drop_table("metrics_ts")
