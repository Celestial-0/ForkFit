"""Visualization Agent Node for the ForkFit intelligence pipeline.

Generates frontend UI component JSON configurations and markdown summaries.
"""
from __future__ import annotations

from typing import Any
import json
import structlog
from src.agents.helpers import Timer, emit_step
from src.agents.state import GraphState, UIElementSpecDict
from src.services.llm import get_chat_model

logger = structlog.get_logger()


async def visualization_node(state: GraphState) -> dict[str, Any]:
    """Generate UI specs and user-facing markdown text summary."""
    logger.info("visualization_node_started", user_id=state.get("user_id"))
    timer = Timer()

    callback = state.get("_step_callback")
    merged_plan = state.get("merged_plan") or {}
    daily_plans = merged_plan.get("daily_plans", [])
    shopping_list = merged_plan.get("shopping_list", [])
    nutrition_summary = merged_plan.get("nutrition_summary", {})
    total_cost = merged_plan.get("total_cost", 0.0)

    input_data = {
        "daily_plans_count": len(daily_plans),
        "shopping_list_count": len(shopping_list),
    }

    ui_elements: list[UIElementSpecDict] = []
    final_text = ""

    with timer:
        # 1. Create Macro Distribution Chart
        p = nutrition_summary.get("protein_g", 0.0)
        c = nutrition_summary.get("carbs_g", 0.0)
        f = nutrition_summary.get("fat_g", 0.0)
        
        ui_elements.append({
            "type": "chart",
            "title": "Daily Macronutrient Split (Average)",
            "config_json": json.dumps({
                "chart_type": "doughnut",
                "index_key": "name",
                "value_key": "value",
                "config": {
                    "protein": {
                        "label": "Protein (g)",
                        "color": "#4F46E5"
                    },
                    "carbs": {
                        "label": "Carbohydrates (g)",
                        "color": "#10B981"
                    },
                    "fats": {
                        "label": "Fats (g)",
                        "color": "#F59E0B"
                    }
                }
            }),
            "data_json": json.dumps([
                {"name": "protein", "value": p, "fill": "var(--color-protein)"},
                {"name": "carbs", "value": c, "fill": "var(--color-carbs)"},
                {"name": "fats", "value": f, "fill": "var(--color-fats)"}
            ])
        })

        # 2. Create Budget Breakdown
        ui_elements.append({
            "type": "chart",
            "title": "Estimated Cost Summary",
            "config_json": json.dumps({
                "chart_type": "bar",
                "index_key": "name",
                "value_key": "value",
                "config": {
                    "cost": {
                        "label": "Total Cost",
                        "color": "#EF4444"
                    },
                    "savings": {
                        "label": "Pantry Savings",
                        "color": "#10B981"
                    }
                }
            }),
            "data_json": json.dumps([
                {"name": "cost", "value": total_cost, "fill": "var(--color-cost)"},
                {"name": "savings", "value": merged_plan.get("pantry_savings", 0.0), "fill": "var(--color-savings)"}
            ])
        })

        # 3. Create Timeline
        timeline_events = []
        for plan in daily_plans:
            date_str = plan.get("date", "")
            for meal in plan.get("meals", []):
                timeline_events.append({
                    "time": date_str,
                    "title": meal.get("recipe_title", "Unknown Meal"),
                    "description": f"{meal.get('meal_type', '').capitalize()} — {meal.get('servings', 1.0)} serving(s)"
                })
        
        ui_elements.append({
            "type": "timeline",
            "title": "Meal Schedule",
            "config_json": json.dumps({
                "layout": "vertical"
            }),
            "data_json": json.dumps({
                "events": timeline_events
            })
        })

        # Create Meal Plan Element (type: "meal_plan")
        meal_plan_items = []
        start_date = None
        end_date = None
        
        for plan in daily_plans:
            date_str = plan.get("date", "")
            if date_str:
                if not start_date or date_str < start_date:
                    start_date = date_str
                if not end_date or date_str > end_date:
                    end_date = date_str
                
            for meal in plan.get("meals", []):
                meal_plan_items.append({
                    "planned_date": date_str,
                    "meal_type": meal.get("meal_type", "dinner").lower(),
                    "recipe_id": meal.get("recipe_id"),
                    "food_item_id": None,
                    "custom_food_name": meal.get("recipe_title"),
                    "servings": float(meal.get("servings", 1.0))
                })
        
        if daily_plans:
            ui_elements.append({
                "type": "meal_plan",
                "title": "Diet Meal Plan",
                "config_json": json.dumps({}),
                "data_json": json.dumps({
                    "name": "Weekly Meal Plan",
                    "start_date": start_date or "",
                    "end_date": end_date or "",
                    "is_active": True,
                    "items": meal_plan_items
                })
            })

        # Create Shopping List Element (type: "shopping_list")
        shopping_items_req = []
        for item in shopping_list:
            shopping_items_req.append({
                "food_item_id": None,
                "custom_item_name": item.get("food_item_name"),
                "quantity": float(item.get("quantity", 0.0)),
                "unit": item.get("unit", "g"),
                "category": item.get("category", "Other")
            })
            
        if shopping_list:
            ui_elements.append({
                "type": "shopping_list",
                "title": "Grocery Shopping List",
                "config_json": json.dumps({}),
                "data_json": json.dumps({
                    "name": "Weekly Grocery List",
                    "items": shopping_items_req
                })
            })

        # 4. Generate fallback programmatic markdown text summary
        p = nutrition_summary.get("protein_g", 0.0)
        c = nutrition_summary.get("carbs_g", 0.0)
        f = nutrition_summary.get("fat_g", 0.0)
        markdown_lines = [
            f"# Your ForkFit Nutrition & Meal Plan",
            f"",
            f"Here is your personalized meal plan tailored to your profile and fitness goal.",
            f"",
            f"## Nutrition Summary (Daily Averages)",
            f"- **Calories**: {nutrition_summary.get('calories', 0.0)} kcal",
            f"- **Protein**: {p}g",
            f"- **Carbohydrates**: {c}g",
            f"- **Fats**: {f}g",
            f"",
            f"## Daily Meal Log",
        ]

        for plan in daily_plans:
            markdown_lines.append(f"### {plan.get('date')}")
            if plan.get("notes"):
                markdown_lines.append(f"*{plan.get('notes')}*")
            
            for meal in plan.get("meals", []):
                title = meal.get("recipe_title")
                servings = meal.get("servings", 1.0)
                mtype = meal.get("meal_type", "").capitalize()
                markdown_lines.append(f"- **{mtype}**: {title} ({servings} serving(s))")
            markdown_lines.append("")

        if shopping_list:
            markdown_lines.append("## Shopping List")
            for item in shopping_list:
                markdown_lines.append(
                    f"- {item.get('food_item_name')}: {item.get('quantity')} {item.get('unit')} "
                    f"({item.get('category')}) — Est. Cost: ₹{item.get('estimated_cost')}"
                )
            markdown_lines.append(f"**Total Estimated Plan Cost**: ₹{total_cost:.2f}")

        programmatic_final_text = "\n".join(markdown_lines)

        # 5. Generate LLM-Synthesized Summary via ChatOllama if settings are available
        settings = state.get("_settings")
        token_callback = state.get("_token_callback")

        if settings:
            daily_plans_summary_list = []
            for plan in daily_plans:
                meals_str = ", ".join(f"{m.get('meal_type', '').capitalize()}: {m.get('recipe_title')}" for m in plan.get("meals", []))
                daily_plans_summary_str = "\n".join(daily_plans_summary_list)
            daily_plans_summary_str = "\n".join(daily_plans_summary_list)

            shopping_list_summary_str = ", ".join(
                f"{item.get('food_item_name')} ({item.get('quantity')} {item.get('unit')})"
                for item in shopping_list
            )

            user_prompt = state.get("prompt", "a healthy meal plan")
            formatted_prompt = (
                "You are the ForkFit AI presenter and personal health coach.\n"
                f"A user requested a meal plan with the prompt: '{user_prompt}'.\n"
                "Here are the finalized meal plan details we generated for them:\n"
                f"- Total Estimated Cost: ₹{total_cost:.2f}\n"
                f"- Nutrition Summary (Daily Average): {nutrition_summary}\n"
                f"- Weekly Daily Meals Log:\n{daily_plans_summary_str}\n"
                f"- Shopping List: {shopping_list_summary_str}\n\n"
                "Your task is to write a personalized, highly encouraging conversational presentation "
                "and summary of this meal plan. Do NOT just dump the data; synthesize it into an engaging "
                "and encouraging health narrative in markdown.\n"
                "Instructions:\n"
                "1. Write a warm, encouraging welcome explaining why this plan fits their goal.\n"
                "2. Break down the key nutritional achievements (protein, calories, fats) and explain "
                "   how it supports their objective.\n"
                "3. Highlight a couple of tasty meal choices scheduled in their daily plan.\n"
                "4. Include a neat, bulleted summary of their schedule and shopping highlights.\n"
                "5. Keep the tone professional, friendly, and motivating.\n"
                "6. Return ONLY the markdown presentation text. Do not include any preambles or notes outside markdown."
            )

            model = get_chat_model(settings, temperature=0.5)
            
            if token_callback:
                accumulated_chunks = []
                index = 0
                # Stream the tokens in real-time
                async for chunk in model.astream(formatted_prompt):
                    content_chunk = chunk.content
                    accumulated_chunks.append(content_chunk)
                    await token_callback({
                        "content": content_chunk,
                        "delta_index": index,
                        "is_complete": False,
                        "delta_type": "markdown"
                    })
                    index += 1
                
                # Yield the final complete sentinel delta
                await token_callback({
                    "content": "",
                    "delta_index": index,
                    "is_complete": True,
                    "delta_type": "markdown"
                })
                final_text = "".join(accumulated_chunks)
            else:
                # Synchronous fallback invocation
                logger.warning("no_token_callback_found_running_ainvoke")
                res = await model.ainvoke(formatted_prompt)
                final_text = res.content
        else:
            logger.warning("no_settings_found_running_fallback_programmatic_summary")
            final_text = programmatic_final_text

    output_data = {
        "ui_elements_count": len(ui_elements),
        "final_text_length": len(final_text),
    }

    await emit_step(
        callback,
        agent_name="Visualization",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "ui_elements": ui_elements,
        "final_text": final_text,
    }
