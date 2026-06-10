"""Nutrition Agent Node for the ForkFit intelligence pipeline.

Calculates daily calorie/macro targets and scores recipes against them.
"""
from __future__ import annotations

from typing import Any
import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import GraphState, NutritionResult, RecipeNutritionScore
from src.db.repositories.recipe_repo import get_recipe_with_ingredients
from src.services.nutrition_math import (
    calculate_bmr,
    calculate_macro_split,
    calculate_recipe_nutrition,
    calculate_tdee,
    score_plan_adherence,
)

logger = structlog.get_logger()


async def nutrition_node(state: GraphState) -> dict[str, NutritionResult]:
    """Calculate target macros and score candidate recipes."""
    if "nutrition" not in state.get("required_agents", []):
        logger.info("nutrition_node_skipped")
        return {}

    logger.info("nutrition_node_started", user_id=state.get("user_id"))
    timer = Timer()

    pool = state["_pool"]
    callback = state.get("_step_callback")

    context = state.get("context", {})
    candidate_recipes = context.get("candidate_recipes", [])

    # Get active goal type (e.g. muscle_gain, weight_loss)
    active_goals = context.get("active_goals", [])
    goal = "maintenance"
    for g in active_goals:
        goal_type = g.get("target_type") or g.get("goal_type")
        if goal_type:
            goal = str(goal_type)
            break

    # Determine daily targets
    macro_targets = context.get("macro_targets")
    if macro_targets:
        daily_targets = {
            "calories": float(macro_targets.get("calories", 2000.0)),
            "protein_g": float(macro_targets.get("protein_g", 120.0)),
            "carbs_g": float(macro_targets.get("carbs_g", 200.0)),
            "fat_g": float(macro_targets.get("fat_g", 65.0)),
        }
    else:
        # Calculate using BMR/TDEE
        weight_kg = context.get("weight_kg") or 70.0
        height_cm = context.get("height_cm") or 170.0
        age = context.get("age") or 30
        gender = context.get("gender") or "male"
        activity_level = context.get("activity_level") or "moderate"

        bmr = calculate_bmr(weight_kg, height_cm, age, gender)
        tdee = calculate_tdee(bmr, activity_level)
        daily_targets = calculate_macro_split(tdee, goal)

    input_data = {
        "goal": goal,
        "candidate_recipes_count": len(candidate_recipes),
        "macro_targets_provided": macro_targets is not None,
    }

    recipe_nutrition_scores: list[RecipeNutritionScore] = []

    with timer:
        for recipe in candidate_recipes:
            recipe_id = recipe.get("recipe_id")
            if not recipe_id:
                continue

            recipe_id_str = str(recipe_id)
            full_recipe = await get_recipe_with_ingredients(pool, recipe_id_str)
            if not full_recipe:
                continue

            # Calculate nutrition breakdown of recipe
            breakdown = calculate_recipe_nutrition(full_recipe.get("ingredients", []))
            
            # Score adherence (scale recipe to meet calories roughly or score relative macro balance)
            score = score_plan_adherence(breakdown, daily_targets)

            recipe_nutrition_scores.append({
                "recipe_id": recipe_id_str,
                "score": score,
                "breakdown": breakdown,
            })

    # Sort scores descending
    recipe_nutrition_scores.sort(key=lambda x: x["score"], reverse=True)

    output_data = {
        "daily_targets": daily_targets,
        "recipe_nutrition_scores_count": len(recipe_nutrition_scores),
    }

    await emit_step(
        callback,
        agent_name="Nutrition",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "nutrition_result": {
            "daily_targets": daily_targets,
            "recipe_nutrition_scores": recipe_nutrition_scores,
        }
    }
