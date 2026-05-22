#!/usr/bin/env pwsh

# 1. Create the kind cluster
Write-Host "--- Creating kind cluster ---" -ForegroundColor Cyan
kind create cluster --config kind-config.yaml

# 2. Deploy NGINX Ingress Controller
Write-Host "--- Deploying NGINX Ingress Controller ---" -ForegroundColor Cyan
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

# 3. Wait for NGINX to be ready
Write-Host "--- Waiting for Ingress Controller to be ready (this may take a minute) ---" -ForegroundColor Yellow
kubectl wait --namespace ingress-nginx `
  --for=condition=ready pod `
  --selector=app.kubernetes.io/component=controller `
  --timeout=90s

# 4. Load the Docker image into the cluster
Write-Host "--- Loading serverless-runner image into kind ---" -ForegroundColor Cyan
kind load docker-image serverless-runner:latest

# 5. Deploy the application stack
Write-Host "--- Deploying serverless stack (Shards, PgBouncer, Runners, HPA) ---" -ForegroundColor Cyan
kubectl apply -f k8s-manifests.yaml

Write-Host "--- Deployment Complete ---" -ForegroundColor Green
Write-Host "You can now run the load test:" -ForegroundColor White
Write-Host "oha -z 2m -c 50 'http://localhost/execute/fibonacci?number=15'" -ForegroundColor Yellow
