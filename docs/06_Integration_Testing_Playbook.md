# 06 - Integration Testing Playbook

## 6.1 Verification Matrix

The following table defines the expected system behavior under various conditions.

| Scenario             | HTTP Code       | Database Log                | Response Body                            |
| :------------------- | :-------------- | :-------------------------- | :--------------------------------------- |
| **Success**          | `200 OK`        | `status_code = 0`           | The guest's output                       |
| **Guest Not Found**  | `404 Not Found` | No log (pre-check fail)     | `{"error": "Guest 'x' not found"}`       |
| **Guest Panic**      | `500 Internal`  | `status_code = 101`         | `{"error": "Wasm exited with code 101"}` |
| **Database Offline** | `503 Service`   | No log (connection fail)    | `{"error": "Database unavailable"}`      |
| **Wasm Timeout**     | `504 Gateway`   | `error_message = "Timeout"` | `{"error": "Execution timed out"}`       |

## 6.2 Testing with `curl`

### Case 1: Standard Success Call

```bash
curl -X POST http://localhost:3000/execute/hello-world \
     -H "Content-Type: text/plain" \
     -d "Rust Developer"
```

**Expected Output:**

```text
Hello, Rust Developer! (Rendered by Wasmtime)
```

### Case 2: Missing Guest

```bash
curl -i -X POST http://localhost:3000/execute/does_not_exist
```

**Expected Response:**

```text
HTTP/1.1 404 Not Found
Content-Type: application/json
{"error": "Guest 'does_not_exist' not found on disk", "status": 404}
```

### Case 3: Empty Payload

```bash
curl -X POST http://localhost:3000/execute/hello-world -d ""
```

**Expected Output:**

```text
Hello, Guest! (Rendered by Wasmtime)
```

## 6.3 Database Inspection

Use the following `psql` command to verify that execution metrics are being correctly persisted.

```sql
SELECT
    id,
    function_name,
    status_code,
    duration_ms,
    LEFT(stdout_snippet, 50) AS output,
    created_at
FROM executions
ORDER BY created_at DESC
LIMIT 5;
```

## 6.4 Automated Integration Testing (Planned)

For CI/CD, a separate `tests/` folder should be created in the workspace root.

### Suggested Test Logic (Rust):

1. **Setup:**
   - Ensure `guest_hello_world` is compiled and moved to `guests_compiled/`.
   - Clear the `executions` table in the test database.
2. **Execute:**
   - Use `reqwest` to send a POST to the running Runner.
3. **Verify:**
   - Assert `200 OK`.
   - Assert body content.
   - Query Postgres and assert a new record exists with `status_code = 0`.
4. **Cleanup:**
   - Truncate the table.
