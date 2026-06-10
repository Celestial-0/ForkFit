"""Intent models extracted from user natural-language prompts."""

from __future__ import annotations

from pydantic import BaseModel, Field


class IntentBlueprint(BaseModel):
    """Structured intent extracted from the user's natural language prompt.

    The planner agent parses a free-form request into this schema so
    downstream nodes (retriever, reasoner, etc.) can work with typed data.
    """

    goal: str = Field(
        ...,
        description="High-level nutritional goal, e.g. 'muscle_gain', 'weight_loss'.",
    )
    diet: str = Field(
        ...,
        description="Dietary preference, e.g. 'vegetarian', 'vegan', 'omnivore'.",
    )
    budget_limit: float = Field(
        default=0.0,
        description="Maximum spend for the plan period (0 = no limit).",
    )
    budget_currency: str = Field(
        default="INR",
        description="ISO-4217 currency code for the budget.",
    )
    timeline: str = Field(
        default="weekly",
        description="Plan granularity — 'daily' or 'weekly'.",
    )
    constraints: list[str] = Field(
        default_factory=list,
        description="Extra constraints, e.g. ['no eggs', 'high protein'].",
    )
    raw_analysis: dict = Field(
        default_factory=dict,
        description="Full LLM analysis output for traceability.",
    )

    @property
    def raw_analysis_json(self) -> str:
        import json
        return json.dumps(self.raw_analysis)
