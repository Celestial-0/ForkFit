"""Context models assembled before the LangGraph reasoning phase."""

from __future__ import annotations

from uuid import UUID

from pydantic import BaseModel, Field


class RetrievedRecipe(BaseModel):
    """A recipe returned from vector / keyword search."""

    recipe_id: UUID
    title: str
    cuisine: str | None = None
    dietary_tags: list[str] = Field(default_factory=list)
    similarity_score: float = 0.0


class RetrievedMemory(BaseModel):
    """A single agent memory relevant to the current session."""

    memory_id: UUID
    memory_type: str = Field(
        ...,
        description="Category: 'preference', 'restriction', or 'habit'.",
    )
    content: str
    confidence: float = 1.0
    importance: int = 5


class MacroTargets(BaseModel):
    """Daily macronutrient targets derived from user profile / goals."""

    calories: float
    protein_g: float
    carbs_g: float
    fat_g: float


class UserContextPackage(BaseModel):
    """Complete context assembled before reasoning begins.

    This is the knowledge package that flows into the LangGraph.  It merges
    profile data, medical safety info, preferences, pantry state, recent
    activity, and candidate recipes into a single envelope.
    """

    user_id: str

    # ── Medical / safety ──────────────────────────────────────────────────
    allergies: list[str] = Field(default_factory=list)
    medical_conditions: list[str] = Field(default_factory=list)
    is_pregnant: bool = False
    is_lactating: bool = False

    # ── Preferences ───────────────────────────────────────────────────────
    preferred_foods: list[str] = Field(default_factory=list)
    avoided_foods: list[str] = Field(default_factory=list)
    preferred_cuisine: str = ""

    # ── Nutritional targets ───────────────────────────────────────────────
    calorie_target: float = 2000.0
    macro_targets: MacroTargets | None = None
    budget_limit: float = 0.0
    budget_currency: str = "INR"

    # ── Biometrics ────────────────────────────────────────────────────────
    weight_kg: float | None = None
    height_cm: float | None = None
    age: int | None = None
    gender: str | None = None
    activity_level: str = "moderate"

    # ── Dynamic data ──────────────────────────────────────────────────────
    recent_memories: list[RetrievedMemory] = Field(default_factory=list)
    candidate_recipes: list[RetrievedRecipe] = Field(default_factory=list)
    pantry_items: list[dict] = Field(default_factory=list)
    recent_workouts: list[dict] = Field(default_factory=list)
    active_goals: list[dict] = Field(default_factory=list)
