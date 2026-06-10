from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.config import Settings
from src.models.intent import IntentBlueprint
from src.agents.intent import classify_intent


@pytest.mark.asyncio
@patch("src.agents.intent.get_chat_model")
async def test_classify_intent_success(mock_get_chat_model):
    # Setup mocks
    mock_llm = MagicMock()
    mock_structured_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    mock_llm.with_structured_output.return_value = mock_structured_llm
    
    # Mock return value of ainvoke
    mock_blueprint = IntentBlueprint(
        goal="weight_loss",
        diet="vegetarian",
        budget_limit=300.0,
        budget_currency="INR",
        timeline="weekly",
        constraints=["no mushrooms"],
        raw_analysis={"reasoning": "mocked reasoning"}
    )
    mock_structured_llm.ainvoke = AsyncMock(return_value=mock_blueprint)
    
    # Run classification
    settings = Settings(database_url="postgresql://localhost/db")
    result = await classify_intent(
        prompt="I need a weekly vegetarian weight loss plan under INR 300 per day. Please avoid mushrooms.",
        user_id="test-user",
        settings=settings
    )
    
    # Assert result attributes
    assert result.goal == "weight_loss"
    assert result.diet == "vegetarian"
    assert result.budget_limit == 300.0
    assert result.budget_currency == "INR"
    assert result.timeline == "weekly"
    assert "no mushrooms" in result.constraints
    assert result.raw_analysis["reasoning"] == "mocked reasoning"
    
    # Verify mock interactions
    mock_llm.with_structured_output.assert_called_once_with(IntentBlueprint)
    mock_structured_llm.ainvoke.assert_called_once()
