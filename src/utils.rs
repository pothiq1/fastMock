// src/utils.rs

// Author: Md Hasan Basri
// Email: pothiq@gmail.com

use anyhow::Result;
use chrono::Utc;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    Client,
}; // Kubernetes client imports
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use std::env;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORDERED_NUMBER: AtomicUsize = AtomicUsize::new(1);

/// Check if running in Kubernetes by detecting the service account token.
fn is_running_in_kubernetes() -> bool {
    fs::metadata("/var/run/secrets/kubernetes.io/serviceaccount/token").is_ok()
}

/// Get other pod IPs if running in Kubernetes; otherwise, skip.
pub async fn get_other_pod_ips() -> Result<Vec<String>> {
    if !is_running_in_kubernetes() {
        println!("Not running in a Kubernetes environment; skipping pod synchronization.");
        return Ok(Vec::new());
    }

    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create Kubernetes client: {}", e);
            return Ok(Vec::new()); // Return empty vector instead of error
        }
    };

    let namespace = env::var("K8S_NAMESPACE").unwrap_or_else(|_| "default".to_string());
    let app_label = env::var("APP_LABEL").unwrap_or_else(|_| "omock".to_string());
    let own_pod_ip = env::var("POD_IP").unwrap_or_else(|_| "".to_string());

    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().labels(&format!("app={}", app_label));

    let pod_list = pods.list(&lp).await.map_err(|e| {
        eprintln!("Failed to list pods: {}", e);
        e
    })?;

    let ips: Vec<String> = pod_list
        .items
        .into_iter()
        .filter_map(|pod| pod.status.and_then(|status| status.pod_ip))
        .filter(|ip| ip != &own_pod_ip) // Exclude own pod IP
        .collect();

    if ips.is_empty() {
        println!("No peer pods found for synchronization.");
    }

    Ok(ips)
}

/// Register custom Handlebars helpers
pub fn register_helpers(handlebars: &mut Handlebars<'_>) {
    handlebars.register_helper("current_datetime", Box::new(current_datetime));
    handlebars.register_helper("random_number", Box::new(random_number));
    handlebars.register_helper("ordered_number", Box::new(ordered_number));
    handlebars.register_helper("random_string", Box::new(random_string));
}

/// Helper for current date-time with custom format
fn current_datetime(
    helper: &Helper<'_, '_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let format = helper
        .param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("%Y-%m-%d %H:%M:%S");
    let current_time = Utc::now().format(format).to_string();
    out.write(&current_time)?;
    Ok(())
}

/// Helper to generate a random number within a range
fn random_number(
    helper: &Helper<'_, '_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let min = helper
        .param(0)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(0);
    let max = helper
        .param(1)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(100);
    let number = rand::thread_rng().gen_range(min..=max);
    out.write(&number.to_string())?;
    Ok(())
}

/// Helper for generating ordered numbers incrementally
fn ordered_number(
    _: &Helper<'_, '_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let number = ORDERED_NUMBER.fetch_add(1, Ordering::SeqCst);
    out.write(&number.to_string())?;
    Ok(())
}

/// Helper for generating random strings based on a regular expression
fn random_string(
    helper: &Helper<'_, '_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let pattern = helper
        .param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("[a-zA-Z0-9]{10}");
    let regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new("[a-zA-Z0-9]{10}").unwrap());

    // Generate a random string that matches the regex pattern
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .filter(|c| regex.is_match(&(*c as char).to_string()))
        .take(10)
        .map(|c| c as char)
        .collect();

    out.write(&random_string)?;
    Ok(())
}
