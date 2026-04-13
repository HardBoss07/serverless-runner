# 02 - Database Schema and Lifecycle

## 2.1 Full Schema Definition (DDL)

The database must be initialized using the following script.

```sql
-- Initial migration
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    function_name VARCHAR(255) NOT NULL,
    status_code INTEGER,
    stdout_snippet TEXT,
    duration_ms BIGINT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexing for observability and dashboard performance
CREATE INDEX idx_executions_function_name ON executions(function_name);
CREATE INDEX idx_executions_created_at ON executions(created_at DESC);
```

### Column Specifications

| Column           | Rust Type        | SQL Type      | Description                                             |
| :--------------- | :--------------- | :------------ | :------------------------------------------------------ |
| `id`             | `uuid::Uuid`     | `UUID`        | Primary identifier for the execution.                   |
| `function_name`  | `String`         | `VARCHAR`     | Corresponds to the .wasm filename on disk.              |
| `status_code`    | `Option<i32>`    | `INTEGER`     | Exit code from WASI (0 for success, 1-255 for failure). |
| `stdout_snippet` | `Option<String>` | `TEXT`        | First 2048 characters of the guest output.              |
| `duration_ms`    | `Option<i64>`    | `BIGINT`      | Total processing time (wall clock).                     |
| `error_message`  | `Option<String>` | `TEXT`        | Any engine-level or platform errors.                    |
| `created_at`     | `DateTime<Utc>`  | `TIMESTAMPTZ` | Timestamp of execution receipt.                         |

## 2.2 Connection Management (sqlx)

The application will use `sqlx::postgres::PgPoolOptions` for robust connection pooling.

### Recommended Configuration (Local/Dev)

| Setting           | Recommended Value          | Rationale                                     |
| :---------------- | :------------------------- | :-------------------------------------------- |
| `max_connections` | `20`                       | Sufficient for local development and testing. |
| `acquire_timeout` | `Duration::from_secs(5)`   | Quick fail if the DB is under heavy load.     |
| `min_connections` | `2`                        | Avoids a completely cold pool.                |
| `idle_timeout`    | `Duration::from_secs(600)` | Prunes unused connections after 10m.          |

## 2.3 Persistence Lifecycle

Wasm executions are non-atomic with respect to the database. We follow a "Log-and-Update" pattern.

1. **The Entry Call:**
   - Function: `log_execution_start`.
   - Action: `INSERT` with `function_name` and `created_at`.
   - Result: Returns a `UUID` to be used for the duration of the request.
2. **The Result Call:**
   - Function: `complete_execution`.
   - Action: `UPDATE` by `id`.
   - Payload: Sets `status_code`, `stdout_snippet`, and `duration_ms`.
3. **The Failure Call:**
   - Function: `log_execution_error`.
   - Action: `UPDATE` by `id`.
   - Payload: Sets `error_message`.

### Why separate calls?

Using a single transaction for the entire Wasm call is an anti-pattern. Wasm execution can be slow (e.g., several seconds), and holding a database transaction open during this time would exhaust the connection pool. We use separate, atomic updates.
