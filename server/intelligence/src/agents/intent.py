"""Intent classification node for the ForkFit Intelligence Service.

Extracts structured intent from raw natural language prompts.
"""
from __future__ import annotations

import structlog
from src.config import Settings
from src.models.intent import IntentBlueprint
from src.services.llm import get_chat_model

logger = structlog.get_logger()


async def classify_intent(
    prompt: str,
    user_id: str,
    settings: Settings,
) -> IntentBlueprint:
    """Classify a raw user prompt into a structured intent using LLM."""
    logger.info("classify_intent_started", user_id=user_id, prompt=prompt[:80])

    llm = get_chat_model(settings)
    structured_llm = llm.with_structured_output(IntentBlueprint)

    system_prompt = (
        "You are a nutritional assistant for ForkFit. Your job is to extract structured intent "
        "from the user's natural language prompt.\n"
        "Extract the goal (e.g. weight_loss, muscle_gain, fat_loss, maintenance, calorie_surplus), "
        "diet (e.g. vegetarian, vegan, omnivore, keto, pescatarian, jain), budget limit, budget currency, "
        "timeline (daily or weekly), and any constraints (e.g. 'no eggs', 'high protein').\n"
        "Provide a detailed raw_analysis dict containing your reasoning."
    )

    try:
        result = await structured_llm.ainvoke([
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": f"User Prompt: {prompt}"}
        ])

        # If raw_analysis is empty or not set, populate it
        if not result.raw_analysis:
            result.raw_analysis = {
                "prompt": prompt,
                "user_id": user_id,
                "goal": result.goal,
                "diet": result.diet,
                "budget_limit": result.budget_limit,
                "budget_currency": result.budget_currency,
                "timeline": result.timeline,
                "constraints": result.constraints,
            }

        logger.info("classify_intent_success", user_id=user_id, goal=result.goal, diet=result.diet)
        return result
    except Exception as exc:
        logger.error("classify_intent_failed", error=str(exc))
        # Fallback intent if LLM fails
        return IntentBlueprint(
            goal="maintenance",
            diet="omnivore",
            budget_limit=0.0,
            budget_currency="INR",
            timeline="weekly",
            constraints=[],
            raw_analysis={"error": str(exc)}
        )
