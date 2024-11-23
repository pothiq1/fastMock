@echo off

REM Variables
set IMAGE_NAME=pothiq/fastmock
set IMAGE_TAG=1.0.3
set DEPLOYMENT_YAML=deploy\deployment.yaml
set HEADLESS_SERVICE_YAML=deploy\headless-service.yaml
set SERVICE_YAML=deploy\service.yaml
set DEPLOYMENT_NAME=omock-deployment
set NAMESPACE=default

REM Update Cargo.lock
echo Updating Cargo.lock...
cargo update || exit /b

REM Clean the Cargo build
echo Cleaning the Cargo build...
cargo clean || exit /b

REM Build the Rust project
echo Building the Rust project...
cargo build --release || exit /b

REM Build the Docker image
echo Building the Docker image...
docker build -t %IMAGE_NAME%:%IMAGE_TAG% . || exit /b

REM Push the Docker image
echo Pushing the Docker image to the registry...
docker push %IMAGE_NAME%:%IMAGE_TAG% || exit /b

REM Delete existing Kubernetes resources
echo Deleting existing Kubernetes resources...
kubectl delete -f %SERVICE_YAML% || echo Service not found.
kubectl delete -f %HEADLESS_SERVICE_YAML% || echo Headless Service not found.
kubectl delete -f %DEPLOYMENT_YAML% || echo Deployment not found.

REM Apply updated Kubernetes YAML files
echo Applying updated Kubernetes resources...
kubectl apply -f %SERVICE_YAML% || exit /b
kubectl apply -f %HEADLESS_SERVICE_YAML% || exit /b
kubectl apply -f %DEPLOYMENT_YAML% || exit /b

REM Wait for the deployment to become healthy
echo Waiting for deployment to become healthy...
set MAX_RETRIES=30
set RETRY_INTERVAL=10
set i=0

:check_deployment
for /f "delims=" %%R in ('kubectl get deployment %DEPLOYMENT_NAME% -n %NAMESPACE% -o jsonpath="{.status.readyReplicas}" 2^>nul') do set READY_REPLICAS=%%R
for /f "delims=" %%R in ('kubectl get deployment %DEPLOYMENT_NAME% -n %NAMESPACE% -o jsonpath="{.spec.replicas}" 2^>nul') do set DESIRED_REPLICAS=%%R

if "%READY_REPLICAS%"=="%DESIRED_REPLICAS%" if not "%DESIRED_REPLICAS%"=="0" (
    echo Deployment %DEPLOYMENT_NAME% is healthy! (%READY_REPLICAS%/%DESIRED_REPLICAS% replicas ready)
    exit /b 0
)

set /a i+=1
if %i% geq %MAX_RETRIES% (
    echo Deployment %DEPLOYMENT_NAME% failed to become healthy within the timeout period.
    exit /b 1
)

echo Waiting... (%i%/%MAX_RETRIES% attempts)
timeout /t %RETRY_INTERVAL% >nul
goto check_deployment