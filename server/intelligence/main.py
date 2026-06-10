"""ForkFit Intelligence Service — main entrypoint.

Boots the gRPC server with a fully compiled LangGraph pipeline,
database connection pool, and all registered service handlers.
"""
from __future__ import annotations

import asyncio

import grpc
import structlog

from src.config import Settings
from src.db.pool import close_pool, create_pool
from src.generated import intelligence_pb2_grpc
from src.graph.orchestrator import build_graph
from src.server.servicer import IntelligenceServiceServicer

logger = structlog.get_logger()

_MAX_MESSAGE_BYTES = 50 * 1024 * 1024  # 50 MiB


async def serve() -> None:
    """Initialise all dependencies and start the gRPC server."""
    # ── Configuration ───────────────────────────────────────────────
    settings = Settings()
    logger.info(
        "settings_loaded",
        grpc_port=settings.grpc_port,
        llm_model=settings.llm_model,
    )

    # ── Database ────────────────────────────────────────────────────
    pool = await create_pool(settings.database_url)
    logger.info("db_pool_created")

    # ── Graph ───────────────────────────────────────────────────────
    graph = build_graph()
    logger.info("langgraph_compiled")

    # ── gRPC server ─────────────────────────────────────────────────
    server = grpc.aio.server(
        options=[
            ("grpc.max_send_message_length", _MAX_MESSAGE_BYTES),
            ("grpc.max_receive_message_length", _MAX_MESSAGE_BYTES),
        ],
    )

    servicer = IntelligenceServiceServicer(
        pool=pool,
        settings=settings,
        graph=graph,
    )
    intelligence_pb2_grpc.add_IntelligenceServiceServicer_to_server(
        servicer, server
    )

    listen_addr = f"{settings.grpc_host}:{settings.grpc_port}"
    server.add_insecure_port(listen_addr)

    logger.info("starting_grpc_server", address=listen_addr)
    await server.start()
    logger.info("grpc_server_started", address=listen_addr)

    try:
        await server.wait_for_termination()
    finally:
        logger.info("shutting_down")
        await close_pool(pool)
        logger.info("shutdown_complete")


if __name__ == "__main__":
    asyncio.run(serve())
