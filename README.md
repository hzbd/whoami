# Whoami

A tiny, high-performance HTTP service, written in Rust, that returns its hostname. It provides both plain text and JSON responses. Ideal for use in containerized environments like Kubernetes or Docker for service discovery and testing.

## Features

*   **High Performance**: Built with Rust and the `warp` framework on top of Tokio and Hyper.
*   **Lightweight**: Minimal resource footprint and a small container image size.
*   **Dual Format**: Provides hostname in both plain text and JSON formats via separate endpoints.

## Quick Start

### 1. Build docker image

```bash
docker build -f Dockerfile -t containerpi/whoami .
```

### 1. Run container

```bash
docker pull containerpi/whoami:latest
docker run -d -p 8080:8080 --name whoami containerpi/whoami
```

### 3. Test the Service

You can now query the service on port 8080.

#### Get Plain Text Response

```bash
# Request the root endpoint for a plain text response
$ curl http://127.0.0.1:8080

# Response:
I'm 0e8cf88790a3
```
*(The hostname will be the container ID or your machine's local hostname)*

#### Get JSON Response

```bash
# Request the /json endpoint for a JSON response
$ curl http://127.0.0.1:8080/json

# Response:
{"hostname":"0e8cf88790a3"}
```

## API Endpoints

### `GET /`

Returns the hostname of the container/machine as a plain text string.

*   **Method**: `GET`
*   **Path**: `/`
*   **Success Response**:
    *   **Code**: `200 OK`
    *   **Content-Type**: `text/plain; charset=utf-p`
    *   **Body**: `I'm <hostname>`

### `GET /json`

Returns the hostname of the container/machine as a JSON object.

*   **Method**: `GET`
*   **Path**: `/json`
*   **Success Response**:
    *   **Code**: `200 OK`
    *   **Content-Type**: `application/json`
    *   **Body**: `{"hostname": "<hostname>"}`
