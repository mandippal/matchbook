# Monitoring Guide

This guide explains the monitoring infrastructure for Matchbook using Prometheus and Grafana.

## Overview

Matchbook uses:
- **Prometheus** for metrics collection and alerting
- **Grafana** for visualization and dashboards

## Quick Start (Local Development)

```bash
# Start all services including monitoring
docker-compose up -d

# Access Grafana
open http://localhost:3000
# Default credentials: admin/admin

# Access Prometheus
open http://localhost:9092
```

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Indexer   │     │     API     │     │    Crank    │
│   :9090     │     │   :8080     │     │   :9091     │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       │    /metrics       │    /metrics       │    /metrics
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │  Prometheus │
                    │    :9092    │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │   Grafana   │
                    │    :3000    │
                    └─────────────┘
```

## Metrics

### Indexer Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `indexer_slot_lag` | Gauge | Slots behind the tip |
| `indexer_events_processed_total` | Counter | Total events processed |
| `indexer_accounts_processed_total` | Counter | Total accounts processed |
| `indexer_parse_errors_total` | Counter | Parse errors by type |

### API Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `api_requests_total` | Counter | Total requests by method, path, status |
| `api_request_duration_seconds` | Histogram | Request latency |
| `api_active_connections` | Gauge | Current HTTP connections |
| `ws_active_connections` | Gauge | Current WebSocket connections |
| `ws_messages_sent_total` | Counter | WebSocket messages sent |
| `ws_subscriptions` | Gauge | Active subscriptions |

### Crank Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `crank_matches_executed_total` | Counter | Total matches executed |
| `crank_transactions_total` | Counter | Transactions by status |
| `crank_profit_lamports` | Gauge | Current profit |
| `crank_priority_fee_lamports` | Gauge | Current priority fee |
| `crank_fees_paid_lamports` | Counter | Total fees paid |

### Market Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `market_volume_total` | Counter | Trading volume by market |
| `market_trades_total` | Counter | Trade count by market |
| `market_order_count` | Gauge | Active orders by market/side |
| `market_spread_bps` | Gauge | Spread in basis points |

## Dashboards

### Overview Dashboard
- Service health status
- Slot lag
- Request rate
- Latency percentiles
- Active connections
- Crank matches

### Indexer Dashboard
- Slot lag over time
- Processing throughput
- Parse errors by type

### API Dashboard
- Request rate by endpoint
- Latency percentiles (p50, p95, p99)
- Error rate
- Requests by status code
- Active connections

### Crank Dashboard
- Matches over time
- Transaction success rate
- Profitability
- Transactions by status

### Market Dashboard
- 24h volume
- 24h trades
- Spread
- Order book depth
- Volume over time

## Alerts

### Critical Alerts

| Alert | Condition | Description |
|-------|-----------|-------------|
| `ServiceDown` | `up == 0` for 1m | Service not responding |
| `IndexerSlotLagCritical` | `slot_lag > 50` for 1m | Indexer severely behind |
| `APIHighErrorRate` | `error_rate > 1%` for 5m | High API error rate |

### Warning Alerts

| Alert | Condition | Description |
|-------|-----------|-------------|
| `IndexerSlotLagHigh` | `slot_lag > 10` for 2m | Indexer falling behind |
| `APIHighLatency` | `p99 > 500ms` for 5m | High API latency |
| `CrankHighFailureRate` | `failure_rate > 10%` for 5m | Crank transactions failing |

## Adding New Metrics

### Rust Implementation

```rust
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MY_COUNTER: Counter = register_counter!(
        "my_counter_total",
        "Description of my counter"
    ).unwrap();
    
    pub static ref MY_HISTOGRAM: Histogram = register_histogram!(
        "my_duration_seconds",
        "Description of my histogram"
    ).unwrap();
    
    pub static ref MY_GAUGE: Gauge = register_gauge!(
        "my_gauge",
        "Description of my gauge"
    ).unwrap();
}

// Usage
MY_COUNTER.inc();
MY_HISTOGRAM.observe(duration.as_secs_f64());
MY_GAUGE.set(value);
```

### Metric Naming Conventions

- Use snake_case
- Include unit suffix: `_seconds`, `_bytes`, `_total`
- Counters should end with `_total`
- Use labels for dimensions: `{method="GET", path="/v1/markets"}`

## Prometheus Configuration

### Scrape Configuration

```yaml
scrape_configs:
  - job_name: my-service
    static_configs:
      - targets:
          - my-service:9090
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        regex: (.+):.*
        replacement: $1
```

### Recording Rules

Pre-compute expensive queries:

```yaml
groups:
  - name: my-rules
    rules:
      - record: my:metric:5m
        expr: sum(rate(my_counter_total[5m]))
```

### Alert Rules

```yaml
groups:
  - name: my-alerts
    rules:
      - alert: MyAlert
        expr: my_metric > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "My alert fired"
          description: "Value is {{ $value }}"
```

## Kubernetes Deployment

### Using ServiceMonitor (Prometheus Operator)

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: my-service
spec:
  selector:
    matchLabels:
      app: my-service
  endpoints:
    - port: metrics
      interval: 15s
```

### Pod Annotations

```yaml
metadata:
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
    prometheus.io/path: "/metrics"
```

## Troubleshooting

### Prometheus Not Scraping

1. Check target status in Prometheus UI (`/targets`)
2. Verify network connectivity
3. Check service annotations/labels

### Missing Metrics

1. Verify metrics endpoint: `curl http://service:port/metrics`
2. Check metric registration in code
3. Verify scrape config matches service

### Dashboard Not Loading

1. Check Grafana datasource configuration
2. Verify Prometheus is accessible from Grafana
3. Check dashboard JSON syntax

## Related Documentation

- [Docker Guide](./docker.md)
- [Kubernetes Guide](../k8s/README.md)
- [Operations Guide](../.internalDoc/08-operations.md)
