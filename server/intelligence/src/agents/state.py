"""LangGraph execution state — the single source of truth.

Every agent node reads from and writes to this TypedDict. Runtime
dependencies (DB pool, settings, streaming callback) are injected
at graph entry and carried through the state.
"""
from __future__ import annotations

from typing import Any, Callable, TypedDict
import asyncpg
from src.config import Settings


# ── Safety Types ─────────────────────────────────────────────
class BlockedRecipe(TypedDict):
    """Details of a candidate recipe blocked by safety constraints."""
    recipe_id: str
    title: str
    reason: str


class SafetyResult(TypedDict):
    """Result payload from the Safety Agent."""
    safe_recipe_ids: list[str]
    blocked_recipes: list[BlockedRecipe]
    warnings: list[str]


# ── Nutrition Types ──────────────────────────────────────────
class RecipeNutritionScore(TypedDict):
    """Nutrition adherence score and breakdown for a recipe."""
    recipe_id: str
    score: float
    breakdown: dict[str, float]


class NutritionResult(TypedDict):
    """Result payload from the Nutrition Agent."""
    daily_targets: dict[str, float]
    recipe_nutrition_scores: list[RecipeNutritionScore]


# ── Budget Types ─────────────────────────────────────────────
class RecipeCost(TypedDict):
    """Cost details of a recipe."""
    recipe_id: str
    title: str
    cost: float


class BudgetResult(TypedDict):
    """Result payload from the Budget Agent."""
    within_budget: bool
    recipe_costs: list[RecipeCost]
    substitutions: list[str]


# ── Culture Types ────────────────────────────────────────────
class RejectedRecipe(TypedDict):
    """Details of a candidate recipe rejected by cultural/dietary filters."""
    recipe_id: str
    title: str
    reason: str


class CultureResult(TypedDict):
    """Result payload from the Culture Agent."""
    aligned_recipe_ids: list[str]
    rejected: list[RejectedRecipe]


# ── Recipe Types ─────────────────────────────────────────────
class RecipeFoodItem(TypedDict):
    """Food item info within SelectedRecipe."""
    name: str
    quantity: float
    unit: str


class SelectedRecipe(TypedDict):
    """Full recipe details of a selected meal plan recipe."""
    recipe_id: str
    title: str
    cuisine: str | None
    servings: float
    nutrition: dict[str, float]
    cost: float
    instructions: list[str]
    food_items: list[RecipeFoodItem]


class RecipeResult(TypedDict):
    """Result payload from the Recipe Agent."""
    selected_recipes: list[SelectedRecipe]
    alternatives: list[str]


# ── Calendar Types ───────────────────────────────────────────
class ScheduledMeal(TypedDict):
    """A meal scheduled in a calendar slot."""
    meal_type: str
    recipe_id: str | None
    recipe_title: str
    servings: float
    nutrition: dict[str, float]
    cost: float


class DailySchedulePlan(TypedDict):
    """A daily list of scheduled meals and workout notes."""
    date: str
    meals: list[ScheduledMeal]
    notes: str


class CalendarResult(TypedDict):
    """Result payload from the Calendar Agent."""
    daily_plans: list[DailySchedulePlan]


# ── Shopping Types ───────────────────────────────────────────
class ShoppingItem(TypedDict):
    """An aggregated shopping list item with deductions applied."""
    food_item_name: str
    quantity: float
    unit: str
    category: str
    estimated_cost: float


class ShoppingResult(TypedDict):
    """Result payload from the Shopping Agent."""
    items: list[ShoppingItem]
    total_cost: float
    pantry_savings: float


# ── Consensus Types ──────────────────────────────────────────
class MergedPlan(TypedDict):
    """The unified, conflict-resolved meal plan."""
    daily_plans: list[DailySchedulePlan]
    shopping_list: list[ShoppingItem]
    total_cost: float
    nutrition_summary: dict[str, float]
    confidence: float


# ── Judge Types ──────────────────────────────────────────────
class JudgeVerdict(TypedDict):
    """Quality and rule validation verdict from the Judge."""
    passed: bool
    confidence: float
    failures: list[str]
    replan_instructions: str | None


# ── Visualization Types ──────────────────────────────────────
class UIElementSpecDict(TypedDict):
    """Type contract for UI elements generated for the frontend."""
    type: str
    title: str
    config_json: str
    data_json: str


# ── Graph State ──────────────────────────────────────────────
class GraphState(TypedDict, total=False):
    """State flowing through the entire LangGraph agent pipeline.

    Fields are grouped by lifecycle stage: input, planner output,
    individual agent outputs, consensus/judge, visualization, and
    runtime dependencies.
    """

    # ── Input (set at graph entry) ──────────────────────────────
    user_id: str
    prompt: str
    context: dict                       # Serialized UserContextPackage
    chat_history: list[dict]

    # ── Planner output ──────────────────────────────────────────
    required_agents: list[str]
    execution_plan: str

    # ── Agent outputs ───────────────────────────────────────────
    safety_result: SafetyResult | None
    nutrition_result: NutritionResult | None
    budget_result: BudgetResult | None
    culture_result: CultureResult | None
    recipe_result: RecipeResult | None
    calendar_result: CalendarResult | None
    shopping_result: ShoppingResult | None

    # ── Consensus & Judge ───────────────────────────────────────
    merged_plan: MergedPlan | None
    judge_verdict: JudgeVerdict | None
    replan_count: int

    # ── Visualization output ────────────────────────────────────
    ui_elements: list[UIElementSpecDict]
    final_text: str

    # ── Runtime dependencies (injected, not serialized) ─────────
    _pool: asyncpg.Pool
    _settings: Settings
    _step_callback: Callable | None     # For gRPC step updates streaming
    _token_callback: Callable | None    # For gRPC text token deltas streaming
