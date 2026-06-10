"""Async connection-pool lifecycle management for PostgreSQL."""

from __future__ import annotations

import asyncpg
import structlog

logger = structlog.get_logger()


async def create_pool(
    database_url: str,
    *,
    min_size: int = 5,
    max_size: int = 20,
) -> asyncpg.Pool:
    """Create and return an asyncpg connection pool.

    A lightweight ``SELECT 1`` probe is executed immediately after creation
    to verify that the database is reachable.

    Args:
        database_url: Full PostgreSQL DSN.
        min_size: Minimum number of connections to keep open.
        max_size: Maximum number of connections in the pool.

    Returns:
        A ready-to-use :class:`asyncpg.Pool`.
    """
    logger.info("creating_db_pool", min_size=min_size, max_size=max_size)

    pool: asyncpg.Pool = await asyncpg.create_pool(
        database_url,
        min_size=min_size,
        max_size=max_size,
        command_timeout=30,
    )

    # Verify connectivity early so we fail fast on misconfiguration.
    async with pool.acquire() as conn:
        await conn.execute("SELECT 1")

    logger.info("db_pool_created")
    return pool


async def close_pool(pool: asyncpg.Pool) -> None:
    """Gracefully close the connection pool.

    All idle connections are released and the pool will no longer hand
    out new connections after this call.
    """
    logger.info("closing_db_pool")
    await pool.close()
    logger.info("db_pool_closed")
