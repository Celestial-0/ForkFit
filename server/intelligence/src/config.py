"""Application settings loaded from environment variables and .env file."""

from __future__ import annotations

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Central configuration for the ForkFit Intelligence Service.

    Values are loaded from environment variables (uppercase) with an optional
    ``.env`` file fallback.  Every field maps 1-to-1 with a ``DATABASE_URL``,
    ``REDIS_URL``, etc. environment variable.
    """

    # ── Database ──────────────────────────────────────────────────────────
    database_url: str
    """PostgreSQL connection string, e.g. ``postgresql://user:pass@host/db``."""

    redis_url: str = "redis://localhost:6379"
    """Redis connection string for caching / pub-sub."""

    # ── LLM / Embedding ──────────────────────────────────────────────────
    ollama_base_url: str = "http://localhost:11434"
    """Base URL of the Ollama inference server."""

    llm_model: str = "gemma4"
    """Name of the chat / reasoning model served by Ollama."""

    embedding_model: str = "qwen3-embedding:4b"
    """Name of the embedding model served by Ollama."""

    embedding_dimensions: int = 1536
    """Dimensionality of embedding vectors — must match ``vector(1536)`` in DB."""

    # ── gRPC ──────────────────────────────────────────────────────────────
    grpc_host: str = "0.0.0.0"
    """Network interface the gRPC server binds to."""

    grpc_port: int = 50051
    """Port the gRPC server listens on."""

    # ── Agent behaviour ───────────────────────────────────────────────────
    max_replan_attempts: int = 2
    """Maximum number of judge → planner replan loops before giving up."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
    )
