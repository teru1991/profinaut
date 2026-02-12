"""commands and acks

Revision ID: 0002_commands_and_acks
Revises: 0001_initial
Create Date: 2026-02-11 00:30:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0002_commands_and_acks"
down_revision: Union[str, Sequence[str], None] = "0001_initial"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "commands",
        sa.Column("command_id", sa.String(length=36), nullable=False),
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("command_type", sa.String(length=32), nullable=False),
        sa.Column("issued_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("expires_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("payload", sa.JSON(), nullable=False),
        sa.Column("status", sa.String(length=32), nullable=False),
        sa.Column("created_by", sa.String(length=255), nullable=False),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("command_id"),
    )

    op.create_table(
        "command_acks",
        sa.Column("ack_id", sa.String(length=36), nullable=False),
        sa.Column("command_id", sa.String(length=36), nullable=False),
        sa.Column("instance_id", sa.String(length=64), nullable=False),
        sa.Column("status", sa.String(length=32), nullable=False),
        sa.Column("reason", sa.Text(), nullable=True),
        sa.Column("timestamp", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["command_id"], ["commands.command_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["instance_id"], ["instances.instance_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("ack_id"),
    )


def downgrade() -> None:
    op.drop_table("command_acks")
    op.drop_table("commands")
