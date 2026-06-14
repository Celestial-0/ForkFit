from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.agents.safety import safety_node
from src.config import Settings


@pytest.mark.asyncio
@patch("src.agents.safety.get_recipe_allergen_food_items")
async def test_safety_node_allergy_blocked(mock_get_allergen_food_items):
    # Mock return values
    # Recipe 1 contains peanuts, Recipe 2 is safe
    mock_get_allergen_food_items.side_effect = lambda pool, recipe_id, allergens: (
        [{"food_item_name": "peanut", "grams_equivalent": 10.0}]
        if recipe_id == "recipe-1"
        else []
    )
    
    mock_callback = AsyncMock()
    
    # State setup
    state = {
        "required_agents": ["safety"],
        "context": {
            "allergies": ["peanut"],
            "medical_conditions": [],
            "candidate_recipes": [
                {"recipe_id": "recipe-1", "title": "Peanut Cookie"},
                {"recipe_id": "recipe-2", "title": "Oatmeal Cookie"}
            ]
        },
        "_pool": MagicMock(),
        "_step_callback": mock_callback,
    }
    
    result = await safety_node(state)
    
    # Assert result structure and values
    safety_result = result["safety_result"]
    assert "recipe-2" in safety_result["safe_recipe_ids"]
    assert "recipe-1" not in safety_result["safe_recipe_ids"]
    
    assert len(safety_result["blocked_recipes"]) == 1
    assert safety_result["blocked_recipes"][0]["recipe_id"] == "recipe-1"
    assert "Peanut Cookie" in safety_result["blocked_recipes"][0]["title"]
    assert "peanut" in safety_result["blocked_recipes"][0]["reason"]
    
    # Callback asserted
    mock_callback.assert_called_once()
