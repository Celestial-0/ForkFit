"""Shared helpers used across all agent nodes.

Provides the step-emission callback wrapper and common utilities
so agent modules stay DRY.
"""
from __future__ import annotations

import json
import time
import uuid
from typing import Any, Callable

import structlog

logger = structlog.get_logger()


async def emit_step(
    callback: Callable | None,
    *,
    agent_name: str,
    status: str,
    step_type: str,
    input_data: dict[str, Any],
    output_data: dict[str, Any],
    latency_ms: int,
) -> None:
    """Emit an AgentStepUpdate via the gRPC streaming callback.

    If *callback* is ``None`` the call is silently skipped — this lets
    agent code call ``emit_step`` unconditionally.

    Args:
        callback: The ``_step_callback`` from GraphState (may be None).
        agent_name: Human-readable agent name (e.g. "Safety").
        status: Step status ("started", "completed", "failed").
        step_type: Category of work ("validation", "llm_call", "tool_call", "aggregation").
        input_data: Serializable dict summarising inputs to this step.
        output_data: Serializable dict summarising outputs from this step.
        latency_ms: Wall-clock time the step took, in milliseconds.
    """
    if callback is None:
        return

    payload = {
        "step_id": f"step-{uuid.uuid4().hex[:8]}",
        "agent_name": agent_name,
        "status": status,
        "step_type": step_type,
        "input_payload_json": json.dumps(input_data, default=str),
        "output_payload_json": json.dumps(output_data, default=str),
        "latency_ms": latency_ms,
    }
    try:
        await callback(payload)
    except Exception:
        logger.warning("step_callback_failed", agent_name=agent_name, exc_info=True)


class Timer:
    """Simple context-manager for measuring wall-clock milliseconds.

    Usage::

        t = Timer()
        with t:
            await do_work()
        print(t.ms)
    """

    def __init__(self) -> None:
        self._start: float = 0.0
        self.ms: int = 0

    def __enter__(self) -> Timer:
        self._start = time.perf_counter()
        return self

    def __exit__(self, *_: object) -> None:
        self.ms = int((time.perf_counter() - self._start) * 1000)
