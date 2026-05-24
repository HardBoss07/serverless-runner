use serverless_core::AppResult;
use sqlx::PgPool;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub enum DbMessage {
    Execution {
        id: Uuid,
        function_name: String,
        status_code: i32,
        stdout_snippet: String,
        duration_ms: i64,
        error_message: Option<String>,
        shard_index: usize,
    },
}

pub struct Batcher {
    tx: mpsc::Sender<DbMessage>,
}

impl Batcher {
    pub fn new(pools: Vec<PgPool>, batch_size: usize, flush_interval: Duration) -> Self {
        let (tx, rx) = mpsc::channel(100_000);

        let pools_clone = pools.clone();
        tokio::spawn(async move {
            run_batcher(pools_clone, rx, batch_size, flush_interval).await;
        });

        Self { tx }
    }

    pub async fn send(&self, msg: DbMessage) -> AppResult<()> {
        self.tx.send(msg).await.map_err(|e| {
            serverless_core::AppError::WasmEngine(format!("Failed to send to batcher: {}", e))
        })
    }
}

async fn run_batcher(
    pools: Vec<PgPool>,
    mut rx: mpsc::Receiver<DbMessage>,
    batch_size: usize,
    flush_interval: Duration,
) {
    let mut batches: Vec<Vec<DbMessage>> = (0..pools.len())
        .map(|_| Vec::with_capacity(batch_size))
        .collect();
    let mut last_flush = Instant::now();

    loop {
        let timeout = tokio::time::sleep(Duration::from_millis(50));
        tokio::pin!(timeout);

        tokio::select! {
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    let shard_index = match &msg {
                        DbMessage::Execution { shard_index, .. } => *shard_index,
                    };
                    batches[shard_index].push(msg);
                    if batches[shard_index].len() >= batch_size {
                        flush_batch(&pools[shard_index], &mut batches[shard_index]).await;
                    }
                } else {
                    // Channel closed, flush all and exit
                    for i in 0..pools.len() {
                        if !batches[i].is_empty() {
                            flush_batch(&pools[i], &mut batches[i]).await;
                        }
                    }
                    break;
                }
            }
            _ = &mut timeout => {
                if last_flush.elapsed() >= flush_interval {
                    for i in 0..pools.len() {
                        if !batches[i].is_empty() {
                            flush_batch(&pools[i], &mut batches[i]).await;
                        }
                    }
                    last_flush = Instant::now();
                }
            }
        }
    }
}

async fn flush_batch(pool: &PgPool, batch: &mut Vec<DbMessage>) {
    if batch.is_empty() {
        return;
    }

    let mut ids = Vec::new();
    let mut function_names = Vec::new();
    let mut status_codes = Vec::new();
    let mut stdout_snippets = Vec::new();
    let mut duration_ms_list = Vec::new();
    let mut error_messages = Vec::new();

    for msg in batch.drain(..) {
        let DbMessage::Execution {
            id,
            function_name,
            status_code,
            stdout_snippet,
            duration_ms,
            error_message,
            ..
        } = msg;

        ids.push(id);
        function_names.push(function_name);
        status_codes.push(status_code);
        stdout_snippets.push(stdout_snippet);
        duration_ms_list.push(duration_ms);
        error_messages.push(error_message);
    }

    // Bulk insert using UNNEST for high performance
    let query = "
        INSERT INTO executions (id, function_name, status_code, stdout_snippet, duration_ms, error_message)
        SELECT * FROM UNNEST($1::uuid[], $2::varchar[], $3::integer[], $4::text[], $5::bigint[], $6::text[])
    ";

    match sqlx::query(query)
        .bind(&ids)
        .bind(&function_names)
        .bind(&status_codes)
        .bind(&stdout_snippets)
        .bind(&duration_ms_list)
        .bind(&error_messages)
        .execute(pool)
        .await
    {
        Ok(_) => {
            tracing::debug!("Successfully flushed batch of {} records", ids.len());
        }
        Err(e) => {
            tracing::error!("Failed to flush batch to DB: {}", e);
            // In a real system, we might want to retry or dead-letter these
        }
    }
}
