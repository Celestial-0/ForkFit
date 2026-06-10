from unittest.mock import AsyncMock, MagicMock
import pytest

from src.config import Settings
from src.agents.judge import judge_node, judge_router


@pytest.mark.asyncio
async def test_judge_node_passed():
    mock_callback = AsyncMock()
    settings = Settings(database_url="postgresql://localhost/db", max_replan_attempts=2)

    state = {
        "replan_count": 0,
        "context": {
            "budget_limit": 200.0
        },
        "merged_plan": {
            "daily_plans": [
                {
                    "date": "2026-06-10",
                    "meals": [
                        {"recipe_id": "recipe-1", "recipe_title": "Oatmeal", "nutrition": {}, "cost": 50.0},
                        {"recipe_id": "recipe-2", "recipe_title": "Salad", "nutrition": {}, "cost": 100.0}
                    ]
                }
            ],
            "total_cost": 150.0,
            "confidence": 1.0
        },
        "_settings": settings,
        "_step_callback": mock_callback,
    }

    result = await judge_node(state)
    verdict = result["judge_verdict"]

    assert verdict["passed"] is True
    assert len(verdict["failures"]) == 0
    assert result["replan_count"] == 0

    mock_callback.assert_called_once()


@pytest.mark.asyncio
async def test_judge_node_failed_budget_and_safety():
    settings = Settings(database_url="postgresql://localhost/db", max_replan_attempts=2)

    state = {
        "replan_count": 0,
        "context": {
            "budget_limit": 100.0
        },
        "merged_plan": {
            "daily_plans": [
                {
                    "date": "2026-06-10",
                    "meals": [
                        {"recipe_id": None, "recipe_title": "SAFE SUBSTITUTE NEEDED (Blocked)", "nutrition": {}, "cost": 150.0}
                    ]
                }
            ],
            "total_cost": 150.0,
            "confidence": 0.5
        },
        "_settings": settings,
    }

    result = await judge_node(state)
    verdict = result["judge_verdict"]

    assert verdict["passed"] is False
    assert len(verdict["failures"]) == 2  # Budget exceeded + safety substitution needed
    assert any("exceeds budget" in f for f in verdict["failures"])
    assert any("unsafe meals" in f for f in verdict["failures"])
    # Replan count should increment on failure
    assert result["replan_count"] == 1


@pytest.mark.asyncio
async def test_judge_node_failed_variety():
    settings = Settings(database_url="postgresql://localhost/db", max_replan_attempts=2)

    state = {
        "replan_count": 0,
        "context": {},
        "merged_plan": {
            "daily_plans": [
                {
                    "meals": [
                        {"recipe_id": "recipe-1", "recipe_title": "Egg"},
                        {"recipe_id": "recipe-1", "recipe_title": "Egg"},
                        {"recipe_id": "recipe-1", "recipe_title": "Egg"},
                        {"recipe_id": "recipe-1", "recipe_title": "Egg"}
                    ]
                }
            ],
            "total_cost": 100.0
        },
        "_settings": settings,
    }

    result = await judge_node(state)
    verdict = result["judge_verdict"]

    assert verdict["passed"] is False
    assert any("repeated too many times" in f for f in verdict["failures"])
    assert result["replan_count"] == 1


def test_judge_router():
    settings = Settings(database_url="postgresql://localhost/db", max_replan_attempts=2)

    # Case 1: Passed validation -> goes to visualization
    state_pass = {
        "judge_verdict": {"passed": True},
        "replan_count": 0,
        "_settings": settings,
    }
    assert judge_router(state_pass) == "visualization"

    # Case 2: Failed validation but replan count < max_replan -> goes to planner
    state_replan = {
        "judge_verdict": {"passed": False},
        "replan_count": 1,
        "_settings": settings,
    }
    assert judge_router(state_replan) == "planner"

    # Case 3: Failed validation but reached max_replan (2) -> goes to visualization (fallback/best-effort)
    state_max_replan = {
        "judge_verdict": {"passed": False},
        "replan_count": 2,
        "_settings": settings,
    }
    assert judge_router(state_max_replan) == "visualization"
