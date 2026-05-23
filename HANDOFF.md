# Handoff: Serverless Runner Platform Implementation

## Goal
To provide a high-performance, sandboxed serverless execution platform using Rust, Axum, Wasmtime (WASI), and PostgreSQL.
**Long-term Goal:** Achieve 1000 RPS locally with 0.0% dropped requests and consistent performance.

## Current State
- **Workspace:** Cargo Workspace with `serverless-runner`, `serverless-core`, `tests`, and guest functions.
- **Infrastructure (K8s):** Fully containerized stack deployed via Kind. Includes NGINX Ingress, Axum Runners (3 replicas), PgBouncer (sharded), and PostgreSQL (multi-shard).
- **Core Logic:** Multi-pool database sharding with `sqlx`. Log-and-Update pattern for execution metrics.
- **Runner Engine:** Wasmtime (WASI) orchestration with fuel-based CPU limits and 64MB memory caps per guest.
- **Automation:** 
    - `redeploy-cluster.ps1`: Rapid full-stack redeployment with readiness synchronization.
    - `validate-all.ps1`: Comprehensive validation loop (Pod health -> DB connectivity -> API Sanity -> Stress Test).
- **Stability:** Resolved PgBouncer statement caching conflicts and tuned Kubernetes probes for robust startup.

## Files Actively Involved
- `k8s-manifests.yaml`: Kubernetes resource definitions (Deployments, Services, ConfigMaps, Secrets, HPA, Ingress).
- `validate-all.ps1` & `redeploy-cluster.ps1`: Primary automation and validation scripts.
- `crates/serverless-runner/src/main.rs`: Multi-shard pool initialization and API routing.
- `crates/serverless-core/src/db/executions.rs`: Shard-aware execution logging.

## Investigation History & Learnings
- **PgBouncer & SQLx:** PgBouncer in `transaction` mode is incompatible with SQLx's default prepared statement caching. Fixed by adding `statement_cache_capacity=0` to connection strings.
- **K8s Startup Race:** Initial liveness/readiness failures were due to runners trying to connect to databases before PgBouncer or Postgres were fully initialized. Increased `initialDelaySeconds` and added `failureThreshold` to stabilize.
- **Kind Ingress:** Required specific `extraPortMappings` and `node-labels` in `kind-config.yaml` to route host traffic through the NGINX ingress controller.

## Next Steps
1. **Metrics Integration:** Deploy `metrics-server` to the cluster to enable HPA and `kubectl top`.
2. **Performance Tuning:** Optimize thread counts, connection pools, and Wasm pre-compilation to reach the 1000 RPS target.
3. **Distribution Tuning:** Monitor DB shard distribution under heavy load and adjust sharding logic if hot-spotting occurs.
