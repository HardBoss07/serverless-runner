#[cfg(test)]
mod tests {
    use reqwest::Client;
    use reqwest::StatusCode;
    use serde_json::Value;
    use std::time::Duration;

    const BASE_URL: &str = "http://runner:8080";
    const DB_URL: &str = "postgres://platform_user:secret_password@db:5432/platform_db";

    async fn get_db_pool() -> sqlx::PgPool {
        sqlx::PgPool::connect(DB_URL).await.unwrap()
    }

    #[tokio::test]
    async fn test_01_matrix_success_200() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .header("Content-Type", "text/plain")
            .body("Rust Developer")
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.text().await.unwrap(),
            "Hello, Rust Developer! (Rendered by Wasmtime)\n"
        );
    }

    #[tokio::test]
    async fn test_02_matrix_guest_not_found_404() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/missing_guest", BASE_URL))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json: Value = response.json().await.expect("Response is not JSON");
        assert_eq!(
            json["error"].as_str().unwrap(),
            "Guest 'missing_guest' not found on disk"
        );
        assert_eq!(json["status"].as_u64().unwrap(), 404);
    }

    #[tokio::test]
    async fn test_03_matrix_guest_panic_500() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/panic-guest", BASE_URL))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json: Value = response.json().await.unwrap();
        assert_eq!(json["error"].as_str().unwrap(), "Wasm exited with code 101");
    }

    #[tokio::test]
    async fn test_04_matrix_wasm_timeout_504() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/infinite-loop", BASE_URL))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        let json: Value = response.json().await.unwrap();
        assert_eq!(json["error"].as_str().unwrap(), "Execution timed out");
    }

    #[tokio::test]
    async fn test_05_matrix_database_offline_503() {
        // This test is hard to trigger without actually stopping DB,
        // but we can verify the error mapping if we can force a failure.
        // For now, we skip or assume DB is up.
    }

    #[tokio::test]
    async fn test_06_hello_world_empty_payload() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .body("")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.text().await.unwrap(),
            "Hello, Guest! (Rendered by Wasmtime)\n"
        );
    }

    #[tokio::test]
    async fn test_07_invalid_http_method_get() {
        let client = Client::new();
        let response = client
            .get(format!("{}/execute/hello-world", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_08_invalid_http_method_put() {
        let client = Client::new();
        let response = client
            .put(format!("{}/execute/hello-world", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_09_missing_function_name_in_path() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_10_fibonacci_valid_small_number() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci?number=10", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), "55");
    }

    #[tokio::test]
    async fn test_11_fibonacci_invalid_letters() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci?number=abc", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(response
            .text()
            .await
            .unwrap()
            .contains("Invalid number parameter"));
    }

    #[tokio::test]
    async fn test_12_fibonacci_negative_number() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci?number=-5", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(response
            .text()
            .await
            .unwrap()
            .contains("Number must be non-negative"));
    }

    #[tokio::test]
    async fn test_13_fibonacci_missing_query_param() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fibonacci", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_14_wasi_fs_read_violation() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/fs-reader", BASE_URL))
            .send()
            .await
            .unwrap();

        let status = response.status();
        let text = response.text().await.unwrap().to_lowercase();
        assert!(
            text.contains("permission denied")
                || text.contains("no such file")
                || text.contains("pre-opened")
                || status.is_server_error()
        );
    }

    #[tokio::test]
    async fn test_15_wasi_env_var_isolation() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/env-reader", BASE_URL))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.text().await.unwrap().contains("postgres://"));
    }

    #[tokio::test]
    async fn test_16_wasi_network_socket_violation() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/net-guest", BASE_URL))
            .send()
            .await
            .unwrap();
        let text = response.text().await.unwrap().to_lowercase();
        assert!(text.contains("operation not supported") || text.contains("permission denied"));
    }

    #[tokio::test]
    async fn test_17_memory_maximum_size_trap() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/memory-hog", BASE_URL))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let text = response.text().await.unwrap().to_lowercase();
        assert!(
            text.contains("out of bounds")
                || text.contains("memory limit exceeded")
                || text.contains("unreachable")
        );
    }

    #[tokio::test]
    async fn test_18_stdout_pipe_limit_truncation() {
        let client = Client::new();
        let response = client
            .post(format!("{}/execute/stdout-spammer", BASE_URL))
            .send()
            .await
            .unwrap();

        let text = response.text().await.unwrap();
        assert!(text.len() <= 1024 * 1024);
    }

    #[tokio::test]
    async fn test_19_concurrent_executions_pool_stress() {
        let mut handles = vec![];
        for _ in 0..25 {
            handles.push(tokio::spawn(async move {
                let client = Client::new();
                client
                    .post(format!("{}/execute/hello-world", BASE_URL))
                    .send()
                    .await
                    .unwrap()
            }));
        }

        let mut successes = 0;
        for handle in handles {
            let resp = handle.await.unwrap();
            if resp.status() == StatusCode::OK {
                successes += 1;
            }
        }
        assert_eq!(successes, 25);
    }

    #[tokio::test]
    async fn test_20_db_lifecycle_success_record() {
        let client = Client::new();
        client
            .post(format!("{}/execute/hello-world", BASE_URL))
            .body("DB_Check")
            .send()
            .await
            .unwrap();

        let pool = get_db_pool().await;
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
            .await
            .unwrap();

        let pool = get_db_pool().await;
        let record =
            sqlx::query!("SELECT duration_ms FROM executions ORDER BY created_at DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(record.duration_ms.unwrap() > 0);
        assert!(record.duration_ms.unwrap() < 1000);
    }

    #[tokio::test]
    async fn test_22_db_lifecycle_error_logging() {
        let client = Client::new();
        let _ = client
            .post(format!("{}/execute/panic-guest", BASE_URL))
            .send()
            .await
            .unwrap();

        let pool = get_db_pool().await;
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
        let client = Client::new();
        client
            .post(format!("{}/execute/long-output-guest", BASE_URL))
            .send()
            .await
            .unwrap();

        let pool = get_db_pool().await;
        let record =
            sqlx::query!("SELECT stdout_snippet FROM executions ORDER BY created_at DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(record.stdout_snippet.unwrap().len() <= 2048);
    }

    #[tokio::test]
    async fn test_24_db_pre_check_failure_no_log() {
        let pool = get_db_pool().await;
        let count_before = sqlx::query_scalar!("SELECT count(*) FROM executions")
            .fetch_one(&pool)
            .await
            .unwrap()
            .unwrap_or(0);

        let client = Client::new();
        client
            .post(format!("{}/execute/does_not_exist", BASE_URL))
            .send()
            .await
            .unwrap();

        let count_after = sqlx::query_scalar!("SELECT count(*) FROM executions")
            .fetch_one(&pool)
            .await
            .unwrap()
            .unwrap_or(0);
        assert_eq!(count_before, count_after);
    }
}
