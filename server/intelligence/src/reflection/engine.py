"""Reflection engine — learning loop for extracting user preferences from feedback.

After a user rates or reviews a meal plan, this module uses an LLM to extract
structured preferences, restrictions, and behavioural patterns.  Each extracted
insight is persisted as a ``Memory`` with a vector embedding for future
semantic retrieval.
"""
from __future__ import annotations

import json
from typing import Any, TypedDict

import asyncpg
import structlog

from src.config import Settings
from src.db.repositories.memory_repo import create_memory, create_memory_embedding
from src.services.embedding import embed_text
from src.services.llm import get_chat_model

logger = structlog.get_logger()


class ExtractedMemory(TypedDict):
    """Structured memory details extracted from user feedback."""
    type: str
    content: str
    confidence: float

_EXTRACTION_SYSTEM_PROMPT = """\
Analyze this user feedback about a meal plan. Extract specific food preferences, \
restrictions, or behavioral patterns.

Return your answer as a JSON list of objects, each with the following fields:
- "type": one of "preference", "restriction", or "habit"
- "content": a concise, first-person statement (e.g. "I prefer spicy food")
- "confidence": a float between 0.0 and 1.0 indicating how certain you are

Example output:
[
  {"type": "preference", "content": "Prefers paneer over tofu", "confidence": 0.9},
  {"type": "restriction", "content": "Avoids mushrooms", "confidence": 0.95}
]

If the feedback does not contain extractable preferences, return an empty JSON list: []
Respond ONLY with valid JSON — no markdown fences, no commentary.
"""


async def _extract_memories_via_llm(
    settings: Settings,
    feedback_text: str,
    feedback_rating: int,
) -> list[ExtractedMemory]:
    """Use the chat model to pull structured memories from raw feedback.

    Returns a list of dicts with keys ``type``, ``content``, ``confidence``.
    Falls back to an empty list if the LLM response cannot be parsed.
    """
    llm = get_chat_model(settings)

    user_message = (
        f"Feedback rating: {feedback_rating}/5\n"
        f"Feedback text: {feedback_text}"
    )

    try:
        response = await llm.ainvoke(
            [
                {"role": "system", "content": _EXTRACTION_SYSTEM_PROMPT},
                {"role": "user", "content": user_message},
            ]
        )
        raw = response.content if hasattr(response, "content") else str(response)

        # Strip markdown fences if model wraps output
        cleaned = raw.strip()
        if cleaned.startswith("```"):
            cleaned = cleaned.split("\n", 1)[-1]
        if cleaned.endswith("```"):
            cleaned = cleaned.rsplit("```", 1)[0]
        cleaned = cleaned.strip()

        memories: list[ExtractedMemory] = json.loads(cleaned)
        if not isinstance(memories, list):
            logger.warning("llm_extraction_not_list", raw=raw)
            return []
        return memories

    except (json.JSONDecodeError, TypeError, ValueError) as exc:
        logger.error("llm_extraction_failed", error=str(exc))
        return []
    except Exception as exc:  # noqa: BLE001
        logger.error("llm_extraction_unexpected_error", error=str(exc))
        return []


def _synthesise_implicit_feedback(
    feedback_rating: int,
) -> list[ExtractedMemory]:
    """Generate a generic memory when the user gives a rating but no text."""
    if feedback_rating >= 4:
        return [
            {
                "type": "habit",
                "content": "User was satisfied with the plan",
                "confidence": 0.5,
            }
        ]
    if feedback_rating <= 2:
        return [
            {
                "type": "habit",
                "content": "User was dissatisfied, plan needs improvement",
                "confidence": 0.5,
            }
        ]
    return []


async def run_reflection(
    pool: asyncpg.Pool,
    settings: Settings,
    user_id: str,
    session_id: str,
    chat_message_id: str,
    feedback_rating: int,
    feedback_text: str,
) -> list[str]:
    """Execute the reflection learning loop.

    Parameters
    ----------
    pool:
        Asyncpg connection pool.
    settings:
        Application settings (LLM configuration, etc.).
    user_id:
        Unique identifier of the user providing feedback.
    session_id:
        Chat session the feedback belongs to.
    chat_message_id:
        Specific message being rated.
    feedback_rating:
        Numeric rating (1-5).
    feedback_text:
        Free-form feedback from the user.

    Returns
    -------
    list[str]
        Content strings of each extracted memory that was persisted.
    """
    logger.info(
        "reflection_started",
        user_id=user_id,
        session_id=session_id,
        rating=feedback_rating,
    )

    # ── Extract memories from feedback text ─────────────────────────
    memories: list[ExtractedMemory]
    if feedback_text and feedback_text.strip():
        memories = await _extract_memories_via_llm(
            settings, feedback_text, feedback_rating
        )
    else:
        memories = _synthesise_implicit_feedback(feedback_rating)

    if not memories:
        logger.info("reflection_no_memories_extracted", user_id=user_id)
        return []

    # ── Persist each memory with its embedding ──────────────────────
    persisted: list[str] = []
    for mem in memories:
        mem_type: str = mem.get("type", "preference")
        content: str = mem.get("content", "")
        confidence: float = mem.get("confidence", 0.5)

        if not content:
            continue

        try:
            memory_id = await create_memory(
                pool,
                user_id=user_id,
                memory_type=mem_type,
                content=content,
                confidence=confidence,
            )

            embedding = await embed_text(content, settings)
            await create_memory_embedding(
                pool,
                memory_id=memory_id,
                embedding=embedding,
                source_text=content,
            )

            persisted.append(content)
            logger.info(
                "memory_persisted",
                memory_id=memory_id,
                memory_type=mem_type,
                content=content,
            )
        except Exception as exc:  # noqa: BLE001
            logger.error(
                "memory_persistence_failed",
                content=content,
                error=str(exc),
            )

    logger.info(
        "reflection_complete",
        user_id=user_id,
        total_extracted=len(persisted),
    )
    return persisted
