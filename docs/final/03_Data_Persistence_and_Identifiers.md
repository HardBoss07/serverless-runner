# 03 - Data Persistence and Identifiers

## 1. Identifier Strategy: The Case for `UUIDv7`

The platform uses `UUIDv7` as the primary key for all execution logs. This is a deliberate departure from `UUIDv4` (random) or `bigserial` (auto-increment).

### 1.1 B-Tree Index Fragmentation

Standard `UUIDv4` identifiers are essentially random. When used as a primary key in a B-Tree index (default for Postgres), they cause **Index Bloat** and **Page Splits**.

- **Random Insertion:** A new record can be inserted anywhere in the B-Tree, requiring the database to constantly re-balance and split pages.
- **Cache Locality:** Random insertions frequently miss the L1/L2 database buffer cache.

### 1.2 Time-Ordered Advantage

`UUIDv7` incorporates a 48-bit Unix timestamp in milliseconds as its prefix. This makes them **lexicographically sortable** by time.

- **Append-Only Performance:** New records are always inserted at the "right-most" edge of the B-Tree.
- **Clustering:** Data is physically stored in near-chronological order, significantly improving the performance of range queries (e.g., `SELECT ... WHERE created_at > NOW() - INTERVAL '1 hour'`).

$$UUIDv7 = \underbrace{timestamp}_{48 \text{ bits}} + \underbrace{version}_{4 \text{ bits}} + \underbrace{variant}_{2 \text{ bits}} + \underbrace{random}_{74 \text{ bits}}$$

---

## 2. Micro-Batching with `UNNEST`

The system achieves 15,000+ RPS by decoupling the database write from the request path and using vectorized SQL operations.

### 2.1 MPSC Decoupling

The Runner uses a `tokio::sync::mpsc` channel as a high-speed ingestion buffer.

- **Channel Capacity:** 100,000 records.
- **Producer (API Handler):** Does a non-blocking `try_send`.
- **Consumer (Batcher):** Aggregates records.

### 2.2 Vectorized Insertion (UNNEST)

Instead of executing 100 individual `INSERT` statements, which would incur massive network and transaction overhead, the batcher sends one single statement using the `UNNEST` operator.

$$Cost_{batch} = \text{Network Latency} + \text{Transaction Overhead} + (N \times \text{Row Insertion Cost})$$

By using `UNNEST`, the "Network Latency" and "Transaction Overhead" are amortized across $N$ rows (where $N \approx 100$).

```sql
INSERT INTO executions (id, function_name, status_code, stdout_snippet, duration_ms, error_message)
SELECT * FROM UNNEST(
    $1::uuid[],     -- Array of IDs
    $2::varchar[],  -- Array of function names
    $3::integer[],  -- Array of status codes
    $4::text[],     -- Array of stdout snippets
    $5::bigint[],   -- Array of durations
    $6::text[]      -- Array of error messages
)
```

---

## 3. Sharding and Distributed Balance

The system uses a simple but effective sharding strategy where the choice of database shard is determined by the `UUIDv7` itself.

### 3.1 Shard Selection Logic

Since `UUIDv7` ends with 74 bits of randomness, the system uses the last byte of the UUID to perform a modulo operation:
$$Shard_{index} = UUID_{last\_byte} \pmod{N_{shards}}$$

### 3.2 Performance Implications

- **Perfect Distribution:** As observed in benchmarks (variance < 0.2%), the randomness of the UUID suffix ensures an even distribution of load across all Postgres shards.
- **Scalability:** New shards can be added by simply updating the modulo factor, provided a re-sharding strategy is in place for historical data.

---

## 4. Tuning: `synchronous_commit = off`

To reach ultra-high throughput, the database shards are configured with `synchronous_commit = off`.

- **Mechanism:** Postgres acknowledges the transaction success as soon as it is written to the WAL (Write-Ahead Log) in memory, rather than waiting for a physical disk flush.
- **Trade-off:** In the event of a power failure, a few milliseconds of recent data might be lost, but the write throughput increases by **3x - 5x**. For execution logs, this is an acceptable trade-off for the massive performance gain.
