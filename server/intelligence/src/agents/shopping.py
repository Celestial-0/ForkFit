"""Shopping Agent Node for the ForkFit intelligence pipeline.

Aggregates required ingredients across the scheduled plan, subtracts pantry stock, and estimates costs.
"""
from __future__ import annotations

import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import GraphState, ShoppingItem, ShoppingResult

logger = structlog.get_logger()


async def shopping_node(state: GraphState) -> dict[str, ShoppingResult]:
    """Compile a shopping list based on selected recipes and scheduled calendar slots."""
    if "shopping" not in state.get("required_agents", []):
        logger.info("shopping_node_skipped")
        return {}

    logger.info("shopping_node_started", user_id=state.get("user_id"))
    timer = Timer()

    callback = state.get("_step_callback")
    context = state.get("context", {})

    recipe_result = state.get("recipe_result") or {}
    selected_recipes = recipe_result.get("selected_recipes", [])
    
    calendar_result = state.get("calendar_result") or {}
    daily_plans = calendar_result.get("daily_plans", [])

    pantry_items = context.get("pantry_items", [])

    input_data = {
        "selected_recipes_count": len(selected_recipes),
        "daily_plans_count": len(daily_plans),
        "pantry_items_count": len(pantry_items),
    }

    shopping_items: list[ShoppingItem] = []
    total_cost = 0.0
    pantry_savings = 0.0

    with timer:
        # 1. Aggregate ingredient requirements
        needed_ingredients: dict[str, dict[str, Any]] = {}
        
        # Build map of selected recipes for quick lookup
        recipes_map = {r["recipe_id"]: r for r in selected_recipes}

        for plan in daily_plans:
            for meal in plan.get("meals", []):
                rid = meal.get("recipe_id")
                servings = meal.get("servings", 1.0)
                
                recipe = recipes_map.get(rid)
                if not recipe:
                    continue
                
                for ing in recipe.get("ingredients", []):
                    name = ing["name"]
                    qty = ing["quantity"] * servings
                    unit = ing["unit"]
                    
                    if name not in needed_ingredients:
                        needed_ingredients[name] = {
                            "ingredient_name": name,
                            "quantity": 0.0,
                            "unit": unit,
                            "category": "Grains & Produce", # Default category
                            "estimated_cost": 0.0,
                        }
                    
                    needed_ingredients[name]["quantity"] += qty

        # 2. Subtract pantry items (case-insensitive name match)
        pantry_map = {str(item.get("ingredient_name", "")).lower().strip(): item for item in pantry_items}

        for name, req in needed_ingredients.items():
            name_lower = name.lower().strip()
            quantity = req["quantity"]
            
            pantry_match = pantry_map.get(name_lower)
            if pantry_match:
                pantry_qty = float(pantry_match.get("quantity") or 0.0)
                # Subtract pantry stock
                if pantry_qty >= quantity:
                    pantry_savings += quantity * 0.1 # Estimate $0.1 per unit saved
                    quantity = 0.0
                else:
                    pantry_savings += pantry_qty * 0.1
                    quantity -= pantry_qty
            
            # Simple mock cost estimation (e.g. $0.15 per gram or unit)
            est_cost = quantity * 0.15
            
            if quantity > 0.0:
                shopping_items.append({
                    "ingredient_name": name,
                    "quantity": round(quantity, 2),
                    "unit": req["unit"],
                    "category": req["category"],
                    "estimated_cost": round(est_cost, 2),
                })
                total_cost += est_cost

    output_data = {
        "shopping_items_count": len(shopping_items),
        "total_cost": round(total_cost, 2),
        "pantry_savings": round(pantry_savings, 2),
    }

    await emit_step(
        callback,
        agent_name="Shopping",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "shopping_result": {
            "items": shopping_items,
            "total_cost": round(total_cost, 2),
            "pantry_savings": round(pantry_savings, 2),
        }
    }
