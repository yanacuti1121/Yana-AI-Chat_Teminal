# Phase 3 — Gateway Ecosystem

Phase 3 connects Yana to models and environments without coupling the core to any single provider.

## Naming boundary

- **Gateway** routes model requests by capability.
- **Bridge** connects Yana to terminal, desktop, IDE, and coding-agent environments.
- **Resource** determines whether a local model fits the current machine.
- **Conductor** assigns roles and tasks through Gateway routes.

## Provider routing

Providers describe capabilities instead of relying on provider-name conditionals:

- chat
- streaming
- tool calling
- vision
- embeddings
- reasoning
- image generation

A route may prefer local execution while still requiring an exact capability set.

## Local-first rule

Local is a routing preference, not an assumption. A local provider is selected only when it satisfies the task requirements and the Resource plan reports a viable fit.

## Safety boundary

Phase 3 does not grant models direct filesystem or shell access. All actions still pass through Operator, Forge, Guard, Lens, and Journal.

## Initial transports

- OpenAI-compatible
- Anthropic-compatible
- Ollama
- llama.cpp
- MLX
- embedded model runtime

Transport support and provider identity remain separate so compatible servers can be added without changing the core.
