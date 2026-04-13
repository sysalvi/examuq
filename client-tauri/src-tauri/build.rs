fn main() {
    println!("cargo:rerun-if-env-changed=EXAMUQ_UPDATE_CHANNEL");
    println!("cargo:rerun-if-env-changed=EXAMUQ_ALLOW_BETA");
    println!("cargo:rerun-if-env-changed=EXAMUQ_UPDATER_ENDPOINT_TEMPLATE");

    let channel = std::env::var("EXAMUQ_UPDATE_CHANNEL").unwrap_or_else(|_| "stable".to_string());
    let normalized_channel = channel.trim().to_ascii_lowercase();
    let allow_beta = std::env::var("EXAMUQ_ALLOW_BETA")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false);

    if !normalized_channel.is_empty()
        && normalized_channel != "stable"
        && normalized_channel != "beta"
    {
        panic!(
            "EXAMUQ_UPDATE_CHANNEL harus 'stable' atau 'beta', dapat: '{}'",
            channel
        );
    }

    if normalized_channel == "beta" && !allow_beta {
        panic!(
            "EXAMUQ_UPDATE_CHANNEL=beta hanya boleh untuk build development (set EXAMUQ_ALLOW_BETA=true)"
        );
    }

    if let Ok(template) = std::env::var("EXAMUQ_UPDATER_ENDPOINT_TEMPLATE") {
        let trimmed = template.trim();
        if trimmed.is_empty() {
            panic!("EXAMUQ_UPDATER_ENDPOINT_TEMPLATE tidak boleh kosong");
        }

        if !trimmed.contains("{channel}") {
            panic!("EXAMUQ_UPDATER_ENDPOINT_TEMPLATE wajib mengandung placeholder '{{channel}}'");
        }

        let profile = std::env::var("PROFILE").unwrap_or_default();
        if profile == "release" && !trimmed.starts_with("https://") {
            panic!("EXAMUQ_UPDATER_ENDPOINT_TEMPLATE release wajib menggunakan https://");
        }
    }

    tauri_build::build()
}
