from unittest.mock import AsyncMock
import pytest

from src.agents.consensus import consensus_node


@pytest.mark.asyncio
async def test_consensus_node_success():
    mock_callback = AsyncMock()

    state = {
        "user_id": "test-user",
        "safety_result": {
            "safe_recipe_ids": ["recipe-1"],
            "blocked_recipes": []
        },
        "nutrition_result": {
            "daily_targets": {"calories": 2000.0, "protein_g": 150.0}
        },
        "budget_result": {
            "within_budget": True
        },
        "recipe_result": {
            "selected_recipes": [
                {"recipe_id": "recipe-1", "title": "Egg Toast"}
            ]
        },
        "calendar_result": {
            "daily_plans": [
                {
                    "date": "2026-06-10",
                    "meals": [
                        {
                            "meal_type": "breakfast",
                            "recipe_id": "recipe-1",
                            "recipe_title": "Egg Toast",
                            "servings": 1.0,
                            "nutrition": {"calories": 400.0, "protein_g": 20.0, "carbs_g": 30.0, "fat_g": 10.0},
                            "cost": 50.0
                        }
                    ],
                    "notes": "Good day"
                }
            ]
        },
        "shopping_result": {
            "items": [{"ingredient_name": "Egg", "quantity": 2.0, "unit": "pcs", "category": "produce", "estimated_cost": 20.0}],
            "total_cost": 50.0,
            "pantry_savings": 0.0
        },
        "_step_callback": mock_callback,
    }

    result = await consensus_node(state)
    merged_plan = result["merged_plan"]

    assert len(merged_plan["daily_plans"]) == 1
    assert merged_plan["total_cost"] == 50.0
    assert merged_plan["confidence"] == 1.0  # No conflicts, within budget
    assert merged_plan["nutrition_summary"]["calories"] == 400.0
    assert merged_plan["nutrition_summary"]["protein_g"] == 20.0

    mock_callback.assert_called_once()


@pytest.mark.asyncio
async def test_consensus_node_conflict_resolution():
    state = {
        "user_id": "test-user",
        "safety_result": {
            "safe_recipe_ids": [],
            # recipe-1 is blocked due to peanut allergy
            "blocked_recipes": [{"recipe_id": "recipe-1", "title": "Peanut Toast", "reason": "Contains peanut"}]
        },
        "nutrition_result": {},
        "budget_result": {
            "within_budget": False  # Over budget
        },
        "calendar_result": {
            "daily_plans": [
                {
                    "date": "2026-06-10",
                    "meals": [
                        {
                            "meal_type": "breakfast",
                            "recipe_id": "recipe-1",
                            "recipe_title": "Peanut Toast",
                            "servings": 1.0,
                            "nutrition": {"calories": 300.0, "protein_g": 10.0},
                            "cost": 40.0
                        }
                    ]
                }
            ]
        },
        "shopping_result": {
            "items": [],
            "total_cost": 40.0,
            "pantry_savings": 0.0
        }
    }

    result = await consensus_node(state)
    merged_plan = result["merged_plan"]

    # Recipe-1 should be replaced with a SAFE SUBSTITUTE NEEDED placeholdered entry
    plan = merged_plan["daily_plans"][0]
    meal = plan["meals"][0]
    assert meal["recipe_id"] is None
    assert "SAFE SUBSTITUTE NEEDED" in meal["recipe_title"]

    # Confidence calculation: 1.0 - 0.2 * 1 (conflict) - 0.15 (over budget) = 0.65
    assert merged_plan["confidence"] == 0.65
