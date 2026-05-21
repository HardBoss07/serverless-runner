-- Initial schema: executions table plus supporting indexes.
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

CREATE INDEX idx_executions_function_name ON executions(function_name);
CREATE INDEX idx_executions_created_at ON executions(created_at DESC);
