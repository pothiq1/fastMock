// src/metrics.rs

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::HttpResponse;

#[cfg(feature = "metrics")]
use futures::future::{ok, LocalBoxFuture, Ready};
#[cfg(not(feature = "metrics"))]
use futures_util::future::{ok, Ready}; // Ensure futures-util is added to Cargo.toml

#[cfg(feature = "metrics")]
use lazy_static::lazy_static;
#[cfg(feature = "metrics")]
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Encoder, HistogramVec, TextEncoder,
};
#[cfg(feature = "metrics")]
use std::task::{Context, Poll};
#[cfg(feature = "metrics")]
use std::time::Instant;

#[cfg(not(feature = "metrics"))]
use std::task::{Context, Poll};

#[cfg(feature = "metrics")]
pub async fn metrics_handler() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        eprintln!("Failed to encode metrics: {}", e);
    }

    let response = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to convert metrics to string: {}", e);
            String::new()
        }
    };

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(response)
}

#[cfg(not(feature = "metrics"))]
pub async fn metrics_handler() -> HttpResponse {
    HttpResponse::NotFound().finish()
}

/// Middleware for collecting metrics
pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;

    #[cfg(feature = "metrics")]
    type Transform = MetricsMiddlewareService<S>;

    #[cfg(not(feature = "metrics"))]
    type Transform = S;

    type InitError = ();

    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        #[cfg(feature = "metrics")]
        {
            ok(MetricsMiddlewareService { service })
        }

        #[cfg(not(feature = "metrics"))]
        {
            ok(service)
        }
    }
}

#[cfg(feature = "metrics")]
pub struct MetricsMiddlewareService<S> {
    service: S,
}

#[cfg(feature = "metrics")]
impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = req.method().as_str().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            let duration = start_time.elapsed().as_secs_f64();

            // Adjust the path to reduce cardinality
            let endpoint = if path.starts_with("/mock/") {
                "/mock"
            } else if path.starts_with("/static/") {
                "/static"
            } else {
                path.as_str()
            };

            HTTP_REQUESTS_TOTAL
                .with_label_values(&[&method, endpoint])
                .inc();
            HTTP_REQUESTS_DURATION_SECONDS
                .with_label_values(&[&method, endpoint])
                .observe(duration);

            let status_code = res.status().as_u16().to_string();

            if !res.status().is_success() {
                HTTP_REQUESTS_ERRORS_TOTAL
                    .with_label_values(&[&method, endpoint, &status_code])
                    .inc();
            }

            Ok(res)
        })
    }
}

#[cfg(feature = "metrics")]
lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "http_requests_total",
        "Number of HTTP requests made.",
        &["method", "endpoint"]
    )
    .unwrap();
    pub static ref HTTP_REQUESTS_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request latencies in seconds.",
        &["method", "endpoint"]
    )
    .unwrap();
    pub static ref HTTP_REQUESTS_ERRORS_TOTAL: CounterVec = register_counter_vec!(
        "http_requests_errors_total",
        "Number of HTTP error responses.",
        &["method", "endpoint", "status"]
    )
    .unwrap();
}
