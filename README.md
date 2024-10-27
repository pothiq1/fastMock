
# Mock API Manager

A dynamic mock API server built with Actix Web (Rust). This server allows for easy creation, editing, and deletion of mock API responses, with support for dynamic responses based on request parameters, body content, or headers.

## Features

1. **Dynamic Mock Responses**:
   - Supports multiple HTTP methods: `GET`, `POST`, `PUT`, `DELETE`.
   - Responds only to the specified method of each mock.
   - Custom status codes, delay settings, and configurable response bodies.
   - JSON-based responses with placeholder values.

2. **Dynamic Placeholder Replacement**:
   - JSONPath support for parsing JSON request bodies.
   - Query parameters, JSON body, and request headers can all be used to dynamically replace placeholders in response templates.

3. **Web-Based Interface**:
   - Manage mocks via a web-based form, built with Bootstrap for a responsive and user-friendly experience.
   - Displays a list of all current mocks with options to edit or delete directly from the table.

4. **Kubernetes-Ready**:
   - Deployable on Kubernetes clusters with in-memory storage shared across multiple pods.
   - Scalable design ensures uniformity and synchronization across instances, making it ideal for load-balanced environments.

5. **Logging**:
   - Integrated environment-based logging with `env_logger` for tracking and debugging.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) and Cargo
- [Docker](https://www.docker.com/) (for containerized deployment)
- [Kubernetes](https://kubernetes.io/) (optional, for cloud scaling)

### Installation

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/your-repo/mock-api-manager.git
   cd mock-api-manager

2. **Build the Project**:
   ```bash
   cargo build --release

3. **Run the Server Locally**:
   ```bash
   cargo run

4. **Access the Web Interface**:

  Open your browser and navigate to http://localhost:8080.

# Configuration for Kubernetes
To deploy this service on Kubernetes, create a Deployment file (deployment.yaml) with the following configuration (assuming the Docker image is already built and pushed):

   ```yaml
     apiVersion: apps/v1
     kind: Deployment
     metadata:
       name: mock-api-manager
     spec:
       replicas: 3
       selector:
         matchLabels:
           app: mock-api
       template:
         metadata:
           labels:
             app: mock-api
         spec:
           containers:
             - name: mock-api-manager
               image: your-docker-image:latest
               ports:
                 - containerPort: 8080
  ``` 
# Usage

## Creating and Updating Mocks

Save a New Mock:

1. Fill in the fields in the form on the main page, including the API name, HTTP method, response body, status, and delay time.
Edit an Existing Mock:

2. Click the “Edit” button next to the mock entry. The form fields will auto-fill with the mock’s current data for editing.
Dynamic Query and Body Parsing:

3. Use placeholders within the response body that dynamically replace based on request body, headers, or query parameters.

# Supported API Endpoints

* GET /list-mocks: Returns a list of all saved mocks.
* POST /save-mock: Saves a new mock API.
* PUT /update-mock/{id}: Updates an existing mock.
* DELETE /delete-mock/{id}: Deletes a mock by ID.
* GET /mock/{api_name}: Returns a dynamic response for the specified mock endpoint.

# Example Usage with curl

   ```bash
   # Save a new mock
   curl -X POST http://localhost:8080/save-mock \
   -H "Content-Type: application/json" \
   -d '{
       "api_name": "example-api",
       "response": "{\"message\": \"Hello, {{username}}\"}",
       "status": 200,
       "delay": 100,
       "method": "GET"
   }'
   ```
# Access a mock response
curl -X GET http://localhost:8080/mock/example-api?username=World

# Contributing

Feel free to fork the repository and submit pull requests with improvements, bug fixes, or new features. Make sure to write descriptive commit messages and follow Rust’s best practices.