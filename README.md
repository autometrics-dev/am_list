# Autometrics List

A command that lists all functions that have the "autometrics" annotation.

The aim is to use this binary as a quick static analyzer that returns from a
codebase the complete list of functions that are annotated to be
autometricized.

The analysis is powered by Tree-sitter, and all the specific logic is contained
in TS queries that are specific for each language implementation.

## Current POC state

Given this (edited for simplicity) example file, where some functions
have the autometrics annotation:

```rust
use autometrics::{autometrics, encode_global_metrics};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

/// This is a simple endpoint that never errors
#[autometrics]
pub async fn get_index() -> &'static str {
    "Hello, World!"
}

/// This is a function that returns an error ~50% of the time
#[autometrics]
pub async fn get_random_error() -> Result<(), ()> {
    let should_error = random::<bool>();

    sleep_random_duration().await;

    if should_error {
        Err(())
    } else {
        Ok(())
    }
}

/// This function doesn't return a Result, but we can determine whether
/// we want to consider it a success or not by passing a function to the `ok_if` parameter.
#[autometrics(ok_if = is_success)]
pub async fn route_that_returns_into_response() -> impl IntoResponse {
    (StatusCode::OK, "Hello, World!")
}

/// Determine whether the response was a success or not
fn is_success<R>(response: &R) -> bool
where
    R: Copy + IntoResponse,
{
    response.into_response().status().is_success()
}

/// This isn't autometricized
pub async fn get_metrics() -> (StatusCode, String) {
    // ...
}

#[tokio::main]
pub async fn main() {
    // ...
    server
        .serve(app.into_make_service())
        .await
        .expect("Error starting example API server");
}

pub async fn generate_random_traffic() {
    let client = reqwest::Client::new();
    loop {
        //...
    }
}
```

We'll get

```console
$ am_list src/main.rs

All functions in src/main.rs:
get_index
get_random_error
route_that_returns_into_response
is_success
get_metrics
main
generate_random_traffic

Autometrics functions in src/main.rs:
get_index
get_random_error
route_that_returns_into_response
```
