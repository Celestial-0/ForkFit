"""Judge Agent Node and Router for the ForkFit intelligence pipeline.

Evaluates plan quality against constraints and controls the replan routing.
"""
from __future__ import annotations

import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import GraphState, JudgeVerdict

logger = structlog.get_logger()


async def judge_node(state: GraphState) -> dict[str, Any]:
    """Evaluate plan adherence and determine if replanning is needed."""
    logger.info("judge_node_started", user_id=state.get("user_id"))
    timer = Timer()

    callback = state.get("_step_callback")
    settings = state["_settings"]
    max_replan = settings.max_replan_attempts

    merged_plan = state.get("merged_plan") or {}
    daily_plans = merged_plan.get("daily_plans", [])
    nutrition_summary = merged_plan.get("nutrition_summary", {})
    confidence = merged_plan.get("confidence", 1.0)

    context = state.get("context", {})
    budget_limit = float(context.get("budget_limit", 0.0))
    total_cost = float(merged_plan.get("total_cost", 0.0))

    failures: list[str] = []
    passed = True

    input_data = {
        "replan_count": state.get("replan_count", 0),
        "confidence": confidence,
        "total_cost": total_cost,
    }

    with timer:
        # 1. Check budget compliance
        if budget_limit > 0.0 and total_cost > budget_limit:
            failures.append(f"Plan cost ({total_cost}) exceeds budget limit ({budget_limit})")
            passed = False

        # 2. Check safety (are there any unresolved substitutes?)
        safety_violations = 0
        for plan in daily_plans:
            for meal in plan.get("meals", []):
                title = meal.get("recipe_title", "")
                if "SAFE SUBSTITUTE NEEDED" in title:
                    safety_violations += 1
        
        if safety_violations > 0:
            failures.append(f"Plan contains {safety_violations} unsafe meals needing substitution")
            passed = False

        # 3. Check variety: no recipe repeated more than twice in a week
        recipe_counts: dict[str, int] = {}
        for plan in daily_plans:
            for meal in plan.get("meals", []):
                rid = meal.get("recipe_id")
                if rid:
                    recipe_counts[rid] = recipe_counts.get(rid, 0) + 1
        
        for rid, count in recipe_counts.items():
            if count > 2:
                failures.append(f"Recipe ID {rid} is repeated too many times ({count} repeats)")
                # We can flag it as a warning or strict failure. Let's make it a failure if > 3 times
                if count > 3:
                    passed = False

        # Generate replan instructions if failed
        replan_instructions = None
        if not passed:
            replan_instructions = "; ".join(failures)

    # Increment replan count in output
    replan_count = state.get("replan_count", 0)
    if not passed:
        replan_count += 1

    output_data = {
        "passed": passed,
        "failures": failures,
        "replan_count": replan_count,
        "replan_instructions": replan_instructions,
    }

    await emit_step(
        callback,
        agent_name="Judge",
        status="completed",
        step_type="validation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    verdict: JudgeVerdict = {
        "passed": passed,
        "confidence": confidence,
        "failures": failures,
        "replan_instructions": replan_instructions,
    }

    return {
        "judge_verdict": verdict,
        "replan_count": replan_count,
    }


def judge_router(state: GraphState) -> str:
    """Route execution back to the planner if validation failed, otherwise to visualization."""
    verdict = state.get("judge_verdict") or {}
    passed = verdict.get("passed", True)
    replan_count = state.get("replan_count", 0)
    
    settings = state["_settings"]
    max_replan = settings.max_replan_attempts

    if passed or replan_count >= max_replan:
        logger.info("judge_router_to_visualization", passed=passed, replan_count=replan_count)
        return "visualization"
    
    logger.info("judge_router_to_planner", passed=passed, replan_count=replan_count)
    return "planner"
