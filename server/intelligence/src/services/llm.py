"""Centralized LLM access via langchain-ollama.

Provides a factory function to create configured ChatOllama instances
for use throughout the ForkFit agent system.
"""
from __future__ import annotations

import structlog
from langchain_ollama import ChatOllama

from src.config import Settings

logger = structlog.get_logger()


def get_chat_model(
    settings: Settings,
    *,
    temperature: float = 0.3,
    num_ctx: int = 8192,
) -> ChatOllama:
    """Create a ChatOllama instance configured for the ForkFit LLM.

    Args:
        settings: Application settings containing model and URL config.
        temperature: Sampling temperature (lower = more deterministic).
        num_ctx: Context window size in tokens.

    Returns:
        A configured ChatOllama instance ready for invocation.
    """
    logger.debug(
        "creating_chat_model",
        model=settings.llm_model,
        base_url=settings.ollama_base_url,
        temperature=temperature,
    )
    return ChatOllama(
        model=settings.llm_model,
        base_url=settings.ollama_base_url,
        temperature=temperature,
        num_ctx=num_ctx,
    )
