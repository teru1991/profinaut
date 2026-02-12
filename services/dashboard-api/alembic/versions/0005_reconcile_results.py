"""reconcile results persistence

Revision ID: 0005_reconcile_results
Revises: 0004_metrics_positions
Create Date: 2026-02-11 02:00:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0005_reconcile_results"
down_revision: Union[str, Sequence[str], None] = "0004_metrics_positions"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "reconcile_results",
        sa.Column("reconcile_id", sa.String(length=36), nullable=False),
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("exchange_equity", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("internal_equity", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("difference", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("status", sa.String(length=16), nullable=False),
        sa.Column("timestamp", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("reconcile_id"),
    )


def downgrade() -> None:
    op.drop_table("reconcile_results")
