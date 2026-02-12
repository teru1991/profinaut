"""alerts table

Revision ID: 0003_alerts
Revises: 0002_commands_and_acks
Create Date: 2026-02-11 00:45:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0003_alerts"
down_revision: Union[str, Sequence[str], None] = "0002_commands_and_acks"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "alerts",
        sa.Column("alert_id", sa.String(length=36), nullable=False),
        sa.Column("source", sa.String(length=64), nullable=False),
        sa.Column("severity", sa.String(length=16), nullable=False),
        sa.Column("message", sa.Text(), nullable=False),
        sa.Column("target_type", sa.String(length=64), nullable=False),
        sa.Column("target_id", sa.String(length=64), nullable=False),
        sa.Column("status", sa.String(length=16), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("last_notified_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("metadata_json", sa.JSON(), nullable=False),
        sa.PrimaryKeyConstraint("alert_id"),
    )


def downgrade() -> None:
    op.drop_table("alerts")
