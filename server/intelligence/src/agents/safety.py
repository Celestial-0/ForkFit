"""Safety Agent Node for the ForkFit intelligence pipeline.

Cross-references user allergies and medical conditions against candidate recipes.
"""
from __future__ import annotations

from typing import Any
import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import BlockedRecipe, GraphState, SafetyResult
from src.db.repositories.recipe_repo import get_recipe_allergen_food_items

logger = structlog.get_logger()


async def safety_node(state: GraphState) -> dict[str, SafetyResult]:
    """Audit candidate recipes for safety constraints."""
    if "safety" not in state.get("required_agents", []):
        logger.info("safety_node_skipped")
        return {}

    logger.info("safety_node_started", user_id=state.get("user_id"))
    timer = Timer()

    pool = state["_pool"]
    callback = state.get("_step_callback")

    context = state.get("context", {})
    allergies = context.get("allergies", [])
    medical_conditions = context.get("medical_conditions", [])
    candidate_recipes = context.get("candidate_recipes", [])

    safe_recipe_ids: list[str] = []
    blocked_recipes: list[BlockedRecipe] = []
    warnings: list[str] = []

    input_data = {
        "allergies": allergies,
        "medical_conditions": medical_conditions,
        "candidate_recipes_count": len(candidate_recipes),
    }

    with timer:
        for recipe in candidate_recipes:
            recipe_id = recipe.get("recipe_id")
            if not recipe_id:
                continue
            
            # Cast recipe_id to str
            recipe_id_str = str(recipe_id)

            # Check for allergen food items
            allergen_matches = []
            if allergies:
                allergen_matches = await get_recipe_allergen_food_items(
                    pool, recipe_id_str, allergies
                )

            if allergen_matches:
                blocked_recipes.append({
                    "recipe_id": recipe_id_str,
                    "title": recipe.get("title", "Unknown"),
                    "reason": f"Contains allergen food items: {', '.join(item['food_item_name'] for item in allergen_matches)}"
                })
            else:
                # No allergen matches, but check medical conditions
                # E.g. diabetes warnings for high carbs, hypertension warnings for sodium
                # For safety node, we just warn about possible medical condition mismatches
                recipe_warnings = []
                for cond in medical_conditions:
                    cond_lower = cond.lower()
                    if "diabetes" in cond_lower:
                        # Add a general warning or check sugar/carbs later
                        recipe_warnings.append(f"Recipe contains carbs; user has diabetes.")
                    elif "hypertension" in cond_lower:
                        recipe_warnings.append(f"Recipe may contain sodium; user has hypertension.")

                if recipe_warnings:
                    warnings.append(f"Recipe {recipe.get('title')} warning: {'; '.join(recipe_warnings)}")
                
                safe_recipe_ids.append(recipe_id_str)

    output_data = {
        "safe_recipe_ids": safe_recipe_ids,
        "blocked_recipes": blocked_recipes,
        "warnings": warnings,
    }

    await emit_step(
        callback,
        agent_name="Safety",
        status="completed",
        step_type="validation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "safety_result": {
            "safe_recipe_ids": safe_recipe_ids,
            "blocked_recipes": blocked_recipes,
            "warnings": warnings,
        }
    }
