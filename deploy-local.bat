@echo off

REM Variables
set IMAGE_NAME=pothiq/fastmock
set IMAGE_TAG=latest

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

REM Run the Docker image locally
echo Running the Docker image locally on port 8080...
docker run -it --rm -p 8080:8080 %IMAGE_NAME%:%IMAGE_TAG% || exit /b
