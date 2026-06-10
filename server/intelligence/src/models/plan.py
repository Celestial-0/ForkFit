"""Meal-plan models: slots, daily plans, and weekly plans."""

from __future__ import annotations

from datetime import date
from uuid import UUID

from pydantic import BaseModel, Field

from src.models.nutrition import NutrientBreakdown


class MealSlot(BaseModel):
    """A single meal within a daily plan."""

    meal_type: str = Field(
        ...,
        description="One of 'breakfast', 'lunch', 'dinner', 'snack'.",
    )
    recipe_id: UUID | None = None
    recipe_title: str = ""
    servings: float = 1.0
    nutrition: NutrientBreakdown = Field(default_factory=NutrientBreakdown)
    cost: float = 0.0
    instructions: list[str] = Field(default_factory=list)
    prep_time_minutes: int | None = None


class DailyPlan(BaseModel):
    """All meals for a single calendar day."""

    date: date
    meals: list[MealSlot] = Field(default_factory=list)
    daily_totals: NutrientBreakdown = Field(default_factory=NutrientBreakdown)
    daily_cost: float = 0.0


class WeeklyPlan(BaseModel):
    """Seven-day meal plan with aggregated totals."""

    start_date: date
    end_date: date
    days: list[DailyPlan] = Field(default_factory=list)
    weekly_totals: NutrientBreakdown = Field(default_factory=NutrientBreakdown)
    weekly_cost: float = 0.0
