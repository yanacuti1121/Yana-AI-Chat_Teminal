# Phase 7 — Model Adapters

Phase 7 turns Gateway routing into a provider-neutral model protocol without giving models direct access to the host.

## Added

- Normalized chat messages, tool specifications, tool calls, usage, finish reasons, and stream events.
- Provider adapter contracts for discovery, completion, and streaming.
- Adapter configuration validation separated from model identity.
- Explicit capability metadata for chat, streaming, tools, vision, reasoning, and embeddings.
- SSE and NDJSON stream decoders for OpenAI-compatible and local-provider event shapes.
- Shared cancellation tokens and bounded exponential retry policy.
- Stable provider error classes with explicit retryability.

## Boundaries

- Adapters only communicate with model endpoints.
- Adapters cannot read or mutate the workspace.
- Tool calls returned by a model are data, not permission to execute.
- Every requested action must still pass Operator, Forge, Guard, Preview, and Approval.
- API keys are referenced by environment-variable name and must never be persisted in project state.

## Deferred

- Concrete HTTP client implementation.
- TLS policy and certificate configuration.
- Provider-specific authentication headers.
- Live model discovery against Ollama, LM Studio, llama.cpp, MLX, OpenAI, Anthropic, and Gemini endpoints.
- Backpressure-aware asynchronous streaming.
- Provider health checks and circuit breakers.
