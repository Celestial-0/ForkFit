"""LangGraph orchestrator for the ForkFit multi-agent pipeline.

Compiles a StateGraph with parallel specialist nodes to optimize latency.
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


def build_graph() -> CompiledStateGraph:
    """Construct and compile the full ForkFit agent graph.

    Execution order:
        planner ───┬──→ safety ─────┬──→ recipe ──→ calendar ──→ shopping ──→ consensus ──→ judge
                   ├──→ nutrition ──┤
                   ├──→ budget ─────┤
                   └──→ culture ────┘
                   
        judge ──→ (visualization ──→ END) | (planner — replan loop)

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

    # ── Fan-out from planner to parallel specialist agents ──────────
    graph.add_edge("planner", "safety")
    graph.add_edge("planner", "nutrition")
    graph.add_edge("planner", "budget")
    graph.add_edge("planner", "culture")

    # ── Fan-in from specialist agents to recipe compiler ────────────
    graph.add_edge("safety", "recipe")
    graph.add_edge("nutrition", "recipe")
    graph.add_edge("budget", "recipe")
    graph.add_edge("culture", "recipe")

    # ── Sequential remaining chain ──────────────────────────────────
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
