"""Recipe Agent Node for the ForkFit intelligence pipeline.

Selects the best set of recipes from safe and culturally aligned candidates.
"""
from __future__ import annotations

import structlog
from pydantic import BaseModel, Field
from src.agents.helpers import Timer, emit_step
from src.agents.state import GraphState, RecipeIngredient, RecipeResult, SelectedRecipe
from src.db.repositories.recipe_repo import get_recipe_with_ingredients, calculate_recipe_cost
from src.services.nutrition_math import calculate_recipe_nutrition
from src.services.llm import get_chat_model

logger = structlog.get_logger()


class RecipeSelectionDecision(BaseModel):
    """Structured selection decision from the Recipe LLM."""

    selected_recipe_ids: list[str] = Field(
        ...,
        description="List of recipe UUIDs selected for the final plan.",
    )
    alternatives: list[str] = Field(
        default_factory=list,
        description="Alternative recipe UUIDs or general suggestions.",
    )
    reasoning: str = Field(
        ...,
        description="Reasoning for selecting these specific recipes.",
    )


async def recipe_node(state: GraphState) -> dict[str, RecipeResult]:
    """Perform final recipe selection and variation formatting."""
    if "recipe" not in state.get("required_agents", []):
        logger.info("recipe_node_skipped")
        return {}

    logger.info("recipe_node_started", user_id=state.get("user_id"))
    timer = Timer()

    pool = state["_pool"]
    settings = state["_settings"]
    callback = state.get("_step_callback")

    context = state.get("context", {})
    candidate_recipes = context.get("candidate_recipes", [])

    # Filter candidate recipe IDs by safety and culture results
    safety_res = state.get("safety_result") or {}
    culture_res = state.get("culture_result") or {}

    safe_ids = safety_res.get("safe_recipe_ids")
    aligned_ids = culture_res.get("aligned_recipe_ids")

    candidate_ids = [str(r.get("recipe_id")) for r in candidate_recipes]
    filtered_ids = set(candidate_ids)

    if safe_ids is not None:
        filtered_ids = filtered_ids.intersection(set(str(x) for x in safe_ids))
    if aligned_ids is not None:
        filtered_ids = filtered_ids.intersection(set(str(x) for x in aligned_ids))

    filtered_list = list(filtered_ids)
    
    # If intersection is empty, fallback to candidate recipes
    if not filtered_list:
        logger.warning("recipe_node_intersection_empty_fallback")
        filtered_list = candidate_ids

    selected_recipes: list[SelectedRecipe] = []
    alternatives: list[str] = []

    input_data = {
        "filtered_ids_count": len(filtered_list),
        "prompt": state.get("prompt"),
    }

    with timer:
        # Fetch full recipe details for all filtered candidates
        full_candidates = []
        for rid in filtered_list:
            recipe_detail = await get_recipe_with_ingredients(pool, rid)
            if recipe_detail:
                # Add cost and nutrition to details
                cost = await calculate_recipe_cost(pool, rid)
                nutrition = calculate_recipe_nutrition(recipe_detail.get("ingredients", []))
                
                recipe_detail["cost"] = cost
                recipe_detail["nutrition"] = nutrition
                full_candidates.append(recipe_detail)

        if not full_candidates:
            logger.warning("recipe_node_no_recipe_details_found")
        else:
            # Use LLM to pick the best set of recipes
            llm = get_chat_model(settings)
            structured_llm = llm.with_structured_output(RecipeSelectionDecision)

            recipes_summary = "\n".join(
                f"- ID: {r['id']} | Title: {r['title']} | Cuisine: {r.get('cuisine')} | Cost: {r.get('cost')} | Nutrition: {r.get('nutrition')}"
                for r in full_candidates
            )

            prompt = (
                f"User Request: {state.get('prompt')}\n"
                f"User constraints: {context.get('constraints', [])}\n"
                f"Select the best subset of recipes from these candidates to compose a balanced meal plan:\n"
                f"{recipes_summary}\n"
                f"Ensure selection aligns with the user's nutritional goals and constraints."
            )

            try:
                decision = await structured_llm.ainvoke([
                    {"role": "system", "content": "You are an expert chef and recipe compiler."},
                    {"role": "user", "content": prompt}
                ])

                selected_ids = [str(x) for x in decision.selected_recipe_ids]
                alternatives = decision.alternatives

                # Map back to full recipe objects
                for r in full_candidates:
                    if str(r["id"]) in selected_ids:
                        selected_recipes.append({
                            "recipe_id": str(r["id"]),
                            "title": r["title"],
                            "cuisine": r.get("cuisine"),
                            "servings": float(r.get("servings") or 1.0),
                            "nutrition": r.get("nutrition"),
                            "cost": r.get("cost"),
                            "instructions": r.get("instructions", "").split("\n"),
                            "ingredients": [
                                {
                                    "name": ing.get("ingredient_name"),
                                    "quantity": float(ing.get("grams_equivalent") or 0.0),
                                    "unit": "g"
                                }
                                for ing in r.get("ingredients", [])
                            ]
                        })

            except Exception as exc:
                logger.error("recipe_selection_failed", error=str(exc))
                # Fallback: take first 3 recipes as selected
                for r in full_candidates[:3]:
                    selected_recipes.append({
                        "recipe_id": str(r["id"]),
                        "title": r["title"],
                        "cuisine": r.get("cuisine"),
                        "servings": float(r.get("servings") or 1.0),
                        "nutrition": r.get("nutrition"),
                        "cost": r.get("cost"),
                        "instructions": r.get("instructions", "").split("\n"),
                        "ingredients": [
                            {
                                "name": ing.get("ingredient_name"),
                                "quantity": float(ing.get("grams_equivalent") or 0.0),
                                "unit": "g"
                            }
                            for ing in r.get("ingredients", [])
                        ]
                    })

    output_data = {
        "selected_recipes_count": len(selected_recipes),
        "alternatives_count": len(alternatives),
    }

    await emit_step(
        callback,
        agent_name="Recipe",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "recipe_result": {
            "selected_recipes": selected_recipes,
            "alternatives": alternatives,
        }
    }
