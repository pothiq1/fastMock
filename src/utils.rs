// src/utils.rs

use kube::{api::{Api, ListParams}, Client}; // Kubernetes client imports
use k8s_openapi::api::core::v1::Pod;
use chrono::Utc;
use anyhow::Result;
use handlebars::{Handlebars, Helper, Context, RenderContext, Output, HelperResult};
use rand::{distributions::Alphanumeric, Rng}; // Add `rand` imports
use regex::Regex; // Add Regex import
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs;

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

    let client = Client::try_default().await.map_err(|e| {
        eprintln!("Failed to create Kubernetes client: {}", e);
        e
    })?;

    let pods: Api<Pod> = Api::namespaced(client, "default"); // Adjust namespace if needed
    let lp = ListParams::default().labels("app=your-app-label"); // Update to match your app's label

    let pod_list = pods.list(&lp).await.map_err(|e| {
        eprintln!("Failed to list pods: {}", e);
        e
    })?;

    let ips: Vec<String> = pod_list
        .items
        .into_iter()
        .filter_map(|pod| pod.status.and_then(|status| status.pod_ip))
        .collect();

    if ips.is_empty() {
        println!("No peer pods found for synchronization.");
    }

    Ok(ips)
}

/// Register custom Handlebars helpers
pub fn register_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("current_datetime", Box::new(current_datetime));
    handlebars.register_helper("random_number", Box::new(random_number));
    handlebars.register_helper("ordered_number", Box::new(ordered_number));
    handlebars.register_helper("random_string", Box::new(random_string));
}

/// Helper for current date-time with custom format
fn current_datetime(helper: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let format = helper.param(0).and_then(|v| v.value().as_str()).unwrap_or("%Y-%m-%d %H:%M:%S");
    let current_time = Utc::now().format(format).to_string();
    out.write(&current_time)?;
    Ok(())
}

/// Helper to generate a random number within a range
fn random_number(helper: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let min = helper.param(0).and_then(|v| v.value().as_u64()).unwrap_or(0);
    let max = helper.param(1).and_then(|v| v.value().as_u64()).unwrap_or(100);
    let number = rand::thread_rng().gen_range(min..=max);
    out.write(&number.to_string())?;
    Ok(())
}

/// Helper for generating ordered numbers incrementally
fn ordered_number(_: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let number = ORDERED_NUMBER.fetch_add(1, Ordering::SeqCst);
    out.write(&number.to_string())?;
    Ok(())
}

/// Helper for generating random strings based on a regular expression
fn random_string(helper: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let pattern = helper.param(0).and_then(|v| v.value().as_str()).unwrap_or("[a-zA-Z0-9]{10}");
    let regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new("[a-zA-Z0-9]{10}").unwrap());

    // Generate a random string that matches the regex pattern
    let random_string: String = (0..10)
        .map(|_| {
            let char = rand::thread_rng().sample(Alphanumeric) as char;
            if regex.is_match(&char.to_string()) {
                char
            } else {
                'a' // default fallback character
            }
        })
        .collect();

    out.write(&random_string)?;
    Ok(())
}
