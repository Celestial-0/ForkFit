"""Embedding service using Ollama's local embedding API.

Provides async functions to generate vector embeddings from text
using the qwen3-embedding:4b model via Ollama's /api/embed endpoint.
"""
from __future__ import annotations

from typing import TYPE_CHECKING

import httpx
import structlog

if TYPE_CHECKING:
    from src.config import Settings

logger = structlog.get_logger()

_DEFAULT_BASE_URL = "http://localhost:11434"
_DEFAULT_MODEL = "qwen3-embedding:4b"


async def embed_text(
    text: str,
    settings: Settings | None = None,
    *,
    base_url: str | None = None,
    model: str | None = None,
) -> list[float]:
    """Generate a single embedding vector from text using Ollama.

    Args:
        text: The input text to embed.
        settings: Optional configuration settings containing ollama URL, model, and dimensions.
        base_url: Ollama API base URL (overrides settings).
        model: Embedding model identifier (overrides settings).

    Returns:
        A list of floats representing the embedding vector.

    Raises:
        httpx.HTTPStatusError: If the Ollama API returns a non-2xx response.
    """
    url = base_url or (settings.ollama_base_url if settings else _DEFAULT_BASE_URL)
    embedding_model = model or (settings.embedding_model if settings else _DEFAULT_MODEL)

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            f"{url}/api/embed",
            json={"model": embedding_model, "input": text},
        )
        response.raise_for_status()
        data = response.json()
        # Ollama returns {"model": ..., "embeddings": [[...]]}
        embeddings: list[list[float]] = data["embeddings"]
        vector = embeddings[0]

        # Truncate according to settings dimensions (MRL support)
        dimensions = settings.embedding_dimensions if settings else None
        if dimensions and len(vector) > dimensions:
            vector = vector[:dimensions]

        logger.debug("embed_text_complete", model=embedding_model, vector_dim=len(vector))
        return vector


async def embed_texts(
    texts: list[str],
    settings: Settings | None = None,
    *,
    base_url: str | None = None,
    model: str | None = None,
) -> list[list[float]]:
    """Batch-embed multiple texts in a single Ollama API call.

    Args:
        texts: List of input texts to embed.
        settings: Optional configuration settings containing ollama URL, model, and dimensions.
        base_url: Ollama API base URL (overrides settings).
        model: Embedding model identifier (overrides settings).

    Returns:
        A list of embedding vectors, one per input text.

    Raises:
        httpx.HTTPStatusError: If the Ollama API returns a non-2xx response.
    """
    url = base_url or (settings.ollama_base_url if settings else _DEFAULT_BASE_URL)
    embedding_model = model or (settings.embedding_model if settings else _DEFAULT_MODEL)

    async with httpx.AsyncClient(timeout=60.0) as client:
        response = await client.post(
            f"{url}/api/embed",
            json={"model": embedding_model, "input": texts},
        )
        response.raise_for_status()
        data = response.json()
        embeddings: list[list[float]] = data["embeddings"]

        dimensions = settings.embedding_dimensions if settings else None
        if dimensions and embeddings:
            embeddings = [v[:dimensions] if len(v) > dimensions else v for v in embeddings]

        logger.debug(
            "embed_texts_complete",
            model=embedding_model,
            count=len(embeddings),
            vector_dim=len(embeddings[0]) if embeddings else 0,
        )
        return embeddings

