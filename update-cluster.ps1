#!/usr/bin/env pwsh

# 1. Delete existing Postgres deployments to force a clean initialization
Write-Host "--- Purging existing Postgres deployments to trigger init scripts ---" -ForegroundColor Yellow
kubectl delete deployment postgres-shard-1 -n serverless-platform --ignore-not-found
kubectl delete deployment postgres-shard-2 -n serverless-platform --ignore-not-found

# 2. Apply updated manifests
Write-Host "--- Applying updated k8s-manifests.yaml ---" -ForegroundColor Cyan
kubectl apply -f k8s-manifests.yaml

# 3. Restart serverless-runner to pick up updated secrets
Write-Host "--- Restarting serverless-runner deployment ---" -ForegroundColor Cyan
kubectl rollout restart deployment serverless-runner -n serverless-platform

# 4. Wait for the new pods to reach Ready state
Write-Host "--- Waiting for database shards to be ready ---" -ForegroundColor Yellow
kubectl wait --namespace serverless-platform `
  --for=condition=ready pod `
  --selector=app=postgres-shard-1 `
  --timeout=90s

kubectl wait --namespace serverless-platform `
  --for=condition=ready pod `
  --selector=app=postgres-shard-2 `
  --timeout=90s

Write-Host "--- Waiting for runners to be ready ---" -ForegroundColor Yellow
kubectl wait --namespace serverless-platform `
  --for=condition=ready pod `
  --selector=app=serverless-runner `
  --timeout=90s

Write-Host "--- Cluster update complete and verified ---" -ForegroundColor Green
Write-Host "You can now retry the load test:" -ForegroundColor White
Write-Host "oha -z 2m -c 50 'http://localhost/execute/fibonacci?number=15'" -ForegroundColor Yellow
