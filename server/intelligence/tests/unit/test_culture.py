from unittest.mock import AsyncMock, MagicMock
import pytest

from src.agents.culture import culture_node


@pytest.mark.asyncio
async def test_culture_node_skipped():
    state = {
        "required_agents": ["safety"],
        "context": {},
    }
    result = await culture_node(state)
    assert result == {}


@pytest.mark.asyncio
async def test_culture_node_cuisine_and_diet_filtering():
    mock_callback = AsyncMock()
    state = {
        "required_agents": ["culture"],
        "context": {
            "preferred_cuisine": "indian",
            "preferences": {"diet": "vegetarian"},
            "candidate_recipes": [
                {
                    "recipe_id": "recipe-1",
                    "title": "Paneer Makhani",
                    "cuisine": "Indian North",
                    "dietary_tags": ["vegetarian", "gluten-free"]
                },
                {
                    "recipe_id": "recipe-2",
                    "title": "Pasta Carbonara",
                    "cuisine": "Italian",
                    "dietary_tags": ["non-vegetarian"]
                },
                {
                    "recipe_id": "recipe-3",
                    "title": "Chicken Tikka",
                    "cuisine": "Indian North",
                    "dietary_tags": ["non-vegetarian", "high-protein"]
                }
            ]
        },
        "_step_callback": mock_callback,
    }

    result = await culture_node(state)
    culture_result = result["culture_result"]

    # Only recipe-1 matches both Indian cuisine and Vegetarian diet
    assert culture_result["aligned_recipe_ids"] == ["recipe-1"]
    assert len(culture_result["rejected"]) == 2
    
    rejected_ids = [r["recipe_id"] for r in culture_result["rejected"]]
    assert "recipe-2" in rejected_ids
    assert "recipe-3" in rejected_ids

    mock_callback.assert_called_once()


@pytest.mark.asyncio
async def test_culture_node_fallback_when_empty():
    state = {
        "required_agents": ["culture"],
        "context": {
            "preferred_cuisine": "japanese",
            "candidate_recipes": [
                {
                    "recipe_id": "recipe-1",
                    "title": "Paneer Makhani",
                    "cuisine": "Indian North",
                    "dietary_tags": ["vegetarian"]
                }
            ]
        }
    }

    result = await culture_node(state)
    culture_result = result["culture_result"]

    # Since Japanese is requested but only Indian is available, strict filtering leaves 0 recipes.
    # Fallback should kick in and return the candidate.
    assert culture_result["aligned_recipe_ids"] == ["recipe-1"]
    assert len(culture_result["rejected"]) == 1
    assert culture_result["rejected"][0]["recipe_id"] == "recipe-1"
