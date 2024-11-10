# Mock API Manager

A dynamic mock API server built with [Rust](https://www.rust-lang.org/) and [Actix Web](https://actix.rs/). This server allows for easy creation, editing, and deletion of mock API responses, with support for dynamic responses based on request parameters, body content, headers, arrays, and more.

## Table of Contents

- [Overview](#overview)
- [Features and Capabilities](#features-and-capabilities)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
   - [Clone the Repository](#clone-the-repository)
   - [Build the Project](#build-the-project)
   - [Run the Server Locally](#run-the-server-locally)
- [Usage](#usage)
   - [Access the Web API Manager](#access-the-web-api-manager)
   - [Managing Mocks via Web Interface](#managing-mocks-via-web-interface)
   - [Dynamic Response Generation](#dynamic-response-generation)
      - [Query Parameters](#query-parameters)
      - [Headers](#headers)
      - [JSON Body Fields](#json-body-fields)
      - [Arrays and Nested JSON](#arrays-and-nested-json)
      - [Path Parameters](#path-parameters)
      - [Built-in Handlebars Helpers](#built-in-handlebars-helpers)
   - [Supported API Endpoints](#supported-api-endpoints)
   - [Example Usage with curl](#example-usage-with-curl)
- [Contributing](#contributing)
- [License](#license)

## Overview

**Mock API Manager** is a powerful tool for developers and testers who need to simulate API endpoints without building the actual backend services. It's ideal for frontend development, integration testing, and simulating various API behaviors in development and testing environments.

Built with high-performance Rust and Actix Web, the Mock API Manager offers dynamic mock responses, placeholder replacement, and an intuitive web interface for managing your mocks.

## Features and Capabilities

1. **Dynamic Mock Responses**:
   - **HTTP Method Support**: Supports `GET`, `POST`, `PUT`, and `DELETE`.
   - **Custom Status Codes**: Return any valid HTTP status code.
   - **Response Delay Simulation**: Add artificial delays to responses to simulate network latency.
   - **Configurable Response Bodies**: Define custom JSON responses, including dynamic content.

2. **Dynamic Placeholder Replacement**:
   - **Request-based Placeholders**: Replace placeholders in responses with values from query parameters, headers, JSON body fields, arrays, or path parameters.
   - **Built-in Helpers**: Use Handlebars helpers for dynamic data generation, such as timestamps, random numbers, and strings.

3. **Web API Manager**:
   - **Intuitive Interface**: Manage mocks via a web-based form with real-time validation and feedback.
   - **Mock List Management**: View, edit, or delete existing mocks directly from the interface.
   - **Syntax Highlighting**: Enhanced JSON editing experience with syntax highlighting and formatting tools.

4. **High Performance**:
   - **Built with Rust**: Leveraging Rust's speed and safety for high throughput and low latency.
   - **Optimized Server**: Actix Web ensures efficient handling of concurrent connections.

5. **Logging and Debugging**:
   - **Environment-based Logging**: Integrated logging with `env_logger` for tracking and debugging.
   - **Error Handling**: Comprehensive error messages for easier troubleshooting.

6. **Scalability**:
   - **In-memory Storage**: Efficient storage of mocks for fast access.
   - **Designed for Load-Balancing**: Ready for deployment in scalable environments.

## Prerequisites

- **Rust**: Version 1.56 or newer. [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo**: Rust package manager (comes with Rust installation).
- **Optional**:
   - **Docker**: For containerized deployment. [Install Docker](https://docs.docker.com/get-docker/)

## Installation

### Clone the Repository

```bash
git clone https://github.com/your-username/mock-api-manager.git
cd mock-api-manager
```

*Replace `your-username` with your GitHub username or the appropriate repository owner.*

### Build the Project

```bash
cargo build --release
```

### Run the Server Locally

```bash
cargo run --release
```

This will start the server on `http://localhost:8080`.

## Usage

### Access the Web API Manager

Open your browser and navigate to `http://localhost:8080` to access the Mock API Manager's web interface.

### Managing Mocks via Web Interface

The Web API Manager provides an intuitive interface for creating, editing, and deleting mocks.

- **Create a New Mock**:
   1. Fill in the fields in the form:
      - **API Name**: Unique identifier for your mock API endpoint (no spaces allowed).
      - **Response**: JSON-formatted response body, can include Handlebars placeholders like `{{username}}`.
      - **Status Code**: HTTP status code to return (e.g., 200, 404).
      - **Response Delay**: Optional delay in milliseconds to simulate network latency.
      - **HTTP Method**: HTTP method to which this mock should respond (`GET`, `POST`, etc.).
   2. Click **Save Mock** to create the mock API.

- **Edit an Existing Mock**:
   - Click the **Edit** button next to the mock entry in the **Current Mocks** table. The form will auto-fill with the mock's current data for editing.
   - Make the necessary changes and click **Save Mock**.

- **Delete a Mock**:
   - Click the **Delete** button next to the mock entry to remove it.

### Dynamic Response Generation

The Mock API Manager supports dynamic content in responses using Handlebars templating. You can include placeholders that will be replaced with values from the incoming request or generated dynamically.

#### Query Parameters

**Usage**: Access query parameters from the URL.

**Example**:

- **Request**:

  ```bash
  GET /mock/user-info?username=JohnDoe
  ```

- **Response Template**:

  ```json
  {
    "message": "Hello, {{username}}"
  }
  ```

- **Resulting Response**:

  ```json
  {
    "message": "Hello, JohnDoe"
  }
  ```

#### Headers

**Usage**: Access values from request headers.

**Example**:

- **Request**:

  ```bash
  GET /mock/header-example
  Header: X-Custom-Header: CustomValue
  ```

- **Response Template**:

  ```json
  {
    "received_header": "{{X-Custom-Header}}"
  }
  ```

- **Resulting Response**:

  ```json
  {
    "received_header": "CustomValue"
  }
  ```

#### JSON Body Fields

**Usage**: Access fields from a JSON request body.

**Example**:

- **Request**:

  ```bash
  POST /mock/login
  Content-Type: application/json

  {
    "username": "JaneDoe",
    "password": "secret"
  }
  ```

- **Response Template**:

  ```json
  {
    "status": "User {{username}} logged in successfully"
  }
  ```

- **Resulting Response**:

  ```json
  {
    "status": "User JaneDoe logged in successfully"
  }
  ```

#### Arrays and Nested JSON

**Usage**: Access elements within arrays and nested JSON structures in the request body.

**Example**:

- **Request**:

  ```bash
  POST /mock/process-items
  Content-Type: application/json

  {
    "items": [
      {"id": 1, "name": "ItemOne"},
      {"id": 2, "name": "ItemTwo"}
    ],
    "total": 2
  }
  ```

- **Response Template**:

  ```json
  {
    "processed_items": [
      {{#each items}}
      {
        "item_id": "{{id}}",
        "item_name": "{{name}}"
      }{{#unless @last}},{{/unless}}
      {{/each}}
    ],
    "total_items": "{{total}}"
  }
  ```

- **Resulting Response**:

  ```json
  {
    "processed_items": [
      {
        "item_id": "1",
        "item_name": "ItemOne"
      },
      {
        "item_id": "2",
        "item_name": "ItemTwo"
      }
    ],
    "total_items": "2"
  }
  ```

**Explanation**:

- The `{{#each items}}...{{/each}}` block iterates over the `items` array in the request body.
- `{{id}}` and `{{name}}` access the fields of each item.
- `{{#unless @last}},{{/unless}}` adds a comma between items, except after the last item.

#### Path Parameters

**Usage**: While the current implementation uses `api_name` as a fixed segment in the URL, you can simulate path parameters within the `api_name` by defining dynamic endpoints.

**Example**:

- **API Name**: `user-{{userId}}`

- **Response Template**:

  ```json
  {
    "userId": "{{userId}}",
    "details": "Details for user {{userId}}"
  }
  ```

- **Request**:

  ```bash
  GET /mock/user-42
  ```

- **Resulting Response**:

  ```json
  {
    "userId": "42",
    "details": "Details for user 42"
  }
  ```

**Note**: This approach treats the user ID as part of the `api_name`. Adjust your mock definitions accordingly.

#### Built-in Handlebars Helpers

The Mock API Manager includes several built-in helpers to generate dynamic content.

1. **`current_datetime`**: Inserts the current date and time.

   - **Usage**: `{{current_datetime "format"}}`
   - **Example**:

     ```json
     {
       "timestamp": "{{current_datetime \"%Y-%m-%d %H:%M:%S\"}}"
     }
     ```

   - **Resulting Response**:

     ```json
     {
       "timestamp": "2024-11-09 12:34:56"
     }
     ```

2. **`random_number`**: Generates a random number within a specified range.

   - **Usage**: `{{random_number min max}}`
   - **Example**:

     ```json
     {
       "verification_code": "{{random_number 1000 9999}}"
     }
     ```

   - **Resulting Response**:

     ```json
     {
       "verification_code": "4821"
     }
     ```

3. **`ordered_number`**: Generates an incrementing number each time it is called.

   - **Usage**: `{{ordered_number}}`
   - **Example**:

     ```json
     {
       "order_id": "{{ordered_number}}"
     }
     ```

   - **Resulting Response**:

     ```json
     {
       "order_id": "1"
     }
     ```

4. **`random_string`**: Generates a random string matching a given regex pattern.

   - **Usage**: `{{random_string "regex_pattern"}}`
   - **Example**:

     ```json
     {
       "session_token": "{{random_string \"[A-Za-z0-9]{16}\"}}"
     }
     ```

   - **Resulting Response**:

     ```json
     {
       "session_token": "a1B2c3D4e5F6g7H8"
     }
     ```

**Note**: When using special characters in format strings or regex patterns, ensure they are properly escaped.

### Supported API Endpoints

- **GET `/list-mocks`**: Returns a list of all saved mocks.
- **POST `/save-mock`**: Saves a new mock API.
- **PUT `/update-mock/{id}`**: Updates an existing mock.
- **DELETE `/delete-mock/{id}`**: Deletes a mock by ID.
- **GET `/get-mock/{id}`**: Retrieves a single mock by ID.
- **Dynamic Endpoint `/mock/{api_name}`**: Returns a dynamic response for the specified mock endpoint.

### Example Usage with curl

#### Save a New Mock

```bash
curl -X POST http://localhost:8080/save-mock \
-H "Content-Type: application/json" \
-d '{
    "api_name": "greeting",
    "response": "{\"message\": \"Hello, {{name}}!\"}",
    "status": 200,
    "delay": 0,
    "method": "GET"
}'
```

#### Access the Mock Response Using Query Parameters

```bash
curl -X GET 'http://localhost:8080/mock/greeting?name=Alice'
```

**Response**:

```json
{
  "message": "Hello, Alice!"
}
```

#### Access the Mock Response Using Headers

First, save a mock that uses a header value:

```bash
curl -X POST http://localhost:8080/save-mock \
-H "Content-Type: application/json" \
-d '{
    "api_name": "header-example",
    "response": "{\"received_header\": \"{{X-Custom-Header}}\"}",
    "status": 200,
    "delay": 0,
    "method": "GET"
}'
```

Then, make a request with the custom header:

```bash
curl -X GET http://localhost:8080/mock/header-example \
-H "X-Custom-Header: CustomValue"
```

**Response**:

```json
{
  "received_header": "CustomValue"
}
```

#### Access the Mock Response Using JSON Body Fields

First, save a mock that uses JSON body fields:

```bash
curl -X POST http://localhost:8080/save-mock \
-H "Content-Type: application/json" \
-d '{
    "api_name": "login",
    "response": "{\"status\": \"User {{username}} logged in successfully\"}",
    "status": 200,
    "delay": 0,
    "method": "POST"
}'
```

Then, make a POST request with a JSON body:

```bash
curl -X POST http://localhost:8080/mock/login \
-H "Content-Type: application/json" \
-d '{"username": "Bob", "password": "secret"}'
```

**Response**:

```json
{
  "status": "User Bob logged in successfully"
}
```

#### Access the Mock Response Using Arrays and Nested JSON

First, save a mock that processes arrays in the request body:

```bash
curl -X POST http://localhost:8080/save-mock \
-H "Content-Type: application/json" \
-d '{
    "api_name": "process-items",
    "response": "{
      \"processed_items\": [
        {{#each items}}
        {
          \"item_id\": \"{{id}}\",
          \"item_name\": \"{{name}}\"
        }{{#unless @last}},{{/unless}}
        {{/each}}
      ],
      \"total_items\": \"{{total}}\"
    }",
    "status": 200,
    "delay": 0,
    "method": "POST"
}'
```

Then, make a POST request with an array in the JSON body:

```bash
curl -X POST http://localhost:8080/mock/process-items \
-H "Content-Type: application/json" \
-d '{
  "items": [
    {"id": 1, "name": "ItemOne"},
    {"id": 2, "name": "ItemTwo"}
  ],
  "total": 2
}'
```

**Response**:

```json
{
  "processed_items": [
    {
      "item_id": "1",
      "item_name": "ItemOne"
    },
    {
      "item_id": "2",
      "item_name": "ItemTwo"
    }
  ],
  "total_items": "2"
}
```

#### Access the Mock Response Using Path Parameters

Since path parameters are part of the `api_name`, you can define mocks that include variable segments.

- **Save a Mock**:

  ```bash
  curl -X POST http://localhost:8080/save-mock \
  -H "Content-Type: application/json" \
  -d '{
      "api_name": "user-{{userId}}",
      "response": "{\"userId\": \"{{userId}}\", \"details\": \"Details for user {{userId}}\"}",
      "status": 200,
      "delay": 0,
      "method": "GET"
  }'
  ```

- **Access the Mock**:

  ```bash
  curl -X GET http://localhost:8080/mock/user-42
  ```

- **Response**:

  ```json
  {
    "userId": "42",
    "details": "Details for user 42"
  }
  ```

**Note**: This method simulates path parameters by embedding them in the `api_name`.

#### Utilize Built-in Helpers

First, save a mock that uses built-in helpers:

```bash
curl -X POST http://localhost:8080/save-mock \
-H "Content-Type: application/json" \
-d '{
    "api_name": "generate-code",
    "response": "{\"timestamp\": \"{{current_datetime \\\"%Y-%m-%dT%H:%M:%SZ\\\"}}\", \"code\": \"{{random_number 1000 9999}}\"}",
    "status": 200,
    "delay": 0,
    "method": "GET"
}'
```

Then, access the mock:

```bash
curl -X GET http://localhost:8080/mock/generate-code
```

**Response**:

```json
{
  "timestamp": "2024-11-09T12:34:56Z",
  "code": "5678"
}
```

#### Simulate Response Delay

You can simulate network latency by specifying a delay in milliseconds when creating the mock.

**Example**:

- **Create a Mock with Delay**:

  ```bash
  curl -X POST http://localhost:8080/save-mock \
  -H "Content-Type: application/json" \
  -d '{
      "api_name": "delayed-response",
      "response": "{\"message\": \"This response is delayed by 2 seconds.\"}",
      "status": 200,
      "delay": 2000,
      "method": "GET"
  }'
  ```

- **Access the Mock**:

  ```bash
  curl -X GET http://localhost:8080/mock/delayed-response
  ```

- **Result**: The response will be delayed by approximately 2 seconds.

#### Use Custom Status Codes

You can specify any valid HTTP status code when creating a mock.

**Example**:

- **Create a Mock with 404 Status**:

  ```bash
  curl -X POST http://localhost:8080/save-mock \
  -H "Content-Type: application/json" \
  -d '{
      "api_name": "not-found-example",
      "response": "{\"error\": \"Resource not found.\"}",
      "status": 404,
      "delay": 0,
      "method": "GET"
  }'
  ```

- **Access the Mock**:

  ```bash
  curl -X GET http://localhost:8080/mock/not-found-example
  ```

- **Response**:

  ```json
  {
    "error": "Resource not found."
  }
  ```

   - The HTTP status code of the response will be 404.

#### Simulate Different HTTP Methods

Mocks respond only to the HTTP method specified during creation.

**Example**:

- **Create a Mock that Responds to POST**:

  ```bash
  curl -X POST http://localhost:8080/save-mock \
  -H "Content-Type: application/json" \
  -d '{
      "api_name": "create-user",
      "response": "{\"message\": \"User {{username}} created.\"}",
      "status": 201,
      "delay": 0,
      "method": "POST"
  }'
  ```

- **Attempt GET Request**:

  ```bash
  curl -X GET http://localhost:8080/mock/create-user?username=Charlie
  ```

- **Response**:

  ```json
  "Method not allowed for this mock"
  ```

- **Make a POST Request**:

  ```bash
  curl -X POST http://localhost:8080/mock/create-user \
  -H "Content-Type: application/json" \
  -d '{"username": "Charlie"}'
  ```

- **Response**:

  ```json
  {
    "message": "User Charlie created."
  }
  ```

#### Combine Multiple Dynamic Elements

You can combine query parameters, headers, body fields, arrays, and helpers in your response templates.

**Example**:

- **Save a Mock**:

  ```bash
  curl -X POST http://localhost:8080/save-mock \
  -H "Content-Type: application/json" \
  -d '{
      "api_name": "complex-example",
      "response": "{
        \"user\": \"{{username}}\",
        \"items\": [
          {{#each items}}
          {
            \"id\": \"{{id}}\",
            \"name\": \"{{name}}\"
          }{{#unless @last}},{{/unless}}
          {{/each}}
        ],
        \"token\": \"{{random_string \\\"[A-Za-z0-9]{16}\\\"}}\",
        \"requested_at\": \"{{current_datetime \\\"%Y-%m-%dT%H:%M:%SZ\\\"}}\",
        \"user_agent\": \"{{User-Agent}}\"
      }",
      "status": 200,
      "delay": 0,
      "method": "POST"
  }'
  ```

- **Access the Mock**:

  ```bash
  curl -X POST 'http://localhost:8080/mock/complex-example?username=Dave' \
  -H "User-Agent: curl/7.68.0" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"id": 1, "name": "ItemOne"},
      {"id": 2, "name": "ItemTwo"}
    ]
  }'
  ```

- **Response**:

  ```json
  {
    "user": "Dave",
    "items": [
      {
        "id": "1",
        "name": "ItemOne"
      },
      {
        "id": "2",
        "name": "ItemTwo"
      }
    ],
    "token": "a1B2c3D4e5F6g7H8",
    "requested_at": "2024-11-09T12:34:56Z",
    "user_agent": "curl/7.68.0"
  }
  ```

## Contributing

Contributions are welcome! Please follow these guidelines:

- **Fork the Repository**: Create a personal fork of the repository on GitHub.
- **Create a Feature Branch**: Develop your feature or fix in a dedicated branch.
- **Write Tests**: Ensure your code changes are covered by tests.
- **Submit a Pull Request**: Create a pull request against the main repository.

Please ensure:

- Code is formatted using `rustfmt`.
- All existing tests pass.
- Commit messages are clear and descriptive.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

*Developed by [Md Hasan Basri](https://www.linkedin.com/in/pothiq/)*

*Version 1.0.0 | Build Year: 2024*

---