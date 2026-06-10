"""Repository for recipe-related database queries.

Covers recipes, recipe_ingredients, and ingredients tables.
"""

from __future__ import annotations

from typing import Any

import asyncpg
import structlog

logger = structlog.get_logger()


async def get_recipe_with_ingredients(
    pool: asyncpg.Pool,
    recipe_id: str,
) -> dict[str, Any] | None:
    """Fetch a recipe joined with its ingredients and per-100g nutrition.

    Returns the recipe dict with a nested ``ingredients`` list, each
    containing ingredient details and ``grams_equivalent``.
    """
    async with pool.acquire() as conn:
        recipe_row = await conn.fetchrow(
            "SELECT * FROM recipes WHERE id = $1",
            recipe_id,
        )
        if recipe_row is None:
            return None

        ingredient_rows = await conn.fetch(
            """
            SELECT ri.ingredient_id,
                   ri.grams_equivalent,
                   ri.notes,
                   i.name              AS ingredient_name,
                   i.calories_per_100g,
                   i.protein_per_100g,
                   i.carbs_per_100g,
                   i.fat_per_100g,
                   i.fiber_per_100g,
                   i.estimated_cost_per_100g
              FROM recipe_ingredients ri
              JOIN ingredients i ON ri.ingredient_id = i.id
             WHERE ri.recipe_id = $1
            """,
            recipe_id,
        )

    result = dict(recipe_row)
    result["ingredients"] = [dict(r) for r in ingredient_rows]
    return result


async def get_recipes_by_ids(
    pool: asyncpg.Pool,
    recipe_ids: list[str],
) -> list[dict[str, Any]]:
    """Batch-fetch recipes by their IDs."""
    if not recipe_ids:
        return []

    async with pool.acquire() as conn:
        rows = await conn.fetch(
            "SELECT * FROM recipes WHERE id = ANY($1::uuid[])",
            recipe_ids,
        )
    return [dict(r) for r in rows]


async def calculate_recipe_cost(
    pool: asyncpg.Pool,
    recipe_id: str,
) -> float:
    """Calculate the total ingredient cost for a recipe.

    Cost is derived from ``estimated_cost_per_100g`` scaled by
    ``grams_equivalent`` for each ingredient.
    """
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            """
            SELECT COALESCE(
                       SUM(i.estimated_cost_per_100g * ri.grams_equivalent / 100.0),
                       0
                   ) AS total_cost
              FROM recipe_ingredients ri
              JOIN ingredients i ON ri.ingredient_id = i.id
             WHERE ri.recipe_id = $1
            """,
            recipe_id,
        )
    return float(row["total_cost"]) if row else 0.0


async def get_recipe_allergen_ingredients(
    pool: asyncpg.Pool,
    recipe_id: str,
    allergen_names: list[str],
) -> list[dict[str, Any]]:
    """Find ingredients in a recipe that match any of the given allergens.

    Matching is case-insensitive via ``ILIKE`` against the ingredient name.
    """
    if not allergen_names:
        return []

    # Build an OR-chain of ILIKE patterns.
    conditions = " OR ".join(
        f"i.name ILIKE ${idx + 2}" for idx in range(len(allergen_names))
    )
    patterns = [f"%{name}%" for name in allergen_names]

    query = f"""
        SELECT ri.ingredient_id,
               i.name AS ingredient_name,
               ri.grams_equivalent
          FROM recipe_ingredients ri
          JOIN ingredients i ON ri.ingredient_id = i.id
         WHERE ri.recipe_id = $1
           AND ({conditions})
    """

    async with pool.acquire() as conn:
        rows = await conn.fetch(query, recipe_id, *patterns)
    return [dict(r) for r in rows]


async def search_recipes_by_cuisine(
    pool: asyncpg.Pool,
    cuisine: str,
    dietary_tags: list[str] | None = None,
    limit: int = 20,
) -> list[dict[str, Any]]:
    """Filter recipes by cuisine and (optionally) dietary-tag overlap.

    When *dietary_tags* is provided the recipe's ``dietary_tags`` array
    must contain **all** of the requested tags (``@>`` operator).
    """
    if dietary_tags:
        async with pool.acquire() as conn:
            rows = await conn.fetch(
                """
                SELECT *
                  FROM recipes
                 WHERE cuisine ILIKE $1
                   AND dietary_tags @> $2::text[]
                 LIMIT $3
                """,
                f"%{cuisine}%",
                dietary_tags,
                limit,
            )
    else:
        async with pool.acquire() as conn:
            rows = await conn.fetch(
                """
                SELECT *
                  FROM recipes
                 WHERE cuisine ILIKE $1
                 LIMIT $2
                """,
                f"%{cuisine}%",
                limit,
            )
    return [dict(r) for r in rows]
