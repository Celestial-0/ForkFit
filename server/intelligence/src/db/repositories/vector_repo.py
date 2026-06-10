"""Repository for pgvector cosine-similarity searches.

Covers recipe_embeddings, ingredient_embeddings, and
agent_memory_embeddings tables.
"""

from __future__ import annotations

from typing import Any

import asyncpg
import structlog

logger = structlog.get_logger()


# ── Public search functions ───────────────────────────────────────────────


async def search_recipes_by_embedding(
    pool: asyncpg.Pool,
    embedding: list[float],
    limit: int = 10,
) -> list[dict[str, Any]]:
    """Find recipes closest to *embedding* using cosine distance.

    Returns recipe metadata together with a ``distance`` score (lower is
    more similar).
    """
    embedding_str = _format_embedding(embedding)

    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT re.recipe_id,
                   r.title,
                   r.cuisine,
                   r.dietary_tags,
                   (re.embedding <=> $1::vector) AS distance
              FROM recipe_embeddings re
              JOIN recipes r ON re.recipe_id = r.id
             ORDER BY distance
             LIMIT $2
            """,
            embedding_str,
            limit,
        )
    return [dict(r) for r in rows]


async def search_ingredients_by_embedding(
    pool: asyncpg.Pool,
    embedding: list[float],
    limit: int = 10,
) -> list[dict[str, Any]]:
    """Find ingredients closest to *embedding* using cosine distance."""
    embedding_str = _format_embedding(embedding)

    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT ie.ingredient_id,
                   i.name,
                   i.category,
                   (ie.embedding <=> $1::vector) AS distance
              FROM ingredient_embeddings ie
              JOIN ingredients i ON ie.ingredient_id = i.id
             ORDER BY distance
             LIMIT $2
            """,
            embedding_str,
            limit,
        )
    return [dict(r) for r in rows]


async def search_memories_by_embedding(
    pool: asyncpg.Pool,
    user_id: str,
    embedding: list[float],
    limit: int = 5,
) -> list[dict[str, Any]]:
    """Find a user's active memories closest to *embedding*.

    Only memories with ``is_active = true`` are considered.
    """
    embedding_str = _format_embedding(embedding)

    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT am.*,
                   ame.chunk_text,
                   (ame.embedding <=> $1::vector) AS distance
              FROM agent_memory_embeddings ame
              JOIN agent_memories am ON ame.memory_id = am.id
             WHERE am.user_id = $2
               AND am.is_active = true
             ORDER BY distance
             LIMIT $3
            """,
            embedding_str,
            user_id,
            limit,
        )
    return [dict(r) for r in rows]


# ── Helpers ───────────────────────────────────────────────────────────────


def _format_embedding(embedding: list[float]) -> str:
    """Convert a Python float list to pgvector's string format.

    Example::

        >>> _format_embedding([0.1, 0.2, 0.3])
        '[0.1,0.2,0.3]'

    asyncpg sends this string as-is and the ``::vector`` cast in the SQL
    query tells PostgreSQL to parse it into a pgvector value.
    """
    inner = ",".join(str(v) for v in embedding)
    return f"[{inner}]"
