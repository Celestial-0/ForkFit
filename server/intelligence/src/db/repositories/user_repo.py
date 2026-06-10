"""Repository for user-related database queries.

Covers profiles, medical_safety_profiles, biometric_logs,
workout_logs, user_goals, and user_preferences tables.
"""

from __future__ import annotations

from typing import Any

import asyncpg
import structlog

logger = structlog.get_logger()


async def get_user_profile(
    pool: asyncpg.Pool,
    user_id: str,
) -> dict[str, Any] | None:
    """Fetch basic profile fields (gender, date-of-birth) for a user."""
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            "SELECT gender, dob FROM profiles WHERE user_id = $1",
            user_id,
        )
    if row is None:
        return None
    return dict(row)


async def get_medical_safety_profile(
    pool: asyncpg.Pool,
    user_id: str,
) -> dict[str, Any] | None:
    """Fetch allergy, medical-condition, and pregnancy/lactation flags."""
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            """
            SELECT allergies,
                   medical_conditions,
                   is_pregnant,
                   is_lactating
              FROM medical_safety_profiles
             WHERE user_id = $1
            """,
            user_id,
        )
    if row is None:
        return None
    return dict(row)


async def get_user_goals(
    pool: asyncpg.Pool,
    user_id: str,
) -> list[dict[str, Any]]:
    """Return all active goals for the given user."""
    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT *
              FROM user_goals
             WHERE user_id = $1
               AND is_active = true
            """,
            user_id,
        )
    return [dict(r) for r in rows]


async def get_latest_biometrics(
    pool: asyncpg.Pool,
    user_id: str,
) -> dict[str, Any]:
    """Get the latest weight, height, and body-fat readings.

    Uses ``DISTINCT ON (metric_type)`` to pick the most recent log per
    metric, then pivots the rows into a flat dict.
    """
    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT DISTINCT ON (metric_type)
                   metric_type,
                   value
              FROM biometric_logs
             WHERE user_id = $1
             ORDER BY metric_type, logged_at DESC
            """,
            user_id,
        )

    result: dict[str, Any] = {}
    for row in rows:
        metric = row["metric_type"]
        if metric == "weight_kg":
            result["weight_kg"] = row["value"]
        elif metric == "height_cm":
            result["height_cm"] = row["value"]
        elif metric == "body_fat_pct":
            result["body_fat_pct"] = row["value"]
    return result


async def get_recent_workouts(
    pool: asyncpg.Pool,
    user_id: str,
    days: int = 7,
) -> list[dict[str, Any]]:
    """Return workout logs from the last *days* days, most-recent first."""
    async with pool.acquire() as conn:
        rows = await conn.fetch(
            """
            SELECT *
              FROM workout_logs
             WHERE user_id = $1
               AND logged_at >= now() - make_interval(days => $2)
             ORDER BY logged_at DESC
            """,
            user_id,
            days,
        )
    return [dict(r) for r in rows]


async def get_user_preferences(
    pool: asyncpg.Pool,
    user_id: str,
) -> dict[str, Any] | None:
    """Fetch the JSONB preferences blob for a user."""
    async with pool.acquire() as conn:
        row = await conn.fetchrow(
            "SELECT preferences FROM user_preferences WHERE user_id = $1",
            user_id,
        )
    if row is None:
        return None
    return dict(row)
