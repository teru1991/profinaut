"""cost ledger for net pnl

Revision ID: 0006_cost_ledger
Revises: 0005_reconcile_results
Create Date: 2026-02-11 03:00:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0006_cost_ledger"
down_revision: Union[str, Sequence[str], None] = "0005_reconcile_results"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "cost_ledger",
        sa.Column("cost_id", sa.String(length=36), nullable=False),
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("symbol", sa.String(length=64), nullable=False),
        sa.Column("cost_type", sa.String(length=16), nullable=False),
        sa.Column("amount", sa.Numeric(precision=20, scale=8), nullable=False),
        sa.Column("timestamp", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("cost_id"),
    )


def downgrade() -> None:
    op.drop_table("cost_ledger")
