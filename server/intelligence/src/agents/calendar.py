"""Calendar Agent Node for the ForkFit intelligence pipeline.

Schedules selected meals into daily/weekly slots, taking workouts into account.
"""
from __future__ import annotations

import datetime
import structlog
from pydantic import BaseModel, Field
from src.agents.helpers import Timer, emit_step
from src.agents.state import CalendarResult, DailySchedulePlan, GraphState
from src.services.llm import get_chat_model

logger = structlog.get_logger()


class MealAssignment(BaseModel):
    """A recipe assigned to a meal slot."""

    meal_type: str = Field(..., description="breakfast, lunch, dinner, snack")
    recipe_id: str = Field(..., description="UUID string of the selected recipe")
    servings: float = Field(default=1.0, description="Servings portion")


class DailySchedule(BaseModel):
    """Meal assignments for a single day."""

    day_index: int = Field(..., description="Index of the day starting at 1")
    meals: list[MealAssignment] = Field(default_factory=list)
    notes: str = Field(default="", description="Specific daily suggestions or workout adjustments")


class CalendarDecision(BaseModel):
    """Complete calendar plan decision."""

    schedules: list[DailySchedule] = Field(..., description="Schedules for each day of the period")
    general_notes: str = Field(default="", description="General execution notes")


async def calendar_node(state: GraphState) -> dict[str, CalendarResult]:
    """Schedule selected recipes into daily slots based on context timeline and workouts."""
    if "calendar" not in state.get("required_agents", []):
        logger.info("calendar_node_skipped")
        return {}

    logger.info("calendar_node_started", user_id=state.get("user_id"))
    timer = Timer()

    settings = state["_settings"]
    callback = state.get("_step_callback")

    context = state.get("context", {})
    timeline = context.get("timeline", "weekly")
    days_count = 7 if timeline == "weekly" else 1

    recipe_result = state.get("recipe_result") or {}
    selected_recipes = recipe_result.get("selected_recipes", [])

    input_data = {
        "timeline": timeline,
        "days_count": days_count,
        "selected_recipes_count": len(selected_recipes),
        "recent_workouts_count": len(context.get("recent_workouts", [])),
    }

    daily_plans: list[DailySchedulePlan] = []

    with timer:
        if not selected_recipes:
            logger.warning("calendar_node_no_recipes_selected")
        else:
            llm = get_chat_model(settings)
            structured_llm = llm.with_structured_output(CalendarDecision)

            recipes_summary = "\n".join(
                f"- ID: {r['recipe_id']} | Title: {r['title']} | Nutrition: {r.get('nutrition')}"
                for r in selected_recipes
            )

            prompt = (
                f"User timeline: {timeline} ({days_count} days).\n"
                f"Recent workouts: {context.get('recent_workouts', [])}\n"
                f"Selected recipes available:\n"
                f"{recipes_summary}\n"
                f"Schedule these recipes into meal slots (breakfast, lunch, dinner, snack) for {days_count} days. "
                f"If workouts are present, schedule a snack slot pre- or post-workout and adjust portion suggestions in notes."
            )

            try:
                decision = await structured_llm.ainvoke([
                    {"role": "system", "content": "You are a meal scheduling assistant and fitness coach."},
                    {"role": "user", "content": prompt}
                ])

                today = datetime.date.today()
                for sched in decision.schedules:
                    day_date = today + datetime.timedelta(days=sched.day_index - 1)
                    
                    meals_list = []
                    for m in sched.meals:
                        # Find full recipe details to get nutrition/cost
                        recipe_detail = next((r for r in selected_recipes if r["recipe_id"] == m.recipe_id), {})
                        
                        meals_list.append({
                            "meal_type": m.meal_type,
                            "recipe_id": m.recipe_id,
                            "recipe_title": recipe_detail.get("title", "Unknown"),
                            "servings": m.servings,
                            "nutrition": recipe_detail.get("nutrition", {}),
                            "cost": recipe_detail.get("cost", 0.0) * m.servings,
                        })

                    daily_plans.append({
                        "date": day_date.isoformat(),
                        "meals": meals_list,
                        "notes": sched.notes,
                    })

            except Exception as exc:
                logger.error("calendar_scheduling_failed", error=str(exc))
                # Fallback: simple loop schedule
                today = datetime.date.today()
                for i in range(days_count):
                    day_date = today + datetime.timedelta(days=i)
                    meals_list = []
                    
                    # Distribute selected recipes
                    if selected_recipes:
                        # Breakfast
                        r_bf = selected_recipes[i % len(selected_recipes)]
                        meals_list.append({
                            "meal_type": "breakfast",
                            "recipe_id": r_bf["recipe_id"],
                            "recipe_title": r_bf["title"],
                            "servings": 1.0,
                            "nutrition": r_bf.get("nutrition", {}),
                            "cost": r_bf.get("cost", 0.0),
                        })
                        # Lunch
                        r_lh = selected_recipes[(i + 1) % len(selected_recipes)]
                        meals_list.append({
                            "meal_type": "lunch",
                            "recipe_id": r_lh["recipe_id"],
                            "recipe_title": r_lh["title"],
                            "servings": 1.0,
                            "nutrition": r_lh.get("nutrition", {}),
                            "cost": r_lh.get("cost", 0.0),
                        })
                        # Dinner
                        r_dn = selected_recipes[(i + 2) % len(selected_recipes)]
                        meals_list.append({
                            "meal_type": "dinner",
                            "recipe_id": r_dn["recipe_id"],
                            "recipe_title": r_dn["title"],
                            "servings": 1.0,
                            "nutrition": r_dn.get("nutrition", {}),
                            "cost": r_dn.get("cost", 0.0),
                        })

                    daily_plans.append({
                        "date": day_date.isoformat(),
                        "meals": meals_list,
                        "notes": "Fallback schedule generated due to exception.",
                    })

    output_data = {
        "daily_plans_count": len(daily_plans),
    }

    await emit_step(
        callback,
        agent_name="Calendar",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "calendar_result": {
            "daily_plans": daily_plans,
        }
    }
