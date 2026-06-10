from unittest.mock import AsyncMock, MagicMock, patch
import pytest
import grpc

from src.generated import intelligence_pb2
from src.server.servicer import IntelligenceServiceServicer
from src.config import Settings
from src.models.intent import IntentBlueprint
from src.models.context import UserContextPackage


@pytest.fixture
def mock_servicer():
    pool = MagicMock()
    settings = Settings(database_url="postgresql://localhost/db")
    graph = MagicMock()
    servicer = IntelligenceServiceServicer(pool=pool, settings=settings, graph=graph)
    return servicer


@pytest.mark.asyncio
@patch("src.server.servicer.classify_intent")
async def test_process_intent(mock_classify, mock_servicer):
    # Mock classify_intent return
    mock_blueprint = IntentBlueprint(
        goal="muscle_gain",
        diet="vegan",
        budget_limit=500.0,
        budget_currency="INR",
        timeline="weekly",
        constraints=["no soy"],
        raw_analysis={"reason": "mocked"}
    )
    mock_classify.return_value = mock_blueprint

    request = intelligence_pb2.IntentRequest(
        user_id="user-1",
        prompt="Build vegan muscle gain plan under 500 INR without soy"
    )
    context = MagicMock(spec=grpc.aio.ServicerContext)

    response = await mock_servicer.ProcessIntent(request, context)

    assert response.goal == "muscle_gain"
    assert response.diet == "vegan"
    assert response.budget_limit == 500.0
    assert response.budget_currency == "INR"
    assert response.timeline == "weekly"
    assert "no soy" in response.constraints
    mock_classify.assert_called_once_with(
        prompt=request.prompt,
        user_id=request.user_id,
        settings=mock_servicer.settings
    )


@pytest.mark.asyncio
async def test_orchestrate_agent_graph(mock_servicer):
    # Mock database hydration helper
    mock_ctx_pkg = UserContextPackage(
        user_id="user-1",
        allergies=["peanut"],
        medical_conditions=[],
        preferred_foods=[],
        avoided_foods=[],
        calorie_target=2000.0,
        budget_limit=200.0,
        preferred_cuisine="Indian"
    )
    mock_servicer._build_context = AsyncMock(return_value=mock_ctx_pkg)

    # Mock graph execution stream (yielding events)
    async def mock_astream(initial_state):
        # Yield some step callback updates
        # (the real servicer captures step callbacks sent during execution, but also uses final state)
        # We mock astream to yield a final dict state representing completion
        yield {
            "ui_elements": [
                {
                    "type": "chart",
                    "title": "Macros",
                    "config_json": '{"type":"doughnut"}',
                    "data_json": '{"values":[20, 50, 30]}'
                }
            ],
            "final_text": "Here is your plan"
        }

    mock_servicer.graph.astream = mock_astream

    request = intelligence_pb2.OrchestrateGraphRequest(
        trace_id="trace-123",
        session_id="session-456",
        prompt="Indian vegetarian plan",
        context=intelligence_pb2.UserContext(user_id="user-1"),
        history=[]
    )
    context = MagicMock(spec=grpc.aio.ServicerContext)

    # Gather responses from generator
    responses = []
    async for resp in mock_servicer.OrchestrateAgentGraph(request, context):
        responses.append(resp)

    # Assert trace_id is preserved
    for r in responses:
        assert r.trace_id == "trace-123"

    # The last yielded item should contain final_text
    final_text_resp = next(r for r in responses if r.HasField("final_text"))
    assert final_text_resp.final_text == "Here is your plan"

    # One response should contain the UI element
    ui_resp = next(r for r in responses if r.HasField("ui_element"))
    assert ui_resp.ui_element.title == "Macros"
    assert ui_resp.ui_element.type == "chart"


@pytest.mark.asyncio
@patch("src.server.servicer.run_reflection")
async def test_trigger_reflection(mock_run_reflection, mock_servicer):
    mock_run_reflection.return_value = ["extracted preference: paneer"]

    request = intelligence_pb2.ReflectionRequest(
        user_id="user-1",
        session_id="session-1",
        chat_message_id="msg-1",
        feedback_rating=5,
        feedback_text="I liked the paneer meal"
    )
    context = MagicMock(spec=grpc.aio.ServicerContext)

    response = await mock_servicer.TriggerReflection(request, context)

    assert response.success is True
    assert response.extracted_memories == ["extracted preference: paneer"]
    mock_run_reflection.assert_called_once_with(
        pool=mock_servicer.pool,
        settings=mock_servicer.settings,
        user_id=request.user_id,
        session_id=request.session_id,
        chat_message_id=request.chat_message_id,
        feedback_rating=request.feedback_rating,
        feedback_text=request.feedback_text
    )
