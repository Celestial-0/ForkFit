"""LangGraph orchestrator for the ForkFit multi-agent pipeline.

Compiles a deterministic sequential StateGraph that routes through all
nutrition-planning agents.  Each agent internally checks whether it is
listed in ``state["required_agents"]`` and performs a fast no-op return
when it is not needed, keeping the graph topology simple and predictable.
"""
from __future__ import annotations

import structlog
from langgraph.graph import END, StateGraph
from langgraph.graph.state import CompiledStateGraph

from src.agents.budget import budget_node
from src.agents.calendar import calendar_node
from src.agents.consensus import consensus_node
from src.agents.culture import culture_node
from src.agents.judge import judge_node, judge_router
from src.agents.nutrition import nutrition_node
from src.agents.planner import planner_node
from src.agents.recipe import recipe_node
from src.agents.safety import safety_node
from src.agents.shopping import shopping_node
from src.agents.state import GraphState
from src.agents.visualization import visualization_node

logger = structlog.get_logger()


def route_from_planner(state: GraphState) -> str:
    """Route from planner to the first agent in the sequential chain.

    Rather than building complex dynamic fan-out logic, every agent in the
    chain is always traversed.  Agents that are not in
    ``state["required_agents"]`` simply return the state unchanged, so
    latency cost is negligible.
    """
    required = state.get("required_agents", [])
    logger.info(
        "route_from_planner",
        required_agents=required,
        replan_count=state.get("replan_count", 0),
    )
    return "safety"


def build_graph() -> CompiledStateGraph:
    """Construct and compile the full ForkFit agent graph.

    Execution order (sequential chain):
        planner → safety → nutrition → budget → culture → recipe
        → calendar → shopping → consensus → judge
        → (visualization → END) | (planner — replan loop)

    Returns
    -------
    CompiledStateGraph
        A compiled LangGraph ready for ``astream`` / ``ainvoke``.
    """
    graph = StateGraph(GraphState)

    # ── Register every node ─────────────────────────────────────────
    graph.add_node("planner", planner_node)
    graph.add_node("safety", safety_node)
    graph.add_node("nutrition", nutrition_node)
    graph.add_node("budget", budget_node)
    graph.add_node("culture", culture_node)
    graph.add_node("recipe", recipe_node)
    graph.add_node("calendar", calendar_node)
    graph.add_node("shopping", shopping_node)
    graph.add_node("consensus", consensus_node)
    graph.add_node("judge", judge_node)
    graph.add_node("visualization", visualization_node)

    # ── Entry point ─────────────────────────────────────────────────
    graph.set_entry_point("planner")

    # ── Planner → first agent via conditional edge ──────────────────
    graph.add_conditional_edges("planner", route_from_planner)

    # ── Sequential chain through specialist agents ──────────────────
    graph.add_edge("safety", "nutrition")
    graph.add_edge("nutrition", "budget")
    graph.add_edge("budget", "culture")
    graph.add_edge("culture", "recipe")
    graph.add_edge("recipe", "calendar")
    graph.add_edge("calendar", "shopping")
    graph.add_edge("shopping", "consensus")
    graph.add_edge("consensus", "judge")

    # ── Judge conditional routing ───────────────────────────────────
    # judge_router returns "visualization" when the plan passes QA,
    # or "planner" to trigger a replan (max 2 replans enforced inside
    # the judge node itself).
    graph.add_conditional_edges(
        "judge",
        judge_router,
        {
            "planner": "planner",
            "visualization": "visualization",
        },
    )

    # ── Terminal edge ───────────────────────────────────────────────
    graph.add_edge("visualization", END)

    compiled = graph.compile()
    logger.info("graph_compiled", node_count=len(compiled.nodes))
    return compiled
