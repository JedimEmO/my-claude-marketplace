# RAS Observability Configuration

## Standard Setup (Quick Start)

```rust
use ras_observability_otel::standard_setup;
use axum::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let otel = standard_setup("my-service")?;

    let app = Router::new()
        .merge(service_router)
        .merge(otel.metrics_router()); // adds /metrics

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Builder Setup (Custom Configuration)

```rust
use ras_observability_otel::OtelSetupBuilder;
use prometheus::Registry;

// Custom Prometheus registry (optional)
let registry = Registry::new();
let custom_counter = prometheus::Counter::new("custom_ops", "Custom operations")?;
registry.register(Box::new(custom_counter.clone()))?;

let otel = OtelSetupBuilder::new("my-service")
    .with_prometheus_registry(registry)
    .build()?;
```

## Usage Tracker Wiring (REST)

```rust
use ras_observability_core::RequestContext;

let router = TaskServiceBuilder::new(service)
    .auth_provider(auth)
    .with_usage_tracker({
        let tracker = otel.usage_tracker();
        move |headers, user, method, path| {
            let context = RequestContext::rest(method, path);
            let tracker = tracker.clone();
            async move {
                tracker.track_request(&headers, user.as_ref(), &context).await;
            }
        }
    })
    .build();
```

## Duration Tracker Wiring (REST)

```rust
.with_method_duration_tracker({
    let tracker = otel.method_duration_tracker();
    move |method, path, user, duration| {
        let context = RequestContext::rest(method, path);
        let tracker = tracker.clone();
        async move {
            tracker.track_duration(&context, user.as_ref(), duration).await;
        }
    }
})
```

## Usage Tracker Wiring (JSON-RPC)

```rust
.with_usage_tracker({
    let tracker = otel.usage_tracker();
    move |headers, user, payload| {
        let context = RequestContext::jsonrpc(payload.method.clone());
        let tracker = tracker.clone();
        async move {
            tracker.track_request(&headers, user.as_ref(), &context).await;
        }
    }
})
```

## Duration Tracker Wiring (JSON-RPC)

```rust
.with_method_duration_tracker({
    let tracker = otel.method_duration_tracker();
    move |method, user, duration| {
        let context = RequestContext::jsonrpc(method.to_string());
        let tracker = tracker.clone();
        async move {
            tracker.track_duration(&context, user.as_ref(), duration).await;
        }
    }
})
```

## Request Metadata (Structured Logs, Not Metrics)

```rust
let context = RequestContext::rest("POST", "/api/orders")
    .with_metadata("request_id", request_id)
    .with_metadata("customer_id", customer_id);

// Metadata is included in structured logs but NOT in metric labels
otel.usage_tracker().track_request(&headers, user.as_ref(), &context).await;
```

## Protecting the Metrics Endpoint

```rust
use tower_http::auth::RequireAuthorizationLayer;

let app = Router::new()
    .merge(service_router)
    .nest(
        "/metrics",
        otel.metrics_router()
            .layer(RequireAuthorizationLayer::bearer("your-metrics-token")),
    );
```

## Prometheus Scrape Config

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'my-service'
    static_configs:
      - targets: ['my-service:3000']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Grafana Queries

```promql
# Request rate by method
rate(requests_completed[5m])

# Success rate (percentage)
sum(rate(requests_completed{success="true"}[5m]))
/ sum(rate(requests_completed[5m])) * 100

# P95 latency by method
histogram_quantile(0.95,
  sum(rate(method_duration_milliseconds_bucket[5m])) by (method, le)
)

# Error rate by protocol
sum(rate(requests_completed{success="false"}[5m])) by (protocol)

# Request volume by method (last hour)
increase(requests_started[1h])
```

## Histogram Buckets

Default duration buckets (seconds):
`0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0`

## Metric Labels

| Label | Values | Notes |
|-------|--------|-------|
| `method` | `"GET /users"`, `"createUser"` | Use path template, not resolved path |
| `protocol` | `"rest"`, `"jsonrpc"`, `"websocket"` | Set by `RequestContext` constructor |
| `success` | `"true"`, `"false"` | Only on `requests_completed` |

Keep labels low-cardinality. Never add per-user or per-request-ID labels — those belong in structured logs via `with_metadata()`.
