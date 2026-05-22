# Project Documentation Index

This directory contains the technical reference documentation for the Serverless Runner project, version 2. This documentation covers the architecture, execution engine, fuel metering, and a comprehensive guest catalog.

## Table of Contents

1. [Index and Overview](00_Index_and_Overview.md)
2. [Architecture and Containerization](01_Architecture_and_Containerization.md)
3. [Fuel Metering and Endpoints](02_Fuel_Metering_and_Endpoints.md)
4. [Guest Catalog](03_Guest_Catalog.md)

---

## High-Level Overview: Modern Wasm Execution with Fuel

The Serverless Runner project implements a secure, sandboxed environment for executing WebAssembly (Wasm) modules. A critical challenge in serverless environments is preventing resource exhaustion by untrusted code (e.g., infinite loops).

This project utilizes **Fuel Metering** as the primary mechanism for resource control and billing.

### The Fuel Concept

Fuel is a monotonic counter representing the computational "budget" allocated to a single execution of a Wasm module. 

1. **Injection:** Before execution, the runner injects a specific amount of fuel into the Wasm store.
2. **Consumption:** As the Wasm instructions are executed by the `wasmtime` engine, fuel is consumed. Each instruction has a cost.
3. **Depletion:** If the fuel counter reaches zero, the execution is immediately trapped and terminated by the engine.
4. **Resumption:** In some advanced scenarios, fuel can be replenished, though this runner currently uses a fixed-budget per-request model.

### Key Benefits of Fuel Metering

- **Deterministic Termination:** Guarantees that no guest can run forever.
- **Granular Resource Tracking:** Provides a more precise measure of computation than wall-clock time, which can be affected by host load.
- **Security:** Prevents Denial of Service (DoS) attacks targeting CPU cycles.
- **Fair Billing:** Enables accurate chargeback based on actual instruction count.

By combining `wasmtime`'s async support with fuel metering, the runner achieves a high-performance, safe, and observable execution environment suitable for multi-tenant serverless workloads.
