"""Budget Agent Node for the ForkFit intelligence pipeline.

Computes recipe ingredient costs and optimizes for budget constraints.
"""
from __future__ import annotations

from typing import Any
import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import BudgetResult, GraphState, RecipeCost
from src.db.repositories.recipe_repo import calculate_recipe_cost
from src.services.llm import get_chat_model

logger = structlog.get_logger()


async def budget_node(state: GraphState) -> dict[str, BudgetResult]:
    """Evaluate candidate recipe costs and suggest cheaper substitutions if needed."""
    if "budget" not in state.get("required_agents", []):
        logger.info("budget_node_skipped")
        return {}

    logger.info("budget_node_started", user_id=state.get("user_id"))
    timer = Timer()

    pool = state["_pool"]
    settings = state["_settings"]
    callback = state.get("_step_callback")

    context = state.get("context", {})
    budget_limit = float(context.get("budget_limit", 0.0))
    candidate_recipes = context.get("candidate_recipes", [])

    recipe_costs: list[RecipeCost] = []
    substitutions: list[str] = []

    input_data = {
        "budget_limit": budget_limit,
        "candidate_recipes_count": len(candidate_recipes),
    }

    with timer:
        total_candidate_cost = 0.0
        for recipe in candidate_recipes:
            recipe_id = recipe.get("recipe_id")
            if not recipe_id:
                continue

            recipe_id_str = str(recipe_id)
            cost = await calculate_recipe_cost(pool, recipe_id_str)
            recipe_costs.append({
                "recipe_id": recipe_id_str,
                "title": recipe.get("title", "Unknown"),
                "cost": cost,
            })
            total_candidate_cost += cost

        # If average candidate cost per day is greater than budget, suggest substitutions
        avg_recipe_cost = (total_candidate_cost / len(candidate_recipes)) if candidate_recipes else 0.0
        within_budget = True
        if budget_limit > 0.0 and avg_recipe_cost > budget_limit:
            within_budget = False
            # Call LLM to suggest general cheap substitutions
            llm = get_chat_model(settings)
            prompt = (
                f"The user has a budget limit of {budget_limit} {context.get('budget_currency', 'INR')} per day. "
                f"The average recipe cost is {avg_recipe_cost:.2f}. "
                f"Suggest 3 simple, healthy, and cheap ingredient substitutions (e.g., replace quinoa with brown rice, or chicken breast with tofu/lentils) "
                f"to reduce the overall cost. Return a clean bulleted list."
            )
            try:
                response = await llm.ainvoke([
                    {"role": "system", "content": "You are a cost-optimization nutritional assistant."},
                    {"role": "user", "content": prompt}
                ])
                raw = response.content if hasattr(response, "content") else str(response)
                substitutions = [line.strip("- *") for line in raw.split("\n") if line.strip()]
            except Exception as exc:
                logger.error("budget_substitutions_failed", error=str(exc))
                substitutions = ["Fallback: Use local seasonal vegetables", "Fallback: Substitute imported grains with oats or rice"]

    output_data = {
        "recipe_costs": recipe_costs,
        "within_budget": within_budget,
        "substitutions": substitutions,
    }

    await emit_step(
        callback,
        agent_name="Budget",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "budget_result": {
            "within_budget": within_budget,
            "recipe_costs": recipe_costs,
            "substitutions": substitutions,
        }
    }
