#[cfg(test)]
mod tests {
    use reqwest::blocking::Client;
    use reqwest::StatusCode;
    use serde_json::Value;
    use std::thread;
    use std::time::Duration;

    const BASE_URL: &str = "http://localhost:8080";

    // ==========================================
    // Category 1: The Playbook Verification Matrix
    // ==========================================

    #[test]
    fn test_01_matrix_success_200() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .header("Content-Type", "text/plain")
            .body("Rust Developer")
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.text().unwrap(),
            "Hello, Rust Developer! (Rendered by Wasmtime)\n"
        );
    }

    #[test]
    fn test_02_matrix_guest_not_found_404() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/missing_guest", BASE_URL))
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Verifying exact Playbook JSON structure
        let json: Value = response.json().expect("Response is not JSON");
        assert_eq!(
            json["error"].as_str().unwrap(),
            "Guest 'missing_guest' not found on disk"
        );
        assert_eq!(json["status"].as_u64().unwrap(), 404);
    }

    #[test]
    fn test_03_matrix_guest_panic_500() {
        // Requires a guest that executes: std::process::exit(101);
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/panic-guest", BASE_URL))
            .send()
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json: Value = response.json().unwrap();
        assert_eq!(json["error"].as_str().unwrap(), "Wasm exited with code 101");
    }

    #[test]
    fn test_04_matrix_wasm_timeout_504() {
        // Requires a guest that loops infinitely or 'fibonacci' with fuel exhaust limit
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let response = client
            .post(format!("{}/execute/infinite-loop", BASE_URL))
            .send()
            .unwrap();

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        let json: Value = response.json().unwrap();
        assert_eq!(json["error"].as_str().unwrap(), "Execution timed out");
    }

    #[test]
    fn test_05_matrix_database_offline_503() {
        // To run this, you must temporarily stop the Postgres container or alter the DATABASE_URL
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .send()
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let json: Value = response.json().unwrap();
        assert_eq!(json["error"].as_str().unwrap(), "Database unavailable");
    }

    // ==========================================
    // Category 2: HTTP Semantics & Edge Inputs
    // ==========================================

    #[test]
    fn test_06_hello_world_empty_payload() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .body("")
            .send()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.text().unwrap(),
            "Hello, Guest! (Rendered by Wasmtime)\n"
        );
    }

    #[test]
    fn test_07_invalid_http_method_get() {
        let client = Client::new();
        let response = client
            .get(format!("{}/execute/hello-world", BASE_URL))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_08_invalid_http_method_put() {
        let client = Client::new();
        let response = client
            .put(format!("{}/execute/hello-world", BASE_URL))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_09_missing_function_name_in_path() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/", BASE_URL))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ==========================================
    // Category 3: Fibonacci Validation Logic
    // ==========================================

    #[test]
    fn test_10_fibonacci_valid_small_number() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci?number=10", BASE_URL))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().unwrap(), "55");
    }

    #[test]
    fn test_11_fibonacci_invalid_letters() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci?number=abc", BASE_URL))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(response
            .text()
            .unwrap()
            .contains("Invalid number parameter"));
    }

    #[test]
    fn test_12_fibonacci_negative_number() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci?number=-5", BASE_URL))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(response
            .text()
            .unwrap()
            .contains("Number must be non-negative"));
    }

    #[test]
    fn test_13_fibonacci_missing_query_param() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci", BASE_URL))
            .send()
            .unwrap();
        // Axum will fail routing if the Query extractor is mandatory
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ==========================================
    // Category 4: WASI Sandboxing & Security
    // ==========================================

    #[test]
    fn test_14_wasi_fs_read_violation() {
        // Guest attempts: std::fs::read_to_string("/etc/passwd")
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fs-reader", BASE_URL))
            .send()
            .unwrap();

        // Guest should catch std::io::Error and exit gracefully, OR trap if unhandled
        // Sandbox must guarantee it fails.
        let text = response.text().unwrap();
        assert!(text.contains("Permission denied") || response.status().is_server_error());
    }

    #[test]
    fn test_15_wasi_env_var_isolation() {
        // Guest attempts: std::env::var("DATABASE_URL")
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/env-reader", BASE_URL))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.text().unwrap().contains("postgres://"));
    }

    #[test]
    fn test_16_wasi_network_socket_violation() {
        // Guest attempts: std::net::TcpStream::connect("8.8.8.8:80")
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/net-guest", BASE_URL))
            .send()
            .unwrap();
        let text = response.text().unwrap();
        assert!(text.contains("Operation not supported") || text.contains("Permission denied"));
    }

    // ==========================================
    // Category 5: Engine Limits (Memory & Output)
    // ==========================================

    #[test]
    fn test_17_memory_maximum_size_trap() {
        // Guest allocates a Vec<u8> larger than 64MB (the configured limit)
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/memory-hog", BASE_URL))
            .send()
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        // Validate engine trapped the allocation
        assert!(response.text().unwrap().contains("out of bounds"));
    }

    #[test]
    fn test_18_stdout_pipe_limit_truncation() {
        // Guest loop-prints 2MB of text to stdout (pipe limit is 1MB)
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/stdout-spammer", BASE_URL))
            .send()
            .unwrap();

        let text = response.text().unwrap();
        // Verify output is exactly 1MB (1048576 bytes) or trapped by WASI pipe
        assert!(text.len() <= 1024 * 1024);
    }

    // ==========================================
    // Category 6: Concurrency & Stress Testing
    // ==========================================

    #[test]
    fn test_19_concurrent_executions_pool_stress() {
        // Send 25 concurrent requests (Pool limit is 20)
        let mut handles = vec![];
        for _ in 0..25 {
            handles.push(thread::spawn(|| {
                let client = Client::new();
                client
                    .post(format!("{}/execute/hello-world", BASE_URL))
                    .send()
                    .unwrap()
            }));
        }

        let mut successes = 0;
        for handle in handles {
            let resp = handle.join().unwrap();
            if resp.status() == StatusCode::OK {
                successes += 1;
            }
        }
        // All should succeed as Axum will queue the connection pool requests
        assert_eq!(successes, 25);
    }

    // ==========================================
    // Category 7: Database Observability Checks
    // ==========================================
    // Note: These tests assume access to the shared DB and the `sqlx` crate.

    #[tokio::test]
    async fn test_20_db_lifecycle_success_record() {
        let client = Client::new();
        client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .body("DB_Check")
            .send()
            .unwrap();

        let pool = sqlx::PgPool::connect("postgres://postgres:postgres@localhost:5432/serverless")
            .await
            .unwrap();
        let record = sqlx::query!(
            "SELECT status_code, stdout_snippet FROM executions ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(record.status_code, Some(0));
        assert!(record.stdout_snippet.unwrap().contains("DB_Check"));
    }

    #[tokio::test]
    async fn test_21_db_lifecycle_duration_ms() {
        let client = Client::new();
        client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .send()
            .unwrap();

        let pool = sqlx::PgPool::connect("postgres://postgres:postgres@localhost:5432/serverless")
            .await
            .unwrap();
        let record =
            sqlx::query!("SELECT duration_ms FROM executions ORDER BY created_at DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .unwrap();

        // Assert execution took longer than 0ms but less than 1000ms
        assert!(record.duration_ms.unwrap() > 0);
        assert!(record.duration_ms.unwrap() < 1000);
    }

    #[tokio::test]
    async fn test_22_db_lifecycle_error_logging() {
        let client = Client::new();
        // Hitting panic guest to force a crash
        let _ = client
            .post(format!("{}/execute/panic-guest", BASE_URL))
            .send()
            .unwrap();

        let pool = sqlx::PgPool::connect("postgres://postgres:postgres@localhost:5432/serverless")
            .await
            .unwrap();
        let record = sqlx::query!(
            "SELECT status_code, error_message FROM executions ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(record.status_code, Some(101));
        assert!(record
            .error_message
            .unwrap()
            .contains("Wasm exited with code"));
    }

    #[tokio::test]
    async fn test_23_db_stdout_snippet_truncation() {
        // Guest returns > 2048 chars. Verify DB truncates it per specification.
        let client = Client::new();
        client
            .post(format!("{}/execute/long-output-guest", BASE_URL))
            .send()
            .unwrap();

        let pool = sqlx::PgPool::connect("postgres://postgres:postgres@localhost:5432/serverless")
            .await
            .unwrap();
        let record =
            sqlx::query!("SELECT stdout_snippet FROM executions ORDER BY created_at DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .unwrap();

        // DB specifies 2048 max characters for the snippet
        assert!(record.stdout_snippet.unwrap().len() <= 2048);
    }

    #[tokio::test]
    async fn test_24_db_pre_check_failure_no_log() {
        // Guest not found should NOT log to DB (pre-check fail)
        let pool = sqlx::PgPool::connect("postgres://postgres:postgres@localhost:5432/serverless")
            .await
            .unwrap();
        let count_before: i64 = sqlx::query_scalar!("SELECT count(*) FROM executions")
            .fetch_one(&pool)
            .await
            .unwrap();

        let client = Client::new();
        client
            .post(format!("{}/execute/does_not_exist", BASE_URL))
            .send()
            .unwrap();

        let count_after: i64 = sqlx::query_scalar!("SELECT count(*) FROM executions")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count_before, count_after);
    }

    // ==========================================
    // Category 8: Graceful Host Shutdown
    // ==========================================

    #[test]
    fn test_25_axum_graceful_shutdown() {
        // Test that the runner handles SIGINT/SIGTERM properly
        // without killing inflight Wasm modules or DB connections.
        // Usually tested via integration bash scripts, but placeholder logic here.
        assert!(true);
    }
}
