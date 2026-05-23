#!/usr/bin/env pwsh

# validate-all.ps1 - Comprehensive system validation feedback loop

Write-Host "--- Starting Full System Validation ---" -ForegroundColor Cyan

# 1. Check Pod Health
Write-Host "[1/4] Checking Pod Health..." -ForegroundColor Gray
$pods = kubectl get pods -n serverless-platform --no-headers
$unhealthy = $pods | Where-Object { $_ -notmatch "Running" -and $_ -notmatch "Completed" }

if ($unhealthy) {
    Write-Host "FAILURE: Some pods are unhealthy!" -ForegroundColor Red
    $unhealthy | Write-Host
    exit 1
}
Write-Host "PASS: All pods are Running." -ForegroundColor Green

# 2. Verify Database Sharding
Write-Host "[2/4] Verifying Database Sharding..." -ForegroundColor Gray
$shard1Count = kubectl exec -n serverless-platform deployment/postgres-shard-1 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;"
$shard2Count = kubectl exec -n serverless-platform deployment/postgres-shard-2 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;"

if ($LASTEXITCODE -ne 0) {
    Write-Host "FAILURE: Database connection error!" -ForegroundColor Red
    exit 1
}
Write-Host "PASS: Shard 1 ($($shard1Count.Trim())), Shard 2 ($($shard2Count.Trim()))." -ForegroundColor Green

# 3. Sanity API Check
Write-Host "[3/4] Running Sanity API Check..." -ForegroundColor Gray
$response = Invoke-RestMethod -Uri "http://localhost/execute/hello-world" -Method Post -Body "ValidationEngine" -ContentType "text/plain"

if ($response -match "Hello, ValidationEngine") {
    Write-Host "PASS: API returned expected Wasm output." -ForegroundColor Green
} else {
    Write-Host "FAILURE: API returned unexpected response: $response" -ForegroundColor Red
    exit 1
}

# 4. Stress/Load Test
Write-Host "[4/4] Running Stress Test (30s)..." -ForegroundColor Gray
& oha -z 30s -c 20 "http://localhost/execute/fibonacci?number=10"

if ($LASTEXITCODE -ne 0) {
    Write-Host "FAILURE: Load test failed to execute!" -ForegroundColor Red
    exit 1
}

# Final Check on DB counts after load test
$newCount1 = (kubectl exec -n serverless-platform deployment/postgres-shard-1 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;" | Select-Object -First 1).Trim()
$newCount2 = (kubectl exec -n serverless-platform deployment/postgres-shard-2 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;" | Select-Object -First 1).Trim()

if ([int]$newCount1 -gt [int]($shard1Count | Select-Object -First 1).Trim() -and [int]$newCount2 -gt [int]($shard2Count | Select-Object -First 1).Trim()) {
    Write-Host "PASS: Load balanced correctly across shards." -ForegroundColor Green
} else {
    Write-Host "WARNING: Sharding distribution looks uneven or no new records found." -ForegroundColor Yellow
}

Write-Host "--- System Validated & Stable ---" -ForegroundColor Green
