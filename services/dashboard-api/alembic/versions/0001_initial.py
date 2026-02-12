"""initial schema

Revision ID: 0001_initial
Revises:
Create Date: 2026-02-11 00:00:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0001_initial"
down_revision: Union[str, Sequence[str], None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "bots",
        sa.Column("bot_id", sa.String(length=64), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("strategy_name", sa.String(length=255), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("bot_id"),
    )

    op.create_table(
        "instances",
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("bot_id", sa.String(length=64), nullable=False),
        sa.Column("runtime_mode", sa.String(length=32), nullable=False),
        sa.Column("exchange", sa.String(length=64), nullable=False),
        sa.Column("symbol", sa.String(length=64), nullable=False),
        sa.Column("status", sa.String(length=32), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["bot_id"], ["bots.bot_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("instance_id"),
    )

    op.create_table(
        "audit_logs",
        sa.Column("audit_id", sa.String(length=36), nullable=False),
        sa.Column("actor", sa.String(length=255), nullable=False),
        sa.Column("action", sa.String(length=255), nullable=False),
        sa.Column("target_type", sa.String(length=64), nullable=False),
        sa.Column("target_id", sa.String(length=64), nullable=False),
        sa.Column("result", sa.String(length=32), nullable=False),
        sa.Column("details", sa.JSON(), nullable=False),
        sa.Column("timestamp", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("audit_id"),
    )

    op.create_table(
        "modules",
        sa.Column("module_id", sa.String(length=36), nullable=False),
        sa.Column("name", sa.String(length=255), nullable=False),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("execution_mode", sa.String(length=32), nullable=False),
        sa.Column("schedule_cron", sa.String(length=128), nullable=True),
        sa.Column("config", sa.JSON(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("module_id"),
    )

    op.create_table(
        "bot_status",
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("bot_id", sa.String(length=64), nullable=False),
        sa.Column("runtime_mode", sa.String(length=32), nullable=False),
        sa.Column("exchange", sa.String(length=64), nullable=False),
        sa.Column("symbol", sa.String(length=64), nullable=False),
        sa.Column("version", sa.String(length=64), nullable=False),
        sa.Column("last_seen", sa.DateTime(timezone=True), nullable=False),
        sa.Column("metadata_json", sa.JSON(), nullable=False),
        sa.ForeignKeyConstraint(["bot_id"], ["bots.bot_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("instance_id"),
    )

    op.create_table(
        "module_runs",
        sa.Column("run_id", sa.String(length=36), nullable=False),
        sa.Column("module_id", sa.String(length=36), nullable=False),
        sa.Column("trigger_type", sa.String(length=32), nullable=False),
        sa.Column("status", sa.String(length=32), nullable=False),
        sa.Column("started_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("ended_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("summary", sa.JSON(), nullable=True),
        sa.ForeignKeyConstraint(["module_id"], ["modules.module_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("run_id"),
    )


def downgrade() -> None:
    op.drop_table("module_runs")
    op.drop_table("bot_status")
    op.drop_table("modules")
    op.drop_table("audit_logs")
    op.drop_table("instances")
    op.drop_table("bots")
