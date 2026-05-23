#!/usr/bin/env pwsh

# run-benchmark.ps1 - Automated build, deploy, and high-throughput stress test

$ErrorActionPreference = "Stop"

Write-Host "--- SRE BENCHMARK INITIALIZATION ---" -ForegroundColor Cyan

# 1. Build the Docker image locally
Write-Host "[1/8] Building serverless-runner:latest..." -ForegroundColor Gray
docker build -t serverless-runner:latest .

# 2. Sideload the image into the cluster
Write-Host "[2/8] Loading image into kind cluster..." -ForegroundColor Gray
kind load docker-image serverless-runner:latest

# 3. Apply the updated manifests
Write-Host "[3/8] Applying optimized Kubernetes manifests..." -ForegroundColor Gray
kubectl apply -f k8s-manifests.yaml

# 4. Force a rollout restart
Write-Host "[4/8] Restarting deployment to ensure fresh high-performance pods..." -ForegroundColor Gray
kubectl rollout restart deployment serverless-runner -n serverless-platform

# 5. Wait for the rollout to complete
Write-Host "[5/8] Waiting for rollout status..." -ForegroundColor Gray
kubectl rollout status deployment serverless-runner -n serverless-platform --timeout=180s

# 6. Launch HPA watcher in a new background job/window (simplified for CLI)
Write-Host "[6/8] System scaling state (Run 'kubectl get hpa -n serverless-platform -w' in another terminal):" -ForegroundColor Gray
kubectl get hpa -n serverless-platform

# 7. Execute the stress test using oha
Write-Host "[7/8] STARTING 5-MINUTE STRESS TEST (500 concurrent connections)..." -ForegroundColor Yellow
Write-Host "Target: http://localhost/execute/fibonacci?number=15" -ForegroundColor Gray
& oha -z 5m -c 500 "http://localhost/execute/fibonacci?number=15"

# 8. Verification: Database Row Counts
Write-Host "[8/8] VERIFICATION: Final Database Distribution" -ForegroundColor Cyan

$shard1Count = (kubectl exec -n serverless-platform deployment/postgres-shard-1 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;" | Select-Object -First 1).Trim()
$shard2Count = (kubectl exec -n serverless-platform deployment/postgres-shard-2 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;" | Select-Object -First 1).Trim()

Write-Host "Shard 1 Count: $shard1Count" -ForegroundColor White
Write-Host "Shard 2 Count: $shard2Count" -ForegroundColor White

$total = [int]$shard1Count + [int]$shard2Count
Write-Host "Total Successful Executions Logged: $total" -ForegroundColor Green
Write-Host "--- BENCHMARK COMPLETE ---" -ForegroundColor Cyan
