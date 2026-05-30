# Project Structure

```
serverless-runner/
├── crates/
│   ├── serverless-core/
│   │   ├── migrations/
│   │   │   ├── 20260420133844_initial_schema.down.sql
│   │   │   └── 20260420133844_initial_schema.up.sql
│   │   ├── src/
│   │   │   ├── db/
│   │   │   │   ├── executions.rs
│   │   │   │   └── mod.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   └── models.rs
│   │   └── Cargo.toml
│   └── serverless-runner/
│       ├── src/
│       │   ├── api/
│       │   │   └── mod.rs
│       │   ├── engine/
│       │   │   ├── batcher.rs
│       │   │   └── mod.rs
│       │   ├── main.rs
│       │   └── state.rs
│       └── Cargo.toml
├── db/
│   └── schema.sql
├── docs/
│   ├── final/
│   │   ├── 00_Index.md
│   │   ├── 01_Executive_Summary.md
│   │   ├── 02_Concurrency_and_Compute.md
│   │   ├── 03_Data_Persistence_and_Identifiers.md
│   │   ├── 04_Resource_Management_and_Isolation.md
│   │   └── 05_Performance_Optimization_Report.md
│   ├── test_results/
│   │   ├── final_test_results.txt
│   │   ├── hello_test_results.txt
│   │   ├── live_test_results.txt
│   │   └── test_results.txt
│   ├── v2_reference/
│   │   ├── 00_Index_and_Overview.md
│   │   ├── 01_Architecture_and_Containerization.md
│   │   ├── 02_Fuel_Metering_and_Endpoints.md
│   │   └── 03_Guest_Catalog.md
│   ├── 01_Architecture_and_Data_Flow.md
│   ├── 02_Database_Schema_and_Lifecycle.md
│   ├── 03_Core_Crate_Deep_Dive.md
│   ├── 04_Runner_Engine_Technical_Spec.md
│   ├── 05_Guest_Development_Kit.md
│   ├── 06_Integration_Testing_Playbook.md
│   ├── 07_High_Throughput_Architecture_and_Benchmarks.md
│   └── 08_Ultra_High_Throughput_Optimization_Report.md
├── guests/
│   ├── env-reader/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── fibonacci/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── fs-reader/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── hello-world/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── infinite-loop/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── long-output-guest/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── memory-hog/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── net-guest/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── panic-guest/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── stdout-spammer/
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
├── tests/
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── GEMINI.md
├── HANDOFF.md
├── LICENSE
├── Project Structure.md
├── Roadmap.md
├── deploy-local.ps1
├── docker-compose.yml
├── k8s-manifests.yaml
├── kind-config.yaml
├── kubernetes-manifests.yaml
├── redeploy-cluster.ps1
├── run-benchmark.ps1
├── run-extensive-stress-test.ps1
├── run-stress-test.ps1
├── run-test.ps1
├── update-cluster.ps1
└── validate-all.ps1
```