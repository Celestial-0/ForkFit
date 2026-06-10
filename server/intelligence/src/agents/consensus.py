"""Consensus Node for the ForkFit intelligence pipeline.

Merges specialist agent outputs and resolves execution conflicts.
"""
from __future__ import annotations

import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import GraphState, MergedPlan

logger = structlog.get_logger()


async def consensus_node(state: GraphState) -> dict[str, MergedPlan]:
    """Assemble a unified meal plan from individual agent recommendations and resolve conflicts."""
    logger.info("consensus_node_started", user_id=state.get("user_id"))
    timer = Timer()

    callback = state.get("_step_callback")

    safety_res = state.get("safety_result") or {}
    nutrition_res = state.get("nutrition_result") or {}
    budget_res = state.get("budget_result") or {}
    recipe_res = state.get("recipe_result") or {}
    calendar_res = state.get("calendar_result") or {}
    shopping_res = state.get("shopping_result") or {}

    input_data = {
        "has_safety": bool(safety_res),
        "has_nutrition": bool(nutrition_res),
        "has_budget": bool(budget_res),
        "has_recipe": bool(recipe_res),
        "has_calendar": bool(calendar_res),
        "has_shopping": bool(shopping_res),
    }

    with timer:
        daily_plans = calendar_res.get("daily_plans", [])
        shopping_items = shopping_res.get("items", [])
        total_cost = shopping_res.get("total_cost", 0.0)

        blocked_ids = {str(x["recipe_id"]) for x in safety_res.get("blocked_recipes", [])}

        # Conflict resolution: Remove safety-blocked recipes from calendar slots
        resolved_daily_plans = []
        conflicts_resolved = 0

        for plan in daily_plans:
            resolved_meals = []
            for meal in plan.get("meals", []):
                rid = str(meal.get("recipe_id"))
                if rid in blocked_ids:
                    logger.warning("consensus_conflict_safety_blocked_recipe_removed", recipe_id=rid)
                    conflicts_resolved += 1
                    # In a real system we would replace it, here we mark it as needing a safe substitution
                    meal["recipe_title"] = f"SAFE SUBSTITUTE NEEDED (Blocked: {meal['recipe_title']})"
                    meal["recipe_id"] = None
                resolved_meals.append(meal)
            
            resolved_daily_plans.append({
                "date": plan.get("date"),
                "meals": resolved_meals,
                "notes": plan.get("notes", ""),
            })

        # Summarize total plan nutrition
        total_calories = 0.0
        total_protein = 0.0
        total_carbs = 0.0
        total_fat = 0.0

        for plan in resolved_daily_plans:
            for meal in plan.get("meals", []):
                nut = meal.get("nutrition", {})
                total_calories += float(nut.get("calories") or 0.0)
                total_protein += float(nut.get("protein_g") or 0.0)
                total_carbs += float(nut.get("carbs_g") or 0.0)
                total_fat += float(nut.get("fat_g") or 0.0)

        days_count = len(resolved_daily_plans) or 1
        nutrition_summary = {
            "calories": round(total_calories / days_count, 1),
            "protein_g": round(total_protein / days_count, 1),
            "carbs_g": round(total_carbs / days_count, 1),
            "fat_g": round(total_fat / days_count, 1),
        }

        # Calculate consensus confidence score (0.0 to 1.0)
        confidence = 1.0
        if conflicts_resolved > 0:
            confidence -= 0.2 * conflicts_resolved
        if not budget_res.get("within_budget", True):
            confidence -= 0.15
        confidence = max(0.2, confidence)

        merged_plan: MergedPlan = {
            "daily_plans": resolved_daily_plans,
            "shopping_list": shopping_items,
            "total_cost": total_cost,
            "nutrition_summary": nutrition_summary,
            "confidence": confidence,
        }

    output_data = {
        "conflicts_resolved": conflicts_resolved,
        "confidence": confidence,
    }

    await emit_step(
        callback,
        agent_name="Consensus",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "merged_plan": merged_plan,
    }
