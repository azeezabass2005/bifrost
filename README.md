# Bifrost

A multithreaded HTTP service in Rust for searching and comparing product data across multiple e-commerce sites.

Send a product name and Bifrost returns a side-by-side view of price, ratings, reviews, availability, and delivery details, including highlights such as cheapest option, best-reviewed listing, and deliverability to your location.

## How It Works

1. A client sends a `POST /search` request with a product name and (optionally) a list of sites and a location.
2. If no sites are specified, Bifrost searches a configurable set of **default e-commerce sites** (Amazon, Best Buy, Walmart, Target, eBay).
3. The API checks the in-memory cache - if a fresh result exists for this query, it returns immediately.
4. Otherwise, one scrape job per target site is fanned out to a pool of async workers via an mpsc channel. All sites are scraped concurrently.
5. Each worker fetches the page with `reqwest`, runs the site-specific parser (built on `scraper`), and extracts structured product data including delivery info.
6. A result aggregator collects all site results, builds a comparison summary, caches it, and returns the response.

Rate limiting, retries with exponential backoff, and connection pooling are handled transparently by the worker layer. If some sites fail or time out, results from the successful sites are still returned.

## Stack

| Crate | Role |
|---|---|
| **tokio** | Async runtime |
| **axum** | HTTP server and routing |
| **reqwest** | HTTP client for page fetching |
| **scraper** | HTML parsing with CSS selectors |
| **tracing** | Structured logging |
| **serde** | JSON serialization |
| **chrono** | Timestamps |

## Getting Started

### Prerequisites

- Rust 2024 edition (1.85+)

### Build and Run

```bash
cargo build
cargo run
```

The server starts on `http://localhost:3000` by default.

### Configuration

All settings are read from environment variables:

| Variable | Default | Description |
|---|---|---|
| `PORT` | `3000` | Server listen port |
| `WORKER_COUNT` | `8` | Number of scraper worker tasks |
| `CACHE_TTL_SECS` | `900` | Cache time-to-live in seconds |
| `MAX_RETRIES` | `3` | Max retry attempts per request |
| `RATE_LIMIT_RPS` | `2` | Max requests per second per domain |
| `DEFAULT_SITES` | `amazon.com,bestbuy.com,walmart.com,target.com,ebay.com` | Comma-separated default site list |
| `REQUEST_TIMEOUT_SECS` | `10` | Per-site request timeout |

## API

### Health Check

```
GET /health
```

Returns `200 OK` with `{ "status": "ok" }`.

### List Default Sites

```
GET /sites
```

Returns the configured default e-commerce sites.

```json
{
  "sites": ["amazon.com", "bestbuy.com", "walmart.com", "target.com", "ebay.com"]
}
```

### Search for a Product

```
POST /search
Content-Type: application/json

{
  "product": "Sony WH-1000XM5",
  "sites": ["amazon.com", "bestbuy.com", "walmart.com"],
  "location": "New York, NY"
}
```

Both `sites` and `location` are optional. Omit `sites` to search all defaults:

```json
{
  "product": "Sony WH-1000XM5"
}
```

**Responses:**
- `200 OK`: cached comparison returned immediately.
- `202 Accepted`: job queued; poll with the returned `job_id`.
- `400 Bad Request`: malformed input.
- `429 Too Many Requests`: rate limit exceeded.

### Get Search Result

```
GET /search/:job_id
```

**Responses:**
- `200 OK`: all sites scraped, full comparison returned.
- `202 Accepted`: still in progress (partial results included).
- `500 Internal Server Error`: entire job failed.

### Response Shape

```json
{
  "product": "Sony WH-1000XM5",
  "results": [
    {
      "site": "amazon.com",
      "product_name": "Sony WH-1000XM5 Wireless Noise Cancelling Headphones",
      "price": 298.00,
      "currency": "USD",
      "rating": 4.6,
      "review_count": 12847,
      "availability": "In Stock",
      "delivery_available": true,
      "delivery_cost": 0.00,
      "delivery_estimate": "Tomorrow",
      "product_url": "https://www.amazon.com/dp/B0BX2L8PZG",
      "scraped_at": "2026-03-16T14:30:00Z"
    },
    {
      "site": "bestbuy.com",
      "product_name": "Sony WH-1000XM5 Headphones",
      "price": 329.99,
      "currency": "USD",
      "rating": 4.7,
      "review_count": 3421,
      "availability": "In Stock",
      "delivery_available": true,
      "delivery_cost": 5.99,
      "delivery_estimate": "2-3 business days",
      "product_url": "https://www.bestbuy.com/site/sony-wh1000xm5/123456.p",
      "scraped_at": "2026-03-16T14:30:01Z"
    }
  ],
  "comparison": {
    "cheapest": {
      "site": "amazon.com",
      "price": 298.00
    },
    "most_expensive": {
      "site": "bestbuy.com",
      "price": 329.99
    },
    "best_rated": {
      "site": "bestbuy.com",
      "rating": 4.7,
      "review_count": 3421
    },
    "most_reviews": {
      "site": "amazon.com",
      "review_count": 12847
    },
    "available_at": ["amazon.com", "bestbuy.com"],
    "deliverable_to": ["amazon.com", "bestbuy.com"],
    "cheapest_delivery": {
      "site": "amazon.com",
      "delivery_cost": 0.00
    }
  },
  "errors": []
}
```

When a site fails, it appears in `errors` instead of `results`:

```json
{
  "errors": [
    {
      "site": "walmart.com",
      "error": "timeout",
      "detail": "Request timed out after 10s"
    }
  ]
}
```

### Metrics

```
GET /metrics
```

Returns throughput, error rates per site, cache hit/miss counts, active job count, and average scrape latency per site.

## Project Structure

```
src/
  main.rs               - entry point, server bootstrap
  config.rs             - environment-based configuration, default sites
  error.rs              - unified error types
  models.rs             - Product, SiteResult, Comparison, request/response types
  api/
    mod.rs              - axum routes and handlers
  worker/
    mod.rs              - scraper worker pool, fan-out job dispatch
  parsers/
    mod.rs              - parser trait definition and parser registry
    amazon.rs           - Amazon parser
    bestbuy.rs          - Best Buy parser
    generic.rs          - fallback parser (schema.org, Open Graph, meta tags)
  aggregator/
    mod.rs              - collects site results and builds comparison summary
  cache/
    mod.rs              - in-memory TTL cache
  rate_limiter/
    mod.rs              - per-domain rate limiting
  metrics/
    mod.rs              - counters and metrics collection
```

## Core Concepts

- **Async/Await** - all I/O is non-blocking on the tokio runtime.
- **Concurrency** - `tokio::sync::mpsc` for job dispatch, `tokio::sync::oneshot` for per-job results, fan-out pattern to scrape all sites in parallel.
- **HTTP server + client** - axum serves the API; reqwest fetches target pages.
- **Error handling** - per-site errors are isolated; one failing site never blocks results from others.
- **Caching** - in-memory cache with TTL-based eviction, keyed by normalized query.
- **Structured logging** - `tracing` spans per request and per site for observability.
- **Benchmarking** - criterion benchmarks for parsing, aggregation, and cache performance.

## Running Tests

```bash
cargo test
```

## License

MIT
