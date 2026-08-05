# Phase 7 — Real Providers

Phase 7 connects the normalized model protocol to real HTTP endpoints without weakening Yana's workspace safety boundary.

## Included

- Bounded blocking HTTP transport using rustls
- Explicit timeout and response-size limits
- Environment-variable API key lookup
- OpenAI-compatible model discovery, completion, tools, and SSE streaming
- Ollama model discovery, completion, and NDJSON streaming
- LM Studio preset through the OpenAI-compatible adapter
- Provider error normalization for authentication, rate limits, timeout, and availability

## Data flow

```text
Knowledge Engine
  -> ModelRequest
  -> ProviderAdapter
  -> HttpTransport
  -> Provider endpoint
  -> normalized ModelResponse / StreamEvent
```

Tool calls returned by a provider remain untrusted data. They do not execute until Operator, Forge, Guard, Preview, and human approval authorize the action.

## Security constraints

- API key values are read from environment variables and are never written to project state.
- Custom headers are validated before requests.
- HTTP responses and streams have hard byte limits.
- Local defaults bind to loopback endpoints.
- Provider adapters have no direct filesystem or shell access.

## Deferred

- Anthropic native Messages adapter
- Gemini native adapter
- async transport and backpressure
- provider health checks and circuit breakers
- automatic endpoint discovery
- per-provider TLS pinning
