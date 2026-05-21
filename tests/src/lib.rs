#[cfg(test)]
mod tests {
    use reqwest::blocking::Client;
    use std::time::Duration;

    #[test]
    fn test_hello_world_execution() {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let response = client
            .post("http://localhost:8080/execute/hello-world")
            .body("TDD Runner")
            .send()
            .expect("Failed to send request");

        assert!(response.status().is_success());
        let text = response.text().unwrap();
        assert!(text.contains("Hello, TDD Runner!"));
    }

    #[test]
    fn test_guest_not_found() {
        let client = Client::new();
        let response = client
            .post("http://localhost:8080/execute/does_not_exist")
            .send()
            .expect("Failed to send request");

        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_fibonacci_execution() {
        let client = Client::new();
        let response = client
            .post("http://localhost:8080/execute/fibonacci?number=12")
            .send()
            .expect("Failed to send request");

        assert!(response.status().is_success());
        let text = response.text().unwrap();
        assert_eq!(text, "144");
    }
}
