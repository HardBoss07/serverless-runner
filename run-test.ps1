#!/usr/bin/env pwsh

Write-Host "--- Starting Load Test (2 minutes) ---" -ForegroundColor Cyan
Write-Host "Monitoring HPA in another window is recommended: 'kubectl get hpa -n serverless-platform -w'" -ForegroundColor Gray

oha -z 2m -c 50 "http://localhost/execute/fibonacci?number=15"

Write-Host "--- Load Test Finished ---" -ForegroundColor Green
Write-Host "--- Verification: Database Row Counts ---" -ForegroundColor Cyan

Write-Host "Shard 1 Count:" -ForegroundColor White
kubectl exec -n serverless-platform -it deployment/postgres-shard-1 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;"

Write-Host "Shard 2 Count:" -ForegroundColor White
kubectl exec -n serverless-platform -it deployment/postgres-shard-2 -- psql -U platform_user -d platform_db -t -c "SELECT count(*) FROM executions;"

Write-Host "If both counts are > 0, sharding distribution is verified." -ForegroundColor Yellow
