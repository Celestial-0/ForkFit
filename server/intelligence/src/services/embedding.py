"""Embedding service using Ollama's local embedding API.

Provides async functions to generate vector embeddings from text
using the qwen3-embedding:4b model via Ollama's /api/embed endpoint.
"""
from __future__ import annotations

import httpx
import structlog

logger = structlog.get_logger()

_DEFAULT_BASE_URL = "http://localhost:11434"
_DEFAULT_MODEL = "qwen3-embedding:4b"


async def embed_text(
    text: str,
    *,
    base_url: str = _DEFAULT_BASE_URL,
    model: str = _DEFAULT_MODEL,
) -> list[float]:
    """Generate a single embedding vector from text using Ollama.

    Args:
        text: The input text to embed.
        base_url: Ollama API base URL.
        model: Embedding model identifier.

    Returns:
        A list of floats representing the embedding vector.

    Raises:
        httpx.HTTPStatusError: If the Ollama API returns a non-2xx response.
    """
    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            f"{base_url}/api/embed",
            json={"model": model, "input": text},
        )
        response.raise_for_status()
        data = response.json()
        # Ollama returns {"model": ..., "embeddings": [[...]]}
        embeddings: list[list[float]] = data["embeddings"]
        logger.debug("embed_text_complete", model=model, vector_dim=len(embeddings[0]))
        return embeddings[0]


async def embed_texts(
    texts: list[str],
    *,
    base_url: str = _DEFAULT_BASE_URL,
    model: str = _DEFAULT_MODEL,
) -> list[list[float]]:
    """Batch-embed multiple texts in a single Ollama API call.

    Args:
        texts: List of input texts to embed.
        base_url: Ollama API base URL.
        model: Embedding model identifier.

    Returns:
        A list of embedding vectors, one per input text.

    Raises:
        httpx.HTTPStatusError: If the Ollama API returns a non-2xx response.
    """
    async with httpx.AsyncClient(timeout=60.0) as client:
        response = await client.post(
            f"{base_url}/api/embed",
            json={"model": model, "input": texts},
        )
        response.raise_for_status()
        data = response.json()
        embeddings: list[list[float]] = data["embeddings"]
        logger.debug(
            "embed_texts_complete",
            model=model,
            count=len(embeddings),
            vector_dim=len(embeddings[0]) if embeddings else 0,
        )
        return embeddings
