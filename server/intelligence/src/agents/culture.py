"""Culture Agent Node for the ForkFit intelligence pipeline.

Filters candidate recipes by cuisine, dietary type, and religious/cultural preferences.
"""
from __future__ import annotations

import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import CultureResult, GraphState, RejectedRecipe

logger = structlog.get_logger()


async def culture_node(state: GraphState) -> dict[str, CultureResult]:
    """Filter candidate recipes based on cultural/dietary preferences."""
    if "culture" not in state.get("required_agents", []):
        logger.info("culture_node_skipped")
        return {}

    logger.info("culture_node_started", user_id=state.get("user_id"))
    timer = Timer()

    callback = state.get("_step_callback")
    context = state.get("context", {})
    
    # Extract preferences
    preferred_cuisine = str(context.get("preferred_cuisine", "")).strip().lower()
    
    # Fallback to checking active goals / preferences for diet type
    user_diet = ""
    # We can infer diet type from user preferences or active goals if not set explicitly
    preferences = context.get("preferences", {})
    if isinstance(preferences, dict):
        user_diet = str(preferences.get("diet", "")).strip().lower()
    
    # Or check user active goals / config
    if not user_diet:
        active_goals = context.get("active_goals", [])
        for g in active_goals:
            diet_pref = g.get("diet") or g.get("diet_preference")
            if diet_pref:
                user_diet = str(diet_pref).strip().lower()
                break

    candidate_recipes = context.get("candidate_recipes", [])
    aligned_recipe_ids: list[str] = []
    rejected: list[RejectedRecipe] = []

    input_data = {
        "preferred_cuisine": preferred_cuisine,
        "user_diet": user_diet,
        "candidate_recipes_count": len(candidate_recipes),
    }

    with timer:
        for recipe in candidate_recipes:
            recipe_id = str(recipe.get("recipe_id"))
            cuisine = str(recipe.get("cuisine", "")).lower()
            dietary_tags = [str(t).lower() for t in recipe.get("dietary_tags", [])]

            reasons = []

            # 1. Check cuisine alignment (if preferred cuisine is specified)
            if preferred_cuisine and preferred_cuisine not in cuisine:
                reasons.append(f"Cuisine '{recipe.get('cuisine')}' does not match preferred '{preferred_cuisine}'")

            # 2. Check diet type alignment (if diet is specified)
            if user_diet:
                # E.g. if user is vegetarian, recipe must have 'vegetarian' or 'vegan' tag
                if user_diet == "vegetarian" and not ("vegetarian" in dietary_tags or "vegan" in dietary_tags):
                    reasons.append("Recipe is not vegetarian/vegan")
                elif user_diet == "vegan" and "vegan" not in dietary_tags:
                    reasons.append("Recipe is not vegan")
                elif user_diet == "jain" and "jain" not in dietary_tags:
                    reasons.append("Recipe is not Jain-compliant")
                # Add other diets if applicable

            if reasons:
                rejected.append({
                    "recipe_id": recipe_id,
                    "title": recipe.get("title", "Unknown"),
                    "reason": "; ".join(reasons)
                })
            else:
                aligned_recipe_ids.append(recipe_id)

        # Fallback: if strict filtering leaves no recipes, keep all candidates but emit warnings
        if not aligned_recipe_ids and candidate_recipes:
            logger.warning("culture_filter_too_strict_fallback_to_all")
            aligned_recipe_ids = [str(r.get("recipe_id")) for r in candidate_recipes]

    output_data = {
        "aligned_recipe_ids_count": len(aligned_recipe_ids),
        "rejected_count": len(rejected),
    }

    await emit_step(
        callback,
        agent_name="Culture",
        status="completed",
        step_type="validation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "culture_result": {
            "aligned_recipe_ids": aligned_recipe_ids,
            "rejected": rejected,
        }
    }
