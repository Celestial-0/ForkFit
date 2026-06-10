"""gRPC servicer implementing the ForkFit IntelligenceService.

Wires together intent classification, the LangGraph agent pipeline, and the
reflection learning loop behind a production-grade gRPC streaming interface.
"""
from __future__ import annotations

import asyncio
import json
from typing import Any, AsyncIterator
from uuid import uuid4

import asyncpg
import grpc
import structlog
from langgraph.graph.state import CompiledStateGraph

from src.agents.intent import classify_intent
from src.config import Settings
from src.db.repositories.memory_repo import get_active_memories
from src.db.repositories.user_repo import (
    get_latest_biometrics,
    get_medical_safety_profile,
    get_recent_workouts,
    get_user_goals,
    get_user_preferences,
    get_user_profile,
)
from src.db.repositories.vector_repo import (
    search_memories_by_embedding,
    search_recipes_by_embedding,
)
from src.generated import intelligence_pb2, intelligence_pb2_grpc
from src.models.context import UserContextPackage
from src.reflection.engine import run_reflection
from src.services.embedding import embed_text

logger = structlog.get_logger()


class IntelligenceServiceServicer(
    intelligence_pb2_grpc.IntelligenceServiceServicer,
):
    """Concrete implementation of the IntelligenceService gRPC service."""

    def __init__(
        self,
        pool: asyncpg.Pool,
        settings: Settings,
        graph: CompiledStateGraph,
    ) -> None:
        self.pool = pool
        self.settings = settings
        self.graph = graph

    # ── ProcessIntent (unary-unary) ─────────────────────────────────

    async def ProcessIntent(
        self,
        request: intelligence_pb2.IntentRequest,
        context: grpc.aio.ServicerContext,
    ) -> intelligence_pb2.IntentResponse:
        """Classify a raw user prompt into a structured intent."""
        logger.info(
            "process_intent_request",
            user_id=request.user_id,
            prompt=request.prompt[:80],
        )

        try:
            blueprint = await classify_intent(
                prompt=request.prompt,
                user_id=request.user_id,
                settings=self.settings,
            )

            return intelligence_pb2.IntentResponse(
                goal=blueprint.goal,
                diet=blueprint.diet,
                budget_limit=blueprint.budget_limit,
                budget_currency=blueprint.budget_currency,
                timeline=blueprint.timeline,
                constraints=list(blueprint.constraints),
                raw_analysis_json=blueprint.raw_analysis_json,
            )

        except Exception as exc:  # noqa: BLE001
            logger.error("process_intent_error", error=str(exc))
            await context.abort(
                grpc.StatusCode.INTERNAL,
                f"Intent classification failed: {exc}",
            )

    # ── OrchestrateAgentGraph (unary-stream) ────────────────────────

    async def OrchestrateAgentGraph(
        self,
        request: intelligence_pb2.OrchestrateGraphRequest,
        context: grpc.aio.ServicerContext,
    ) -> AsyncIterator[intelligence_pb2.OrchestrateGraphResponse]:
        """Execute the full agent graph, streaming step updates to the client."""
        trace_id = request.trace_id
        user_id = request.context.user_id
        logger.info(
            "orchestrate_request",
            trace_id=trace_id,
            session_id=request.session_id,
            user_id=user_id,
        )

        try:
            # ── 1. Build rich context from DB + proto request ───────
            context_pkg = await self._build_context(request)

            # ── 2. Prepare streaming queue ─────────────────────────
            stream_queue: asyncio.Queue[dict[str, Any] | None] = asyncio.Queue()

            async def step_callback(step_data: dict[str, Any]) -> None:
                await stream_queue.put({"type": "step", "data": step_data})

            async def token_callback(token_data: dict[str, Any]) -> None:
                await stream_queue.put({"type": "token", "data": token_data})

            # ── 3. Assemble initial graph state ─────────────────────
            chat_history = [
                {"role": msg.role, "content": msg.content}
                for msg in request.history
            ]

            initial_state: dict[str, Any] = {
                "user_id": user_id,
                "prompt": request.prompt,
                "context": context_pkg.model_dump(),
                "chat_history": chat_history,
                "replan_count": 0,
                "ui_elements": [],
                "final_text": "",
                "_pool": self.pool,
                "_settings": self.settings,
                "_step_callback": step_callback,
                "_token_callback": token_callback,
            }

            # ── 4. Run graph in background, stream from queue ───────
            final_state_holder: dict[str, Any] = {}

            async def run_graph() -> None:
                try:
                    async for event in self.graph.astream(initial_state):
                        is_node_update = any(isinstance(v, dict) for v in event.values())
                        if is_node_update:
                            for node_output in event.values():
                                if isinstance(node_output, dict):
                                    final_state_holder.update(node_output)
                        else:
                            final_state_holder.update(event)
                except Exception as exc:  # noqa: BLE001
                    logger.error("graph_execution_error", error=str(exc))
                    await stream_queue.put(
                        {
                            "type": "error",
                            "message": str(exc),
                        }
                    )
                finally:
                    await stream_queue.put(None)  # sentinel

            task = asyncio.create_task(run_graph())

            # Stream step updates and tokens as they arrive
            while True:
                item = await stream_queue.get()
                if item is None:
                    break

                if item["type"] == "step":
                    step = item["data"]
                    yield intelligence_pb2.OrchestrateGraphResponse(
                        trace_id=trace_id,
                        step_update=intelligence_pb2.AgentStepUpdate(
                            step_id=str(uuid4()),
                            agent_name=step.get("agent_name", "unknown"),
                            status=step.get("status", "completed"),
                            step_type=step.get("step_type", "llm_call"),
                            input_payload_json=step.get(
                                "input_payload_json", "{}"
                            ),
                            output_payload_json=step.get(
                                "output_payload_json", "{}"
                            ),
                            latency_ms=step.get("latency_ms", 0),
                            error_message=step.get("error_message", ""),
                        ),
                    )
                elif item["type"] == "token":
                    token = item["data"]
                    yield intelligence_pb2.OrchestrateGraphResponse(
                        trace_id=trace_id,
                        text_delta=intelligence_pb2.TextDelta(
                            content=token.get("content", ""),
                            delta_index=token.get("delta_index", 0),
                            is_complete=token.get("is_complete", False),
                            delta_type=token.get("delta_type", "markdown"),
                        ),
                    )
                elif item["type"] == "error":
                    yield intelligence_pb2.OrchestrateGraphResponse(
                        trace_id=trace_id,
                        step_update=intelligence_pb2.AgentStepUpdate(
                            step_id=str(uuid4()),
                            agent_name="system",
                            status="error",
                            step_type="error",
                            error_message=item["message"],
                        ),
                    )

            # Ensure the task completes and propagate exceptions
            await task

            # ── 5. Yield UI elements ────────────────────────────────
            for ui in final_state_holder.get("ui_elements", []):
                yield intelligence_pb2.OrchestrateGraphResponse(
                    trace_id=trace_id,
                    ui_element=intelligence_pb2.UIElement(
                        type=ui.get("type", "card"),
                        title=ui.get("title", ""),
                        config_json=ui.get("config_json", "{}"),
                        data_json=ui.get("data_json", "{}"),
                    ),
                )

            # ── 6. Yield final text (Deprecated - for backward compat)
            yield intelligence_pb2.OrchestrateGraphResponse(
                trace_id=trace_id,
                final_text=final_state_holder.get("final_text", ""),
            )

        except Exception as exc:  # noqa: BLE001
            logger.error("orchestrate_fatal_error", error=str(exc))
            yield intelligence_pb2.OrchestrateGraphResponse(
                trace_id=trace_id,
                step_update=intelligence_pb2.AgentStepUpdate(
                    step_id=str(uuid4()),
                    agent_name="system",
                    status="error",
                    step_type="error",
                    error_message=str(exc),
                ),
            )

    # ── TriggerReflection (unary-unary) ─────────────────────────────

    async def TriggerReflection(
        self,
        request: intelligence_pb2.ReflectionRequest,
        context: grpc.aio.ServicerContext,
    ) -> intelligence_pb2.ReflectionResponse:
        """Run the reflection learning loop on user feedback."""
        logger.info(
            "trigger_reflection_request",
            user_id=request.user_id,
            rating=request.feedback_rating,
        )

        try:
            extracted = await run_reflection(
                pool=self.pool,
                settings=self.settings,
                user_id=request.user_id,
                session_id=request.session_id,
                chat_message_id=request.chat_message_id,
                feedback_rating=request.feedback_rating,
                feedback_text=request.feedback_text,
            )

            return intelligence_pb2.ReflectionResponse(
                success=True,
                extracted_memories=extracted,
            )

        except Exception as exc:  # noqa: BLE001
            logger.error("trigger_reflection_error", error=str(exc))
            return intelligence_pb2.ReflectionResponse(
                success=False,
                extracted_memories=[],
            )

    # ── Private helpers ─────────────────────────────────────────────

    async def _build_context(
        self,
        request: intelligence_pb2.OrchestrateGraphRequest,
    ) -> UserContextPackage:
        """Hydrate a full ``UserContextPackage`` from DB + proto fields."""
        user_id = request.context.user_id
        proto_ctx = request.context

        # Fan-out independent DB queries concurrently
        (
            safety_profile,
            goals,
            biometrics,
            profile,
            workouts,
            preferences,
            active_memories,
        ) = await asyncio.gather(
            get_medical_safety_profile(self.pool, user_id),
            get_user_goals(self.pool, user_id),
            get_latest_biometrics(self.pool, user_id),
            get_user_profile(self.pool, user_id),
            get_recent_workouts(self.pool, user_id),
            get_user_preferences(self.pool, user_id),
            get_active_memories(self.pool, user_id),
        )

        # Embed prompt for semantic search
        prompt_embedding = await embed_text(request.prompt, self.settings)

        # Semantic search: recipes + memories
        candidate_recipes, relevant_memories = await asyncio.gather(
            search_recipes_by_embedding(self.pool, prompt_embedding),
            search_memories_by_embedding(
                self.pool, user_id, prompt_embedding
            ),
        )

        # Merge proto-supplied fields with DB data
        allergies = list(proto_ctx.allergies) or safety_profile.get(
            "allergies", []
        )
        medical_conditions = list(
            proto_ctx.medical_conditions
        ) or safety_profile.get("conditions", [])

        return UserContextPackage(
            user_id=user_id,
            allergies=allergies,
            medical_conditions=medical_conditions,
            goals=goals,
            biometrics=biometrics,
            profile=profile,
            recent_workouts=workouts,
            preferences=preferences,
            active_memories=active_memories,
            candidate_recipes=candidate_recipes,
            relevant_memories=relevant_memories,
            daily_calorie_target=proto_ctx.daily_calorie_target or None,
            target_protein_g=proto_ctx.target_protein_g or None,
            target_carbs_g=proto_ctx.target_carbs_g or None,
            target_fat_g=proto_ctx.target_fat_g or None,
            budget_limit=proto_ctx.budget_limit or None,
            preferred_cuisine=proto_ctx.preferred_cuisine or None,
            preferred_foods=list(proto_ctx.preferred_foods),
            avoided_foods=list(proto_ctx.avoided_foods),
        )
