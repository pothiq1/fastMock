@echo off

REM Variables
set IMAGE_NAME=pothiq/fastmock
set IMAGE_TAG=1.0.1

REM Check if Docker is installed
echo Checking Docker...
docker --version >nul 2>&1 || (
    echo Docker is not installed or not running. Please install and start Docker.
    exit /b
)

REM Stop any running container using the same port
echo Stopping any running containers on port 8080...
FOR /F "tokens=*" %%i IN ('docker ps -q --filter "ancestor=%IMAGE_NAME%:%IMAGE_TAG%"') DO docker stop %%i

REM Update Cargo.lock
echo Updating Cargo.lock...
cargo update || (
    echo Cargo update failed. Aborting.
    exit /b
)

REM Clean the Cargo build
echo Cleaning the Cargo build...
cargo clean || (
    echo Cargo clean failed. Aborting.
    exit /b
)

REM Build the Rust project
echo Building the Rust project...
cargo build --release || (
    echo Cargo build failed. Aborting.
    exit /b
)

REM Build the Docker image
echo Building the Docker image...
docker build -t %IMAGE_NAME%:%IMAGE_TAG% . || (
    echo Docker build failed. Aborting.
    exit /b
)

REM Run the Docker image locally
echo Running the Docker image locally on port 8080...
docker run -it --rm -p 8080:8080 %IMAGE_NAME%:%IMAGE_TAG% || (
    echo Docker run failed. Aborting.
    exit /b
)

echo Deployment complete. Application is running on http://localhost:8080.