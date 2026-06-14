from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.config import Settings
from src.agents.budget import budget_node


@pytest.mark.asyncio
async def test_budget_node_skipped():
    # If budget is not in required_agents, it should exit early with {}
    state = {
        "required_agents": ["safety"],
        "context": {},
        "_pool": MagicMock(),
    }
    result = await budget_node(state)
    assert result == {}


@pytest.mark.asyncio
@patch("src.agents.budget.get_recipe_with_food_items")
@patch("src.agents.budget.calculate_recipe_cost")
async def test_budget_node_within_budget(mock_calculate_cost, mock_get_recipe):
    # Mock cost of recipes to be under the budget limit
    mock_calculate_cost.side_effect = lambda pool, recipe_id: 150.0 if recipe_id == "recipe-1" else 100.0
    mock_get_recipe.return_value = {
        "food_items": [
            {"food_item_name": "Paneer", "estimated_cost_per_100g": 10.0}
        ]
    }

    mock_callback = AsyncMock()
    settings = Settings(database_url="postgresql://localhost/db")

    state = {
        "required_agents": ["budget"],
        "context": {
            "budget_limit": 200.0,
            "budget_currency": "INR",
            "candidate_recipes": [
                {"recipe_id": "recipe-1", "title": "Paneer Tikka"},
                {"recipe_id": "recipe-2", "title": "Dal Tadka"}
            ]
        },
        "_pool": MagicMock(),
        "_settings": settings,
        "_step_callback": mock_callback,
    }

    result = await budget_node(state)
    budget_result = result["budget_result"]

    assert budget_result["within_budget"] is True
    assert len(budget_result["recipe_costs"]) == 2
    assert budget_result["recipe_costs"][0]["cost"] == 150.0
    assert budget_result["recipe_costs"][1]["cost"] == 100.0
    assert len(budget_result["substitutions"]) == 0

    mock_callback.assert_called_once()


@pytest.mark.asyncio
@patch("src.agents.budget.get_recipe_with_food_items")
@patch("src.agents.budget.get_chat_model")
@patch("src.agents.budget.calculate_recipe_cost")
async def test_budget_node_over_budget(mock_calculate_cost, mock_get_chat_model, mock_get_recipe):
    # Mock costs: average cost (250) is greater than budget limit (200)
    mock_calculate_cost.side_effect = lambda pool, recipe_id: 300.0 if recipe_id == "recipe-1" else 200.0
    mock_get_recipe.return_value = {
        "food_items": [
            {"food_item_name": "Avocado", "estimated_cost_per_100g": 50.0}
        ]
    }

    # Mock LLM and its response
    mock_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    
    mock_response = MagicMock()
    mock_response.content = "- Replace organic avocados with standard ones\n- Use lentils instead of lamb\n- Buy generic brand oats"
    mock_llm.ainvoke = AsyncMock(return_value=mock_response)

    mock_callback = AsyncMock()
    settings = Settings(database_url="postgresql://localhost/db")

    state = {
        "required_agents": ["budget"],
        "context": {
            "budget_limit": 200.0,
            "budget_currency": "INR",
            "candidate_recipes": [
                {"recipe_id": "recipe-1", "title": "Fancy salad"},
                {"recipe_id": "recipe-2", "title": "Standard soup"}
            ]
        },
        "_pool": MagicMock(),
        "_settings": settings,
        "_step_callback": mock_callback,
    }

    result = await budget_node(state)
    budget_result = result["budget_result"]

    assert budget_result["within_budget"] is False
    assert len(budget_result["recipe_costs"]) == 2
    assert len(budget_result["substitutions"]) == 3
    assert "Replace organic avocados with standard ones" in budget_result["substitutions"]
    assert "Use lentils instead of lamb" in budget_result["substitutions"]
    assert "Buy generic brand oats" in budget_result["substitutions"]

    mock_callback.assert_called_once()
    mock_llm.ainvoke.assert_called_once()
