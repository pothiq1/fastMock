#!/bin/bash

# Variables
IMAGE_NAME="pothiq/fastmock"
IMAGE_TAG="latest"
DEPLOYMENT_YAML="./deploy/deployment.yaml"
HEADLESS_SERVICE_YAML="./deploy/headless-service.yaml"
SERVICE_YAML="./deploy/service.yaml"
DEPLOYMENT_NAME="omock-deployment" # Update with your deployment name
NAMESPACE="default"               # Update with your namespace, if not default

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo update

# Clean the Cargo build
echo "Cleaning the Cargo build..."
cargo clean

# Build the Rust project
echo "Building the Rust project..."
cargo build --release

# Build the Docker image
echo "Building the Docker image..."
docker build -t $IMAGE_NAME:$IMAGE_TAG .

# Push the Docker image
echo "Pushing the Docker image to the registry..."
docker push $IMAGE_NAME:$IMAGE_TAG

# Delete existing Kubernetes resources
echo "Deleting existing Kubernetes resources..."
kubectl delete -f $SERVICE_YAML || echo "Service not found."
kubectl delete -f $HEADLESS_SERVICE_YAML || echo "Headless Service not found."
kubectl delete -f $DEPLOYMENT_YAML || echo "Deployment not found."

# Apply updated Kubernetes YAML files
echo "Applying updated Kubernetes resources..."
kubectl apply -f $SERVICE_YAML
kubectl apply -f $HEADLESS_SERVICE_YAML
kubectl apply -f $DEPLOYMENT_YAML

# Wait for the deployment to be ready
echo "Waiting for deployment to become healthy..."
MAX_RETRIES=30
RETRY_INTERVAL=10

for ((i=1; i<=MAX_RETRIES; i++)); do
    READY_REPLICAS=$(kubectl get deployment $DEPLOYMENT_NAME -n $NAMESPACE -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
    DESIRED_REPLICAS=$(kubectl get deployment $DEPLOYMENT_NAME -n $NAMESPACE -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
    
    if [[ "$READY_REPLICAS" == "$DESIRED_REPLICAS" && "$DESIRED_REPLICAS" != "0" ]]; then
        echo "Deployment $DEPLOYMENT_NAME is healthy! ($READY_REPLICAS/$DESIRED_REPLICAS replicas ready)"
        exit 0
    fi

    echo "Waiting... ($i/$MAX_RETRIES attempts)"
    sleep $RETRY_INTERVAL
done

echo "Deployment $DEPLOYMENT_NAME failed to become healthy within the timeout period."
exit 1