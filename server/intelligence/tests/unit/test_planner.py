from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.config import Settings
from src.agents.planner import planner_node, PlannerDecision


@pytest.mark.asyncio
@patch("src.agents.planner.get_chat_model")
async def test_planner_node_success(mock_get_chat_model):
    # Setup mocks
    mock_llm = MagicMock()
    mock_structured_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    mock_llm.with_structured_output.return_value = mock_structured_llm
    
    # Mock return value of ainvoke
    mock_decision = PlannerDecision(
        required_agents=["safety", "nutrition", "recipe"],
        execution_plan="We need safety for allergies, nutrition for macros, recipe for retrieval."
    )
    mock_structured_llm.ainvoke = AsyncMock(return_value=mock_decision)
    
    # Callbacks
    mock_callback = AsyncMock()
    
    # Construct state
    settings = Settings(database_url="postgresql://localhost/db")
    state = {
        "user_id": "test-user",
        "prompt": "weekly weight loss plan",
        "context": {
            "allergies": ["peanut"],
            "medical_conditions": []
        },
        "_settings": settings,
        "_step_callback": mock_callback,
    }
    
    # Run planner node
    result = await planner_node(state)
    
    # Assert result keys and values
    assert result["required_agents"] == ["safety", "nutrition", "recipe"]
    assert "safety" in result["required_agents"]
    assert "nutrition" in result["required_agents"]
    assert "recipe" in result["required_agents"]
    assert result["execution_plan"] == "We need safety for allergies, nutrition for macros, recipe for retrieval."
    
    # Verify callback was invoked
    mock_callback.assert_called_once()
    
    # Verify mock interactions
    mock_llm.with_structured_output.assert_called_once_with(PlannerDecision, method="json_mode")
    mock_structured_llm.ainvoke.assert_called_once()
