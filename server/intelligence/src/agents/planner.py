"""Cognitive Planner Agent Node for the ForkFit intelligence pipeline.

Analyzes user prompt and context to decide which agents are required.
"""
from __future__ import annotations

import structlog
from pydantic import BaseModel, Field
from src.agents.helpers import Timer, emit_step
from src.agents.state import GraphState
from src.services.llm import get_chat_model

logger = structlog.get_logger()


class PlannerDecision(BaseModel):
    """Structured decision output from the Planner LLM."""

    required_agents: list[str] = Field(
        ...,
        description="List of agent node names that MUST run. Choices: 'safety', 'nutrition', 'budget', 'culture', 'recipe', 'calendar', 'shopping'.",
    )
    execution_plan: str = Field(
        ...,
        description="Detailed text explaining the reasoning for the chosen agents and the workflow steps.",
    )


async def planner_node(state: GraphState) -> dict[str, Any]:
    """Decide which agents are required for the current request."""
    logger.info("planner_node_started", user_id=state.get("user_id"))
    timer = Timer()

    # Get dependencies from state
    settings = state["_settings"]
    callback = state.get("_step_callback")

    input_data = {
        "prompt": state.get("prompt"),
        "user_id": state.get("user_id"),
        "context_keys": list(state.get("context", {}).keys()),
    }

    with timer:
        llm = get_chat_model(settings)
        structured_llm = llm.with_structured_output(PlannerDecision, method="json_mode")

        system_prompt = (
            "You are the Cognitive Planner for ForkFit. Your role is to analyze the user's prompt "
            "and the provided context to decide which specialist agents need to run.\n"
            "Available agents:\n"
            "- 'safety': Runs allergen/medical checks. (Include if user has allergies or medical conditions in context).\n"
            "- 'nutrition': Computes BMR, TDEE, and macro splits. (Include if user wants a diet plan or goal targets).\n"
            "- 'budget': Evaluates cost and suggests cheap alternatives. (Include if user has a non-zero budget limit in context or specifies budget limits).\n"
            "- 'culture': Filters recipes by cuisine and religious/cultural tags. (Include if preferred cuisine or diet preferences exist).\n"
            "- 'recipe': Handles final recipe selection and semantic retrieval. (Always include for meal/recipe queries).\n"
            "- 'calendar': Schedules meals and adjusts pre/post workout slots. (Always include for meal, recipe, or meal plan queries).\n"
            "- 'shopping': Aggregates groceries and subtracts pantry stock. (Always include for meal, recipe, or meal plan queries).\n\n"
            "You MUST respond with a JSON object containing the following keys:\n"
            "- \"required_agents\": A list of strings representing the chosen agent names (from the available agents above).\n"
            "- \"execution_plan\": A detailed string explaining the reasoning for the chosen agents and the workflow steps.\n\n"
            "Do NOT wrap the JSON response in markdown code blocks or any other formatting."
        )

        try:
            decision = await structured_llm.ainvoke([
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": f"Prompt: {state.get('prompt')}\nContext: {state.get('context')}"}
            ])

            required_agents = decision.required_agents
            # Always ensure planner is not in list, and it's valid names
            valid_agents = {"safety", "nutrition", "budget", "culture", "recipe", "calendar", "shopping"}
            required_agents = [a for a in required_agents if a in valid_agents]
            
            # Default fallback if LLM returns empty list
            if not required_agents:
                required_agents = ["safety", "nutrition", "recipe"]

            execution_plan = decision.execution_plan
            logger.info("planner_node_decision", required_agents=required_agents)

        except Exception as exc:
            logger.error("planner_node_failed", error=str(exc))
            required_agents = ["safety", "nutrition", "recipe", "calendar", "shopping"]
            execution_plan = f"Fallback execution plan due to error: {exc}"

    output_data = {
        "required_agents": required_agents,
        "execution_plan": execution_plan,
    }

    await emit_step(
        callback,
        agent_name="Planner",
        status="completed",
        step_type="llm_call",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "required_agents": required_agents,
        "execution_plan": execution_plan,
    }
