# Project Structure

```
serverless-runner/
├── crates/
│   ├── serverless-core/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   └── serverless-runner/
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
├── docs/
│   ├── 01_Architecture_and_Data_Flow.md
│   ├── 02_Database_Schema_and_Lifecycle.md
│   ├── 03_Core_Crate_Deep_Dive.md
│   ├── 04_Runner_Engine_Technical_Spec.md
│   ├── 05_Guest_Development_Kit.md
│   └── 06_Integration_Testing_Playbook.md
├── guests/
│   ├── hello-world/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── Cargo.toml
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── Project Structure.md
├── Roadmap.md
└── docker-compose.yml
```