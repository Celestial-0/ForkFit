from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from src.config import Settings
from src.reflection.engine import run_reflection


@pytest.mark.asyncio
@patch("src.reflection.engine.get_chat_model")
@patch("src.reflection.engine.create_memory")
@patch("src.reflection.engine.embed_text")
@patch("src.reflection.engine.create_memory_embedding")
async def test_run_reflection_llm_success(
    mock_create_embedding,
    mock_embed,
    mock_create_memory,
    mock_get_chat_model
):
    # Setup LLM response containing extracted memories as JSON
    mock_llm = MagicMock()
    mock_get_chat_model.return_value = mock_llm
    
    mock_response = MagicMock()
    mock_response.content = """
    [
        {"type": "preference", "content": "I like paneer", "confidence": 0.9},
        {"type": "restriction", "content": "I avoid mushrooms", "confidence": 0.95}
    ]
    """
    mock_llm.ainvoke = AsyncMock(return_value=mock_response)

    # Setup database mocks
    mock_create_memory.side_effect = lambda pool, user_id, memory_type, content, confidence: f"{memory_type}-uuid"
    mock_embed.return_value = [0.1] * 2560

    settings = Settings(database_url="postgresql://localhost/db")
    pool = MagicMock()

    result = await run_reflection(
        pool=pool,
        settings=settings,
        user_id="user-123",
        session_id="session-456",
        chat_message_id="msg-789",
        feedback_rating=4,
        feedback_text="The paneer was great but please avoid mushrooms"
    )

    assert result == ["I like paneer", "I avoid mushrooms"]
    
    # Assert DB and embedding calls were made twice (one for each memory)
    assert mock_create_memory.call_count == 2
    assert mock_embed.call_count == 2
    assert mock_create_embedding.call_count == 2


@pytest.mark.asyncio
@patch("src.reflection.engine.create_memory")
@patch("src.reflection.engine.embed_text")
@patch("src.reflection.engine.create_memory_embedding")
async def test_run_reflection_implicit_feedback(
    mock_create_embedding,
    mock_embed,
    mock_create_memory
):
    # Mocks setup
    mock_create_memory.return_value = "memory-uuid"
    mock_embed.return_value = [0.1] * 2560
    
    settings = Settings(database_url="postgresql://localhost/db")
    pool = MagicMock()

    # Case 1: Rating = 5, empty text -> satisfies positive implicit feedback
    res_pos = await run_reflection(
        pool=pool,
        settings=settings,
        user_id="user-123",
        session_id="session-456",
        chat_message_id="msg-789",
        feedback_rating=5,
        feedback_text=""
    )
    assert res_pos == ["User was satisfied with the plan"]
    mock_create_memory.assert_called_with(
        pool,
        user_id="user-123",
        memory_type="habit",
        content="User was satisfied with the plan",
        confidence=0.5
    )

    # Reset mocks
    mock_create_memory.reset_mock()
    mock_embed.reset_mock()
    mock_create_embedding.reset_mock()

    # Case 2: Rating = 1, empty text -> satisfies negative implicit feedback
    res_neg = await run_reflection(
        pool=pool,
        settings=settings,
        user_id="user-123",
        session_id="session-456",
        chat_message_id="msg-789",
        feedback_rating=1,
        feedback_text="  "  # Whitespaces
    )
    assert res_neg == ["User was dissatisfied, plan needs improvement"]
    mock_create_memory.assert_called_with(
        pool,
        user_id="user-123",
        memory_type="habit",
        content="User was dissatisfied, plan needs improvement",
        confidence=0.5
    )

    # Reset mocks
    mock_create_memory.reset_mock()

    # Case 3: Rating = 3, empty text -> no memories generated
    res_neutral = await run_reflection(
        pool=pool,
        settings=settings,
        user_id="user-123",
        session_id="session-456",
        chat_message_id="msg-789",
        feedback_rating=3,
        feedback_text=""
    )
    assert res_neutral == []
    mock_create_memory.assert_not_called()
