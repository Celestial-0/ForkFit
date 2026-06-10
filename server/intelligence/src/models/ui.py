"""UI element specifications mapped to proto UIElement messages."""

from __future__ import annotations

import json

from pydantic import BaseModel, Field


class UIElementSpec(BaseModel):
    """Maps directly to the proto ``UIElement`` message.

    The ``config`` dict holds layout / styling parameters while ``data``
    carries the payload (chart series, recipe fields, etc.).  The two
    ``*_json`` properties serialise them for the gRPC transport layer.
    """

    type: str = Field(
        ...,
        description="Widget kind: 'chart', 'timeline', 'recipe_card', 'dashboard'.",
    )
    title: str
    config: dict = Field(default_factory=dict)
    data: dict = Field(default_factory=dict)

    @property
    def config_json(self) -> str:
        """Serialise *config* to a JSON string for gRPC."""
        return json.dumps(self.config)

    @property
    def data_json(self) -> str:
        """Serialise *data* to a JSON string for gRPC."""
        return json.dumps(self.data)
