# Adaptive Context Engine

The Adaptive Context Engine builds a bounded, deterministic input bundle before a request reaches the model gateway.

## Pipeline

```text
Intent
  -> Knowledge candidates
  -> Deterministic ranking
  -> Progressive expansion
  -> Budget enforcement
  -> Context bundle
  -> Gateway
```

## Guarantees

- No LLM call is used to rank or budget context.
- Candidate ordering is deterministic, including score ties.
- Duplicate sources are removed before output.
- Token, file-count, and byte limits are enforced together.
- Expansion proceeds through bounded batches instead of loading the entire workspace.
- Reserved output capacity is never consumed by the input bundle.

## Ranking inputs

The initial scoring model combines:

- entity relevance;
- architectural decisions;
- verified facts;
- workspace relevance;
- action receipts;
- timeline recency.

Decision and entity evidence receive the highest initial weights. These weights are explicit and testable rather than learned implicitly.

## Non-goals

This sprint does not:

- read files directly;
- mutate workspace state;
- call providers;
- summarize evidence with an LLM;
- learn ranking weights from user behavior.

Workspace learning and Project DNA remain separate later sprints so they cannot silently change the deterministic base ranking.
