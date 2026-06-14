from unittest.mock import AsyncMock, MagicMock
import pytest

from src.agents.shopping import shopping_node


@pytest.mark.asyncio
async def test_shopping_node_skipped():
    state = {
        "required_agents": ["safety"],
        "context": {},
    }
    result = await shopping_node(state)
    assert result == {}


@pytest.mark.asyncio
async def test_shopping_node_success():
    mock_callback = AsyncMock()

    state = {
        "required_agents": ["shopping"],
        "context": {
            "pantry_items": [
                {
                    "food_item_name": "rice",
                    "quantity": 200.0,
                    "unit": "g"
                },
                {
                    "food_item_name": "salt",
                    "quantity": 500.0,
                    "unit": "g"
                }
            ]
        },
        "recipe_result": {
            "selected_recipes": [
                {
                    "recipe_id": "recipe-1",
                    "title": "Fried Rice",
                    "food_items": [
                        {"name": "Rice", "quantity": 150.0, "unit": "g"},
                        {"name": "Egg", "quantity": 2.0, "unit": "pcs"}
                    ]
                },
                {
                    "recipe_id": "recipe-2",
                    "title": "Boiled Egg",
                    "food_items": [
                        {"name": "Egg", "quantity": 1.0, "unit": "pcs"}
                    ]
                }
            ]
        },
        "calendar_result": {
            "daily_plans": [
                {
                    "date": "2026-06-10",
                    "meals": [
                        # 1 serving of Fried Rice = 150g Rice, 2 Eggs
                        {"recipe_id": "recipe-1", "servings": 1.0},
                        # 2 servings of Boiled Egg = 2 Eggs
                        {"recipe_id": "recipe-2", "servings": 2.0}
                    ]
                }
            ]
        },
        "_step_callback": mock_callback,
    }

    result = await shopping_node(state)
    shopping_result = result["shopping_result"]

    # Food items needed:
    # - Rice: 150g * 1.0 = 150g. Pantry has 200g.
    #   After subtraction: remaining Rice needed = 0g.
    #   Pantry savings: 150g * 0.1 = 15.0
    # - Egg: 2.0 * 1.0 (from Fried Rice) + 1.0 * 2.0 (from Boiled Egg) = 4.0 pcs.
    #   Pantry has no Eggs.
    #   After subtraction: remaining Eggs needed = 4.0 pcs.
    #   Pantry savings: 0.0

    # Total Pantry savings: 15.0
    assert shopping_result["pantry_savings"] == 15.0

    # Shopping items should only include Eggs since Rice was fully covered by the pantry
    assert len(shopping_result["items"]) == 1
    egg_item = shopping_result["items"][0]
    assert egg_item["food_item_name"] == "Egg"
    assert egg_item["quantity"] == 4.0
    assert egg_item["unit"] == "pcs"
    # Cost = quantity (4.0) * 0.15 = 0.6
    assert egg_item["estimated_cost"] == 0.6

    assert shopping_result["total_cost"] == 0.6

    mock_callback.assert_called_once()
