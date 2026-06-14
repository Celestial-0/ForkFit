"""Budget Agent Node for the ForkFit intelligence pipeline.

Computes recipe food item costs and optimizes for budget constraints.
"""
from __future__ import annotations

from typing import Any
import structlog
from langchain_core.tools import tool
from src.agents.helpers import Timer, emit_step
from src.agents.state import BudgetResult, GraphState, RecipeCost
from src.db.repositories.recipe_repo import calculate_recipe_cost, get_recipe_with_food_items
from src.services.llm import get_chat_model

logger = structlog.get_logger()


def make_tools(pool, settings):
    @tool
    async def get_raw_food_costs(foods: list[str]) -> list[dict[str, Any]]:
        """Look up existing raw food cost patterns in the database for a list of food names.
        
        Arguments:
        - foods: A list of food names/patterns to search.
        """
        results = []
        async with pool.acquire() as conn:
            for food in foods:
                rows = await conn.fetch(
                    """
                    SELECT food_pattern, cost_per_100g::float8 as cost_per_100g, price_currency
                      FROM raw_food_costs
                     WHERE food_pattern ILIKE $1 OR $2 ILIKE '%' || food_pattern || '%'
                    """,
                    f"%{food}%",
                    food
                )
                for r in rows:
                    results.append({
                        "food_pattern": r["food_pattern"],
                        "cost_per_100g": r["cost_per_100g"],
                        "price_currency": r["price_currency"]
                    })
        return results

    @tool
    async def search_food_item_market_price(food_item_name: str) -> str:
        """Search DuckDuckGo HTML for the market price of a food item in India (INR).
        
        Arguments:
        - food_item_name: The name of the food item to search.
        """
        import httpx
        import re
        
        query = f"price of {food_item_name} per kg in India INR"
        url = f"https://html.duckduckgo.com/html/?q={query.replace(' ', '+')}"
        
        headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        }
        
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(url, headers=headers, timeout=10.0)
                if response.status_code != 200:
                    return f"Error: DuckDuckGo returned status code {response.status_code}"
                
                # Extract snippets
                html_content = response.text
                snippets = re.findall(r'<a class="result__snippet"[^>]*>(.*?)</a>', html_content, re.DOTALL)
                
                def clean_html(text: str) -> str:
                    cleaned = re.sub(r'<[^>]+>', '', text)
                    entities = {
                        "&amp;": "&",
                        "&lt;": "<",
                        "&gt;": ">",
                        "&quot;": '"',
                        "&#x27;": "'",
                        "&#x2F;": "/",
                        "&nbsp;": " "
                    }
                    for ent, val in entities.items():
                        cleaned = cleaned.replace(ent, val)
                    return cleaned
                
                cleaned_snippets = [clean_html(s) for s in snippets[:5]]
                if not cleaned_snippets:
                    return "No results found on DuckDuckGo."
                
                return "\n\n".join(f"- {s}" for s in cleaned_snippets)
        except Exception as e:
            return f"Error occurred during search: {str(e)}"

    @tool
    async def save_raw_food_cost(food_pattern: str, cost_per_100g: float, price_currency: str = "INR") -> str:
        """Save or update the cost per 100g for a food pattern in the database.
        
        Arguments:
        - food_pattern: The name or pattern of the food (e.g. 'chicken', 'milk').
        - cost_per_100g: The price in INR (Rupees) per 100 grams of the food.
        - price_currency: The currency of the price (default 'INR').
        """
        async with pool.acquire() as conn:
            row = await conn.fetchrow(
                """
                INSERT INTO raw_food_costs (id, food_pattern, cost_per_100g, price_currency)
                VALUES (gen_random_uuid(), $1, $2, $3)
                ON CONFLICT (food_pattern) DO UPDATE
                SET cost_per_100g = EXCLUDED.cost_per_100g,
                    updated_at = now()
                RETURNING id
                """,
                food_pattern.lower().strip(),
                cost_per_100g,
                price_currency
            )
            raw_id = row["id"]
            
            # Auto-link food_items matching the pattern
            await conn.execute(
                """
                UPDATE food_items
                   SET raw_food_cost_id = $1
                 WHERE name ILIKE '%' || $2 || '%'
                   AND raw_food_cost_id IS NULL
                """,
                raw_id,
                food_pattern.lower().strip()
            )
            
        return f"Successfully saved cost for '{food_pattern}' as ₹{cost_per_100g:.2f}/100g and linked matching food items."

    return get_raw_food_costs, search_food_item_market_price, save_raw_food_cost


async def budget_node(state: GraphState) -> dict[str, BudgetResult]:
    """Evaluate candidate recipe costs and suggest cheaper substitutions if needed."""
    if "budget" not in state.get("required_agents", []):
        logger.info("budget_node_skipped")
        return {}

    logger.info("budget_node_started", user_id=state.get("user_id"))
    timer = Timer()

    pool = state["_pool"]
    settings = state["_settings"]
    callback = state.get("_step_callback")

    context = state.get("context", {})
    budget_limit = float(context.get("budget_limit", 0.0))
    candidate_recipes = context.get("candidate_recipes", [])

    recipe_costs: list[RecipeCost] = []
    substitutions: list[str] = []

    input_data = {
        "budget_limit": budget_limit,
        "candidate_recipes_count": len(candidate_recipes),
    }

    with timer:
        # Step 1: Identify food items with missing costs (estimated_cost_per_100g == 0.0)
        missing_names = []
        for recipe in candidate_recipes:
            recipe_id = recipe.get("recipe_id")
            if not recipe_id:
                continue
            recipe_detail = await get_recipe_with_food_items(pool, str(recipe_id))
            if not recipe_detail:
                continue
            for item in recipe_detail.get("food_items", []):
                cost = float(item.get("estimated_cost_per_100g") or 0.0)
                if cost == 0.0:
                    missing_names.append(item["food_item_name"])

        # De-duplicate missing names
        unique_missing = list(set(missing_names))

        # Step 2: If we have missing costs, run LLM to lookup and save them
        if unique_missing:
            logger.info("resolving_missing_food_costs", count=len(unique_missing), items=unique_missing)
            llm = get_chat_model(settings)
            get_raw_food_costs, search_food_item_market_price, save_raw_food_cost = make_tools(pool, settings)
            llm_with_tools = llm.bind_tools([get_raw_food_costs, search_food_item_market_price, save_raw_food_cost])

            messages = [
                {
                    "role": "system",
                    "content": (
                        "You are an expert budgeting and food item price resolution assistant for India.\n"
                        "Your goal is to find and record the cost in INR (Rupees) per 100g for a list of missing food items.\n"
                        "Rules:\n"
                        "1. First call `get_raw_food_costs` to see if a matching pattern already exists in the database.\n"
                        "2. If it does not exist, use `search_food_item_market_price` to fetch web search results for the price per kg in India.\n"
                        "3. Parse the search results to compute the cost per 100g (e.g. ₹60 per kg = ₹6 per 100g). Do not make assumptions or default fallbacks; always search if unknown.\n"
                        "4. Call `save_raw_food_cost` to save the resolved pattern and cost in the database. Use simple, generic patterns (e.g. 'chicken', 'paneer', 'spinach') rather than full recipe names.\n"
                        "5. Continue executing tool calls until you have resolved all items."
                    )
                },
                {
                    "role": "user",
                    "content": f"Please resolve the costs for these missing food items: {', '.join(unique_missing)}"
                }
            ]

            # Loop up to 5 steps to allow tool calls
            for _ in range(5):
                response = await llm_with_tools.ainvoke(messages)
                messages.append(response)

                if not response.tool_calls:
                    break

                for tc in response.tool_calls:
                    tool_name = tc["name"]
                    tool_args = tc["args"]
                    tool_call_id = tc["id"]

                    # Execute appropriate tool
                    if tool_name == "get_raw_food_costs":
                        result = await get_raw_food_costs.ainvoke(tool_args)
                    elif tool_name == "search_food_item_market_price":
                        result = await search_food_item_market_price.ainvoke(tool_args)
                    elif tool_name == "save_raw_food_cost":
                        result = await save_raw_food_cost.ainvoke(tool_args)
                    else:
                        result = f"Error: Tool '{tool_name}' not found."

                    messages.append({
                        "role": "tool",
                        "name": tool_name,
                        "tool_call_id": tool_call_id,
                        "content": str(result)
                    })

        # Step 3: Compute actual candidate recipe costs
        total_candidate_cost = 0.0
        for recipe in candidate_recipes:
            recipe_id = recipe.get("recipe_id")
            if not recipe_id:
                continue

            recipe_id_str = str(recipe_id)
            cost = await calculate_recipe_cost(pool, recipe_id_str)
            recipe_costs.append({
                "recipe_id": recipe_id_str,
                "title": recipe.get("title", "Unknown"),
                "cost": cost,
            })
            total_candidate_cost += cost

        # If average candidate cost per day is greater than budget, suggest substitutions
        avg_recipe_cost = (total_candidate_cost / len(candidate_recipes)) if candidate_recipes else 0.0
        within_budget = True
        if budget_limit > 0.0 and avg_recipe_cost > budget_limit:
            within_budget = False
            # Call LLM to suggest general cheap substitutions
            llm = get_chat_model(settings)
            prompt = (
                f"The user has a budget limit of {budget_limit} {context.get('budget_currency', 'INR')} per day. "
                f"The average recipe cost is {avg_recipe_cost:.2f}. "
                f"Suggest 3 simple, healthy, and cheap food item substitutions (e.g., replace quinoa with brown rice, or chicken breast with tofu/lentils) "
                f"to reduce the overall cost. Return a clean bulleted list."
            )
            try:
                response = await llm.ainvoke([
                    {"role": "system", "content": "You are a cost-optimization nutritional assistant."},
                    {"role": "user", "content": prompt}
                ])
                raw = response.content if hasattr(response, "content") else str(response)
                substitutions = [line.strip("- *") for line in raw.split("\n") if line.strip()]
            except Exception as exc:
                logger.error("budget_substitutions_failed", error=str(exc))
                substitutions = ["Fallback: Use local seasonal vegetables", "Fallback: Substitute imported grains with oats or rice"]

    output_data = {
        "recipe_costs": recipe_costs,
        "within_budget": within_budget,
        "substitutions": substitutions,
    }

    await emit_step(
        callback,
        agent_name="Budget",
        status="completed",
        step_type="aggregation",
        input_data=input_data,
        output_data=output_data,
        latency_ms=timer.ms,
    )

    return {
        "budget_result": {
            "within_budget": within_budget,
            "recipe_costs": recipe_costs,
            "substitutions": substitutions,
        }
    }
