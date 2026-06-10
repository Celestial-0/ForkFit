"""Data models for the ForkFit Intelligence Service."""

from __future__ import annotations

from src.models.context import (
    MacroTargets,
    RetrievedMemory,
    RetrievedRecipe,
    UserContextPackage,
)
from src.models.intent import IntentBlueprint
from src.models.nutrition import NutrientBreakdown
from src.models.plan import DailyPlan, MealSlot, WeeklyPlan
from src.models.ui import UIElementSpec

__all__ = [
    "DailyPlan",
    "IntentBlueprint",
    "MacroTargets",
    "MealSlot",
    "NutrientBreakdown",
    "RetrievedMemory",
    "RetrievedRecipe",
    "UIElementSpec",
    "UserContextPackage",
    "WeeklyPlan",
]
