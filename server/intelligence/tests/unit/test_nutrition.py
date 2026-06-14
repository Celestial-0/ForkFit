from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.agents.nutrition import nutrition_node
from src.config import Settings


@pytest.mark.asyncio
async def test_nutrition_node_skipped():
    # If nutrition is not in required_agents, it should exit early with {}
    state = {
        "required_agents": ["safety"],
        "context": {},
        "_pool": MagicMock(),
    }
    result = await nutrition_node(state)
    assert result == {}


@pytest.mark.asyncio
@patch("src.agents.nutrition.get_recipe_with_food_items")
async def test_nutrition_node_with_macro_targets(mock_get_recipe):
    # Mock database return for a recipe
    mock_get_recipe.return_value = {
        "id": "recipe-1",
        "title": "Protein Shake",
        "food_items": [
            {
                "grams_equivalent": 100.0,
                "calories_per_100g": 300.0,
                "protein_per_100g": 30.0,
                "carbs_per_100g": 20.0,
                "fat_per_100g": 5.0,
                "fiber_per_100g": 2.0,
                "sodium_mg_per_100g": 100.0,
            }
        ]
    }

    mock_callback = AsyncMock()

    state = {
        "required_agents": ["nutrition"],
        "context": {
            "candidate_recipes": [
                {"recipe_id": "recipe-1", "title": "Protein Shake"}
            ],
            "macro_targets": {
                "calories": 2000.0,
                "protein_g": 150.0,
                "carbs_g": 200.0,
                "fat_g": 70.0,
            },
            "active_goals": [{"target_type": "muscle_gain"}]
        },
        "_pool": MagicMock(),
        "_step_callback": mock_callback,
    }

    result = await nutrition_node(state)
    nutrition_result = result["nutrition_result"]

    # Assert daily targets match provided macro targets
    assert nutrition_result["daily_targets"]["calories"] == 2000.0
    assert nutrition_result["daily_targets"]["protein_g"] == 150.0

    # Assert recipe scores structure and value
    assert len(nutrition_result["recipe_nutrition_scores"]) == 1
    score_entry = nutrition_result["recipe_nutrition_scores"][0]
    assert score_entry["recipe_id"] == "recipe-1"
    assert "score" in score_entry
    assert score_entry["breakdown"]["calories"] == 300.0
    assert score_entry["breakdown"]["protein_g"] == 30.0

    # Assert callback was called
    mock_callback.assert_called_once()


@pytest.mark.asyncio
@patch("src.agents.nutrition.get_recipe_with_food_items")
async def test_nutrition_node_calculated_macros(mock_get_recipe):
    # Mock database return for a recipe
    mock_get_recipe.return_value = {
        "id": "recipe-1",
        "title": "Salad",
        "food_items": [
            {
                "grams_equivalent": 200.0,
                "calories_per_100g": 50.0,
                "protein_per_100g": 2.0,
                "carbs_per_100g": 5.0,
                "fat_per_100g": 3.0,
                "fiber_per_100g": 1.5,
                "sodium_mg_per_100g": 50.0,
            }
        ]
    }

    mock_callback = AsyncMock()

    # Context has biometrics but no macro_targets
    state = {
        "required_agents": ["nutrition"],
        "context": {
            "candidate_recipes": [
                {"recipe_id": "recipe-1", "title": "Salad"}
            ],
            "weight_kg": 75.0,
            "height_cm": 175.0,
            "age": 25,
            "gender": "male",
            "activity_level": "moderate",
            "active_goals": [{"target_type": "muscle_gain"}]
        },
        "_pool": MagicMock(),
        "_step_callback": mock_callback,
    }

    result = await nutrition_node(state)
    nutrition_result = result["nutrition_result"]

    # Assert daily targets are calculated
    # MSJ BMR = 10*75 + 6.25*175 - 5*25 + 5 = 750 + 1093.75 - 125 + 5 = 1723.75
    # TDEE (moderate: 1.55) = 1723.75 * 1.55 = 2671.8125
    # Muscle Gain adjustment (+300 calories) = 2971.8
    # Let's check calculations
    daily_targets = nutrition_result["daily_targets"]
    assert daily_targets["calories"] > 0
    assert daily_targets["protein_g"] > 0
    assert daily_targets["carbs_g"] > 0
    assert daily_targets["fat_g"] > 0

    # Assert scoring happened
    assert len(nutrition_result["recipe_nutrition_scores"]) == 1
    assert nutrition_result["recipe_nutrition_scores"][0]["recipe_id"] == "recipe-1"

    # Assert callback was called
    mock_callback.assert_called_once()
