"""Repository for agent memory persistence and retrieval.

Covers agent_memories and agent_memory_embeddings tables.
"""

from __future__ import annotations

from typing import Any

import asyncpg
import structlog

logger = structlog.get_logger()


async def get_active_memories(
    pool: asyncpg.Pool,
    user_id: str,
) -> list[dict[str, Any]]:
    """Return all active memories for a user, ordered by importance."""
    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT *
              FROM agent_memories
             WHERE user_id = $1
               AND is_active = true
             ORDER BY importance DESC, updated_at DESC
            """,
            user_id,
        )
    return [dict(r) for r in rows]


async def create_memory(
    pool: asyncpg.Pool,
    user_id: str,
    memory_type: str,
    content: str,
    confidence: float = 1.0,
    importance: int = 5,
) -> str:
    """Insert a new agent memory and return its ID.

    Args:
        pool: Database connection pool.
        user_id: Owning user.
        memory_type: One of ``'preference'``, ``'restriction'``, ``'habit'``.
        content: Free-text description of the memory.
        confidence: Model confidence in this memory (0.0–1.0).
        importance: Importance ranking (1–10).

    Returns:
        The UUID of the newly created memory (as a string).
    """
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            """
            INSERT INTO agent_memories
                (user_id, memory_type, content, confidence, importance)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            """,
            user_id,
            memory_type,
            content,
            confidence,
            importance,
        )

    memory_id = str(row["id"])
    logger.info(
        "memory_created",
        memory_id=memory_id,
        user_id=user_id,
        memory_type=memory_type,
    )
    return memory_id


async def create_memory_embedding(
    pool: asyncpg.Pool,
    memory_id: str,
    embedding: list[float],
    chunk_text: str,
) -> None:
    """Store a vector embedding for an existing memory.

    The embedding list is formatted to pgvector's string representation
    before insertion.
    """
    embedding_str = _format_embedding(embedding)

    async with pool.acquire() as conn:
        await conn.execute(
            """
            INSERT INTO agent_memory_embeddings
                (memory_id, embedding, chunk_text)
            VALUES ($1, $2::vector, $3)
            """,
            memory_id,
            embedding_str,
            chunk_text,
        )

    logger.info("memory_embedding_created", memory_id=memory_id)


async def update_memory_confidence(
    pool: asyncpg.Pool,
    memory_id: str,
    confidence: float,
) -> None:
    """Update the confidence score of an existing memory."""
    async with pool.acquire() as conn:
        await conn.execute(
            """
            UPDATE agent_memories
               SET confidence = $2,
                   updated_at = now()
             WHERE id = $1
            """,
            memory_id,
            confidence,
        )

    logger.info(
        "memory_confidence_updated",
        memory_id=memory_id,
        confidence=confidence,
    )


def _format_embedding(embedding: list[float]) -> str:
    """Convert a Python float list to pgvector's string format.

    Example::

        >>> _format_embedding([0.1, 0.2, 0.3])
        '[0.1,0.2,0.3]'
    """
    inner = ",".join(str(v) for v in embedding)
    return f"[{inner}]"
