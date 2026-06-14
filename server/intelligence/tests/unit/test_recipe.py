from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.config import Settings
from src.agents.recipe import recipe_node, RecipeSelectionDecision


@pytest.mark.asyncio
async def test_recipe_node_skipped():
    state = {
        "required_agents": ["safety"],
        "context": {},
    }
    result = await recipe_node(state)
    assert result == {}


@pytest.mark.asyncio
@patch("src.agents.recipe.get_chat_model")
@patch("src.agents.recipe.calculate_recipe_cost")
@patch("src.agents.recipe.get_recipe_with_food_items")
async def test_recipe_node_success(mock_get_recipe, mock_calc_cost, mock_get_chat_model):
    # Mock database returns
    mock_get_recipe.side_effect = lambda pool, rid: {
        "id": rid,
        "title": f"Recipe {rid}",
        "cuisine": "Indian",
        "servings": 2.0,
        "instructions": "Step 1\nStep 2",
        "food_items": [
            {
                "food_item_name": "Paneer",
                "grams_equivalent": 100.0,
                "calories_per_100g": 300.0,
                "protein_per_100g": 20.0,
                "carbs_per_100g": 5.0,
                "fat_per_100g": 20.0,
                "fiber_per_100g": 0.0,
                "sodium_mg_per_100g": 100.0,
            }
        ]
    }
    mock_calc_cost.return_value = 150.0

    # Mock LLM structure
    mock_llm = MagicMock()
    mock_structured_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    mock_llm.with_structured_output.return_value = mock_structured_llm

    mock_decision = RecipeSelectionDecision(
        selected_recipe_ids=["recipe-1"],
        alternatives=["recipe-2"],
        reasoning="Selecting recipe-1 because it meets protein goals"
    )
    mock_structured_llm.ainvoke = AsyncMock(return_value=mock_decision)

    mock_callback = AsyncMock()
    settings = Settings(database_url="postgresql://localhost/db")

    state = {
        "required_agents": ["recipe"],
        "prompt": "high protein vegetarian",
        "context": {
            "candidate_recipes": [
                {"recipe_id": "recipe-1", "title": "Recipe recipe-1"},
                {"recipe_id": "recipe-2", "title": "Recipe recipe-2"},
                {"recipe_id": "recipe-3", "title": "Recipe recipe-3"}
            ]
        },
        "safety_result": {
            "safe_recipe_ids": ["recipe-1", "recipe-2"]
        },
        "culture_result": {
            "aligned_recipe_ids": ["recipe-1", "recipe-2", "recipe-3"]
        },
        "_pool": MagicMock(),
        "_settings": settings,
        "_step_callback": mock_callback,
    }

    result = await recipe_node(state)
    recipe_result = result["recipe_result"]

    assert len(recipe_result["selected_recipes"]) == 1
    selected = recipe_result["selected_recipes"][0]
    assert selected["recipe_id"] == "recipe-1"
    assert selected["title"] == "Recipe recipe-1"
    assert selected["cuisine"] == "Indian"
    assert selected["servings"] == 2.0
    assert selected["cost"] == 150.0
    assert selected["nutrition"]["protein_g"] == 20.0  # from Paneer (100g)
    assert selected["instructions"] == ["Step 1", "Step 2"]
    assert len(selected["food_items"]) == 1
    assert selected["food_items"][0]["name"] == "Paneer"
    assert selected["food_items"][0]["quantity"] == 100.0

    assert recipe_result["alternatives"] == ["recipe-2"]

    mock_callback.assert_called_once()
    mock_llm.with_structured_output.assert_called_once_with(RecipeSelectionDecision, method="json_mode")


@pytest.mark.asyncio
@patch("src.agents.recipe.get_chat_model")
@patch("src.agents.recipe.calculate_recipe_cost")
@patch("src.agents.recipe.get_recipe_with_food_items")
async def test_recipe_node_llm_failure_fallback(mock_get_recipe, mock_calc_cost, mock_get_chat_model):
    mock_get_recipe.side_effect = lambda pool, rid: {
        "id": rid,
        "title": f"Recipe {rid}",
        "cuisine": "Indian",
        "servings": 1.0,
        "instructions": "Instructions",
        "food_items": []
    }
    mock_calc_cost.return_value = 50.0

    # LLM throws an exception
    mock_llm = MagicMock()
    mock_structured_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    mock_llm.with_structured_output.return_value = mock_structured_llm
    mock_structured_llm.ainvoke.side_effect = Exception("Ollama disconnected")

    state = {
        "required_agents": ["recipe"],
        "prompt": "anything",
        "context": {
            "candidate_recipes": [
                {"recipe_id": "recipe-1", "title": "Recipe recipe-1"},
                {"recipe_id": "recipe-2", "title": "Recipe recipe-2"},
                {"recipe_id": "recipe-3", "title": "Recipe recipe-3"}
            ]
        },
        "safety_result": None,
        "culture_result": None,
        "_pool": MagicMock(),
        "_settings": Settings(database_url="postgresql://localhost/db"),
    }

    result = await recipe_node(state)
    recipe_result = result["recipe_result"]

    # Should fallback to selecting first 3 candidate recipes
    assert len(recipe_result["selected_recipes"]) == 3
    selected_ids = {r["recipe_id"] for r in recipe_result["selected_recipes"]}
    assert selected_ids == {"recipe-1", "recipe-2", "recipe-3"}
    assert recipe_result["alternatives"] == []
