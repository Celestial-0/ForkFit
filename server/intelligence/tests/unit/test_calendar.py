from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.config import Settings
from src.agents.calendar import calendar_node, CalendarDecision, DailySchedule, MealAssignment


@pytest.mark.asyncio
async def test_calendar_node_skipped():
    state = {
        "required_agents": ["safety"],
        "context": {},
    }
    result = await calendar_node(state)
    assert result == {}


@pytest.mark.asyncio
@patch("src.agents.calendar.get_chat_model")
async def test_calendar_node_success(mock_get_chat_model):
    # Mock LLM and response
    mock_llm = MagicMock()
    mock_structured_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    mock_llm.with_structured_output.return_value = mock_structured_llm

    mock_decision = CalendarDecision(
        schedules=[
            DailySchedule(
                day_index=1,
                meals=[
                    MealAssignment(meal_type="breakfast", recipe_id="recipe-1", servings=1.0),
                    MealAssignment(meal_type="lunch", recipe_id="recipe-2", servings=1.5),
                ],
                notes="Heavy lunch due to workout"
            )
        ],
        general_notes="Plan looks solid"
    )
    mock_structured_llm.ainvoke = AsyncMock(return_value=mock_decision)

    mock_callback = AsyncMock()
    settings = Settings(database_url="postgresql://localhost/db")

    state = {
        "required_agents": ["calendar"],
        "context": {
            "timeline": "daily",
            "recent_workouts": [{"activity": "running"}],
        },
        "recipe_result": {
            "selected_recipes": [
                {
                    "recipe_id": "recipe-1",
                    "title": "Oatmeal",
                    "nutrition": {"calories": 300.0, "protein_g": 10.0},
                    "cost": 50.0
                },
                {
                    "recipe_id": "recipe-2",
                    "title": "Chicken Salad",
                    "nutrition": {"calories": 600.0, "protein_g": 40.0},
                    "cost": 150.0
                }
            ]
        },
        "_settings": settings,
        "_step_callback": mock_callback,
    }

    result = await calendar_node(state)
    calendar_result = result["calendar_result"]

    assert len(calendar_result["daily_plans"]) == 1
    plan = calendar_result["daily_plans"][0]
    assert plan["notes"] == "Heavy lunch due to workout"
    assert len(plan["meals"]) == 2

    m1 = plan["meals"][0]
    assert m1["meal_type"] == "breakfast"
    assert m1["recipe_id"] == "recipe-1"
    assert m1["recipe_title"] == "Oatmeal"
    assert m1["cost"] == 50.0

    m2 = plan["meals"][1]
    assert m2["meal_type"] == "lunch"
    assert m2["recipe_id"] == "recipe-2"
    assert m2["recipe_title"] == "Chicken Salad"
    # Cost is cost_per_serving (150) * servings (1.5) = 225.0
    assert m2["cost"] == 225.0

    mock_callback.assert_called_once()
    mock_llm.with_structured_output.assert_called_once_with(CalendarDecision, method="json_mode")


@pytest.mark.asyncio
@patch("src.agents.calendar.get_chat_model")
async def test_calendar_node_fallback(mock_get_chat_model):
    # Mock LLM to throw an exception
    mock_llm = MagicMock()
    mock_structured_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    mock_llm.with_structured_output.return_value = mock_structured_llm
    mock_structured_llm.ainvoke.side_effect = Exception("Ollama error")

    state = {
        "required_agents": ["calendar"],
        "context": {
            "timeline": "daily",
        },
        "recipe_result": {
            "selected_recipes": [
                {
                    "recipe_id": "recipe-1",
                    "title": "Oatmeal",
                    "nutrition": {"calories": 300.0},
                    "cost": 50.0
                }
            ]
        },
        "_settings": Settings(database_url="postgresql://localhost/db"),
    }

    result = await calendar_node(state)
    calendar_result = result["calendar_result"]

    # Fallback should schedule 3 meals (breakfast, lunch, dinner)
    assert len(calendar_result["daily_plans"]) == 1
    plan = calendar_result["daily_plans"][0]
    assert "Fallback schedule generated" in plan["notes"]
    assert len(plan["meals"]) == 3
    assert plan["meals"][0]["meal_type"] == "breakfast"
    assert plan["meals"][0]["recipe_id"] == "recipe-1"
