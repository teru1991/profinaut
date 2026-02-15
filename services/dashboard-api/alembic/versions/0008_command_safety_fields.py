"""command safety reason and expiry fields

Revision ID: 0008_command_safety_fields
Revises: 0007_execution_quality_ts
Create Date: 2026-02-15 00:00:00

"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


revision: str = "0008_command_safety_fields"
down_revision: Union[str, Sequence[str], None] = "0007_execution_quality_ts"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column("commands", sa.Column("reason", sa.Text(), nullable=True))
    op.add_column("commands", sa.Column("expires_at", sa.DateTime(timezone=True), nullable=True))


def downgrade() -> None:
    op.drop_column("commands", "expires_at")
    op.drop_column("commands", "reason")
