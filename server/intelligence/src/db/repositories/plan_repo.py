"""Repository for meal-plan and shopping-list persistence.

Covers meal_plans, meal_plan_items, pantry_items, shopping_lists,
and shopping_list_items tables.
"""

from __future__ import annotations

from datetime import date
from typing import Any

import asyncpg
import structlog

logger = structlog.get_logger()


async def get_pantry_items(
    pool: asyncpg.Pool,
    user_id: str,
) -> list[dict[str, Any]]:
    """Return all pantry items for a user, with ingredient names joined."""
    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT pi.*,
                   i.name AS ingredient_name
              FROM pantry_items pi
              JOIN ingredients i ON pi.ingredient_id = i.id
             WHERE pi.user_id = $1
            """,
            user_id,
        )
    return [dict(r) for r in rows]


async def create_meal_plan(
    pool: asyncpg.Pool,
    user_id: str,
    name: str,
    start_date: date,
    end_date: date,
    items: list[dict[str, Any]],
) -> str:
    """Insert a meal plan header and its items in a single transaction.

    Each dict in *items* must contain at minimum:
    ``plan_date``, ``meal_type``, ``recipe_id``, ``servings``.

    Returns:
        The UUID of the newly created meal plan (as a string).
    """
    async with pool.acquire() as conn:
        async with conn.transaction():
            row = await conn.fetchrow(
                """
                INSERT INTO meal_plans (user_id, name, start_date, end_date)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                """,
                user_id,
                name,
                start_date,
                end_date,
            )
            plan_id = str(row["id"])

            if items:
                await conn.executemany(
                    """
                    INSERT INTO meal_plan_items
                        (meal_plan_id, plan_date, meal_type, recipe_id, servings)
                    VALUES ($1, $2, $3, $4, $5)
                    """,
                    [
                        (
                            plan_id,
                            item["plan_date"],
                            item["meal_type"],
                            item["recipe_id"],
                            item.get("servings", 1.0),
                        )
                        for item in items
                    ],
                )

    logger.info(
        "meal_plan_created",
        plan_id=plan_id,
        user_id=user_id,
        item_count=len(items),
    )
    return plan_id


async def create_shopping_list(
    pool: asyncpg.Pool,
    user_id: str,
    name: str,
    items: list[dict[str, Any]],
) -> str:
    """Insert a shopping list header and its items in a single transaction.

    Each dict in *items* must contain at minimum:
    ``ingredient_id``, ``quantity``, ``unit``.

    Returns:
        The UUID of the newly created shopping list (as a string).
    """
    async with pool.acquire() as conn:
        async with conn.transaction():
            row = await conn.fetchrow(
                """
                INSERT INTO shopping_lists (user_id, name)
                VALUES ($1, $2)
                RETURNING id
                """,
                user_id,
                name,
            )
            list_id = str(row["id"])

            if items:
                await conn.executemany(
                    """
                    INSERT INTO shopping_list_items
                        (shopping_list_id, ingredient_id, quantity, unit)
                    VALUES ($1, $2, $3, $4)
                    """,
                    [
                        (
                            list_id,
                            item["ingredient_id"],
                            item["quantity"],
                            item.get("unit", "g"),
                        )
                        for item in items
                    ],
                )

    logger.info(
        "shopping_list_created",
        list_id=list_id,
        user_id=user_id,
        item_count=len(items),
    )
    return list_id


async def get_recent_meal_plans(
    pool: asyncpg.Pool,
    user_id: str,
    limit: int = 3,
) -> list[dict[str, Any]]:
    """Fetch the most recent meal plans for variety checking."""
    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT *
              FROM meal_plans
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT $2
            """,
            user_id,
            limit,
        )
    return [dict(r) for r in rows]
