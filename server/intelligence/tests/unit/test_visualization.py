from unittest.mock import AsyncMock
import json
import pytest

from src.agents.visualization import visualization_node


@pytest.mark.asyncio
async def test_visualization_node_success():
    mock_callback = AsyncMock()

    state = {
        "user_id": "test-user",
        "merged_plan": {
            "daily_plans": [
                {
                    "date": "2026-06-10",
                    "meals": [
                        {"recipe_title": "Egg Toast", "meal_type": "breakfast", "servings": 1.0}
                    ],
                    "notes": "Good morning"
                }
            ],
            "shopping_list": [
                {"ingredient_name": "Egg", "quantity": 2.0, "unit": "pcs", "category": "produce", "estimated_cost": 20.0}
            ],
            "nutrition_summary": {
                "calories": 400.0,
                "protein_g": 20.0,
                "carbs_g": 30.0,
                "fat_g": 10.0,
            },
            "total_cost": 50.0,
            "pantry_savings": 10.0,
        },
        "_step_callback": mock_callback,
    }

    result = await visualization_node(state)

    # 1. Assert ui_elements structure and count
    ui_elements = result["ui_elements"]
    assert len(ui_elements) == 3

    # Check chart type and values
    macro_chart = next(e for e in ui_elements if e["title"] == "Daily Macronutrient Split (Average)")
    assert macro_chart["type"] == "chart"
    config = json.loads(macro_chart["config_json"])
    assert config["chart_type"] == "doughnut"
    assert config["index_key"] == "name"
    assert config["value_key"] == "value"
    assert config["config"]["protein"]["label"] == "Protein (g)"
    data = json.loads(macro_chart["data_json"])
    assert data == [
        {"name": "protein", "value": 20.0, "fill": "var(--color-protein)"},
        {"name": "carbs", "value": 30.0, "fill": "var(--color-carbs)"},
        {"name": "fats", "value": 10.0, "fill": "var(--color-fats)"}
    ]

    # Check budget chart
    budget_chart = next(e for e in ui_elements if e["title"] == "Estimated Cost Summary")
    assert budget_chart["type"] == "chart"
    b_config = json.loads(budget_chart["config_json"])
    assert b_config["chart_type"] == "bar"
    assert b_config["index_key"] == "name"
    assert b_config["value_key"] == "value"
    assert b_config["config"]["cost"]["label"] == "Total Cost"
    b_data = json.loads(budget_chart["data_json"])
    assert b_data == [
        {"name": "cost", "value": 50.0, "fill": "var(--color-cost)"},
        {"name": "savings", "value": 10.0, "fill": "var(--color-savings)"}
    ]

    # Check timeline
    timeline = next(e for e in ui_elements if e["title"] == "Meal Schedule")
    assert timeline["type"] == "timeline"

    # 2. Assert final_text Markdown generation
    final_text = result["final_text"]
    assert "# Your ForkFit Nutrition & Meal Plan" in final_text
    assert "## Nutrition Summary" in final_text
    assert "**Calories**: 400.0 kcal" in final_text
    assert "**Protein**: 20.0g" in final_text
    assert "## Daily Meal Log" in final_text
    assert "### 2026-06-10" in final_text
    assert "**Breakfast**: Egg Toast" in final_text
    assert "## Shopping List" in final_text
    assert "Egg: 2.0 pcs (produce)" in final_text
    assert "**Total Estimated Plan Cost**: ₹50.00" in final_text

    mock_callback.assert_called_once()
