"""Initial schema

Revision ID: 001
Revises:
Create Date: 2026-02-11 12:00:00.000000

"""
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

from alembic import op

# revision identifiers
revision = '001'
down_revision = None
branch_labels = None
depends_on = None


def upgrade() -> None:
    # Create bot_configs table
    op.create_table(
        'bot_configs',
        sa.Column('id', sa.Integer(), nullable=False),
        sa.Column('bot_id', sa.String(), nullable=False),
        sa.Column('bot_type', sa.String(), nullable=False),
        sa.Column('enabled', sa.Boolean(), nullable=True),
        sa.Column('config', postgresql.JSON(astext_type=sa.Text()), nullable=False),
        sa.Column('created_at', sa.DateTime(), nullable=True),
        sa.Column('updated_at', sa.DateTime(), nullable=True),
        sa.PrimaryKeyConstraint('id')
    )
    op.create_index(op.f('ix_bot_configs_bot_id'), 'bot_configs', ['bot_id'], unique=True)
    op.create_index(op.f('ix_bot_configs_id'), 'bot_configs', ['id'], unique=False)

    # Create market_data table
    op.create_table(
        'market_data',
        sa.Column('id', sa.Integer(), nullable=False),
        sa.Column('symbol', sa.String(), nullable=False),
        sa.Column('data_type', sa.String(), nullable=False),
        sa.Column('timestamp', sa.DateTime(), nullable=False),
        sa.Column('value', postgresql.JSON(astext_type=sa.Text()), nullable=False),
        sa.Column('source', sa.String(), nullable=True),
        sa.Column('created_at', sa.DateTime(), nullable=True),
        sa.PrimaryKeyConstraint('id')
    )
    op.create_index(op.f('ix_market_data_id'), 'market_data', ['id'], unique=False)
    op.create_index(op.f('ix_market_data_symbol'), 'market_data', ['symbol'], unique=False)
    op.create_index(op.f('ix_market_data_timestamp'), 'market_data', ['timestamp'], unique=False)


def downgrade() -> None:
    op.drop_index(op.f('ix_market_data_timestamp'), table_name='market_data')
    op.drop_index(op.f('ix_market_data_symbol'), table_name='market_data')
    op.drop_index(op.f('ix_market_data_id'), table_name='market_data')
    op.drop_table('market_data')
    op.drop_index(op.f('ix_bot_configs_id'), table_name='bot_configs')
    op.drop_index(op.f('ix_bot_configs_bot_id'), table_name='bot_configs')
    op.drop_table('bot_configs')
