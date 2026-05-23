#!/usr/bin/env pwsh

# redeploy-cluster.ps1 - Fast redeployment of the application stack

Write-Host "--- Redeploying Serverless Platform Stack ---" -ForegroundColor Cyan

# 1. Delete existing deployments to ensure fresh start (optional, but helps with init scripts)
Write-Host "Cleaning up existing deployments..." -ForegroundColor Gray
kubectl delete -f k8s-manifests.yaml --ignore-not-found

# 2. Re-apply manifests
Write-Host "Applying k8s-manifests.yaml..." -ForegroundColor Gray
kubectl apply -f k8s-manifests.yaml

# 3. Wait for database and runner to be ready
Write-Host "Waiting for components to reach Ready state..." -ForegroundColor Yellow
$timeout = 120
$startTime = Get-Date

while (((Get-Date) - $startTime).TotalSeconds -lt $timeout) {
    $pods = kubectl get pods -n serverless-platform --no-headers
    $allReady = $true
    foreach ($pod in $pods) {
        if ($pod -notmatch "1/1\s+Running") {
            $allReady = $false
            break
        }
    }
    
    if ($allReady -and $pods.Count -ge 7) { # 2 postgres + 2 pgbouncer + 3 runners
        Write-Host "All components are ready!" -ForegroundColor Green
        return
    }
    
    Write-Host "." -NoNewline
    Start-Sleep -Seconds 5
}

Write-Host "`nTimeout reached! Some pods might still be initializing." -ForegroundColor Red
kubectl get pods -n serverless-platform
exit 1
