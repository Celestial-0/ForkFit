"""Nutrient breakdown model used across meals, daily totals, and plans."""

from __future__ import annotations

from pydantic import BaseModel


class NutrientBreakdown(BaseModel):
    """Complete nutrient profile for a meal or daily total.

    Provides the core macro- and micro-nutrient fields that are tracked
    throughout meal planning, validation, and scoring.
    """

    calories: float = 0.0
    protein_g: float = 0.0
    carbs_g: float = 0.0
    fat_g: float = 0.0
    fiber_g: float = 0.0
    sodium_mg: float = 0.0
