# Bifrost - Product Requirements Document

## Overview

Bifrost is a multithreaded HTTP service written in Rust that searches and compares product data across multiple e-commerce sites.

A client submits a product query and can optionally provide a specific site list. Bifrost then fans out scrape jobs to all target sites concurrently, parses and normalizes each result, and returns a comparison showing where the product is cheapest, best-reviewed, in stock, and deliverable.

When no sites are specified, Bifrost searches a configurable set of default e-commerce sites.

## Goals

- Let users find a product once and instantly compare price, reviews, ratings, availability, and delivery info across multiple e-commerce sites.
- Provide a fast, concurrent multi-site scraping service behind a clean REST API.
- Demonstrate idiomatic Rust async patterns: `async/await`, channels, worker pools, error propagation across network boundaries.
- Practice production-grade concerns: rate limiting, retries, connection pooling, caching, structured logging, and metrics.

## Non-Goals

- Long-term persistent storage (database). Results live in an in-memory cache only.
- Browser-based rendering (no headless Chrome). Only static HTML pages are supported.
- User authentication or multi-tenancy.
- Deployment orchestration (Docker, Kubernetes).
- Affiliate link generation or purchase flow.

---

## Functional Requirements

### FR-1 - Product Search API

| Field | Detail |
|---|---|
| **Endpoint** | `POST /search` |
| **Request body** | JSON with `product` (required), optional `sites` array, optional `location`. |
| **Behavior** | If `sites` is omitted or empty, Bifrost uses the configured default sites. Each site is scraped concurrently for the given product. |
| **Response** | `202 Accepted` with a `job_id`, or `200 OK` with cached comparison data if a fresh result exists for the same query. |
| **Errors** | `400` for malformed input, `429` when globally rate-limited. |

Example request:

```json
{
  "product": "Sony WH-1000XM5",
  "sites": ["amazon.com", "bestbuy.com", "walmart.com"],
  "location": "New York, NY"
}
```

Example request using defaults:

```json
{
  "product": "Sony WH-1000XM5"
}
```

### FR-2 - Default Sites

- Bifrost ships with a configurable list of default e-commerce sites (e.g., Amazon, Best Buy, Walmart, Target, eBay).
- The list is defined in configuration and can be overridden via environment variable.
- When a request omits the `sites` field, all default sites are searched.

### FR-3 - Job Result Retrieval

| Field | Detail |
|---|---|
| **Endpoint** | `GET /search/:job_id` |
| **Response** | `200 OK` with the full comparison result when all sites have been scraped; `202 Accepted` with partial results while some sites are still pending; `500` if the entire job failed. |

### FR-4 - Comparative Response

The response groups results by site and includes a `comparison` summary that highlights:

- **cheapest** - the site with the lowest price.
- **most_expensive** - the site with the highest price.
- **best_rated** - the site where the product has the highest rating or most reviews.
- **available_at** - list of sites where the product is in stock.
- **deliverable_to** - list of sites that deliver to the user's location (if `location` was provided).
- **cheapest_delivery** - the site with the lowest delivery cost (if delivery info is available).

### FR-5 - Structured Product Data (per site)

Each site result returns a `SiteResult` containing:

- `site: String` - domain name of the source site.
- `product_name: String` - product title as listed on that site.
- `price: Option<f64>`
- `currency: Option<String>`
- `rating: Option<f64>`
- `review_count: Option<u32>`
- `availability: Option<String>` - e.g., "In Stock", "Out of Stock", "Pre-order".
- `delivery_available: Option<bool>` - whether the site delivers to the requested location.
- `delivery_cost: Option<f64>` - shipping cost, if available.
- `delivery_estimate: Option<String>` - e.g., "2-3 business days".
- `product_url: String` - direct link to the product page.
- `scraped_at: DateTime<Utc>`

### FR-6 - Worker Pool

- A configurable number of async worker tasks consume jobs from an internal channel.
- For a single search request, one job per target site is dispatched - all run concurrently.
- Workers use `reqwest` to fetch pages and `scraper` to parse HTML.
- Each worker runs on the tokio runtime and communicates results back via channels.

### FR-7 - Site-Specific Parsers

- Each supported site has a dedicated parser module that knows the HTML structure / CSS selectors for that site.
- Parsers implement a common trait (`SiteParser`) so new sites can be added without touching the worker or API layers.
- A fallback generic parser attempts extraction using common e-commerce markup patterns (schema.org, Open Graph, meta tags).

### FR-8 - Rate Limiting

- Per-domain rate limiting to avoid overwhelming target sites.
- Configurable requests-per-second per domain.
- Requests that exceed the limit are queued, not rejected.

### FR-9 - Retry Logic

- Retries on transient HTTP errors (5xx, timeouts, connection resets).
- Exponential backoff with jitter, configurable max retries (default: 3).
- A failed site does not block results from other sites - partial results are returned.

### FR-10 - In-Memory Cache

- Cache keyed by normalized query (product + sorted site list + location).
- TTL-based eviction (default: 15 minutes, configurable).
- A cache hit on a fresh entry skips the worker pool entirely and returns the full comparison result.

### FR-11 - Metrics Endpoint

| Field | Detail |
|---|---|
| **Endpoint** | `GET /metrics` |
| **Data** | Total searches, active jobs, cache hits/misses, error counts by site, average scrape latency per site, per-domain request counts. |

### FR-12 - Health Check

| Field | Detail |
|---|---|
| **Endpoint** | `GET /health` |
| **Response** | `200 OK` with `{ "status": "ok" }`. |

### FR-13 - List Default Sites

| Field | Detail |
|---|---|
| **Endpoint** | `GET /sites` |
| **Response** | `200 OK` with the list of default sites and their supported status. |

---

## Non-Functional Requirements

### NFR-1 - Performance

- Target p99 latency under 2 seconds for cached responses.
- Support at least 100 concurrent in-flight scrape jobs across all searches.
- Fan-out scrapes to all target sites in parallel; total latency is bounded by the slowest site (plus overhead), not the sum.

### NFR-2 - Partial Results

- If some sites succeed and others fail or time out, the API returns the successful results plus error details for the failed sites. It does not wait indefinitely or fail the entire request.

### NFR-3 - Observability

- Structured logging via `tracing` (JSON output in release, pretty-print in dev).
- Span-per-request and span-per-site with `trace_id` propagation.

### NFR-4 - Error Handling

- All public API errors return consistent JSON error bodies: `{ "error": "...", "detail": "..." }`.
- Per-site errors are reported inline in the response, not as HTTP-level failures.
- Internal panics are caught; the service stays up.

### NFR-5 - Configuration

All tunable values configurable via environment variables with sensible defaults:

| Variable | Default | Description |
|---|---|---|
| `PORT` | `3000` | Server listen port |
| `WORKER_COUNT` | `8` | Number of scraper worker tasks |
| `CACHE_TTL_SECS` | `900` | Cache time-to-live in seconds |
| `MAX_RETRIES` | `3` | Max retry attempts per request |
| `RATE_LIMIT_RPS` | `2` | Max requests per second per domain |
| `DEFAULT_SITES` | `amazon.com,bestbuy.com,walmart.com,target.com,ebay.com` | Comma-separated default site list |
| `REQUEST_TIMEOUT_SECS` | `10` | Per-site request timeout |

---

## Architecture

```
                ┌──────────────────┐
  POST /search  │                  │
  "product":    │    Axum API      │
  "Pixel 9"    ─┤    (router)      │
                │                  │
                └────────┬─────────┘
                         │
              ┌──────────┴──────────┐
              │                     │
        cache hit?             cache miss
              │                     │
        return cached        fan out: 1 job per site
          comparison         onto channel (mpsc)
                                    │
                  ┌─────────────────┼─────────────────┐
                  │                 │                  │
           ┌──────┴──────┐  ┌──────┴──────┐  ┌───────┴─────┐
           │  Worker W-1  │  │  Worker W-2  │  │  Worker W-N  │
           │  amazon.com  │  │ bestbuy.com  │  │  walmart.com │
           │  ┌─────────┐ │  │  ┌─────────┐│  │  ┌─────────┐│
           │  │ reqwest  │ │  │  │ reqwest  ││  │  │ reqwest  ││
           │  │ parser   │ │  │  │ parser   ││  │  │ parser   ││
           │  └────┬─────┘ │  │  └────┬─────┘│  │  └────┬─────┘│
           └───────┼───────┘  └───────┼──────┘  └───────┼──────┘
                   │                  │                  │
                   └──────────┬───────┘                  │
                              │      ┌───────────────────┘
                              ▼      ▼
                    ┌───────────────────┐
                    │  Result Aggregator │
                    │  - collect all     │
                    │  - build comparison│
                    │  - cache result    │
                    └───────────────────┘
```

## Tech Stack

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `axum` | HTTP server / routing |
| `reqwest` | HTTP client for fetching pages |
| `scraper` | HTML parsing and CSS selector queries |
| `tracing` / `tracing-subscriber` | Structured logging |
| `serde` / `serde_json` | Serialization |
| `chrono` | Timestamps |

## Core Concepts Demonstrated

- **Async/Await** - all I/O is non-blocking on the tokio runtime.
- **Concurrency via channels and worker pools** - `tokio::sync::mpsc` for job dispatch, `tokio::sync::oneshot` for per-job result delivery, fan-out pattern for multi-site search.
- **HTTP server + client** - axum serves the API; reqwest acts as the scraping client.
- **Error handling across network boundaries** - custom error types map cleanly to HTTP status codes; per-site errors don't fail the whole request.
- **Caching** - `DashMap` or `tokio::sync::RwLock<HashMap>` with TTL-based eviction.
- **Structured logging** - `tracing` spans and events for every request, per-site scrape, and aggregation step.
- **Benchmarking** - criterion benchmarks for parsing, aggregation, and cache operations.

---

## Milestones

1. **M1 - Skeleton** - Axum server boots, health and `/sites` endpoints respond, tracing configured.
2. **M2 - Worker Pool & Fan-out** - mpsc channel, N worker tasks, fan-out of one job per site.
3. **M3 - Site Parsers** - `SiteParser` trait, first parser (e.g., generic/schema.org), product field extraction.
4. **M4 - Result Aggregation** - Collect per-site results, build comparison summary (cheapest, best-rated, etc.).
5. **M5 - Delivery & Location** - Parse delivery cost/availability, filter by user location.
6. **M6 - Cache** - In-memory cache with TTL eviction; cache-hit path wired into API.
7. **M7 - Rate Limiting & Retries** - Per-domain rate limiter, exponential backoff, partial-result handling.
8. **M8 - Metrics & Observability** - `/metrics` endpoint, structured tracing spans per site.
9. **M9 - Polish** - Error responses, configuration, integration tests, benchmarks.
