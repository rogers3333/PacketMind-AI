mod proxy;
mod commands;
mod ai_analyzer;
mod ai_response;

use std::sync::Arc;
use commands::{
    ProxyState, start_proxy, stop_proxy, get_transactions, add_filter, remove_filter, clear_transactions, is_proxy_running,
    search_transactions, toggle_favorite, get_favorites, add_rule, remove_rule, get_rules,
    export_har, encode_base64, decode_base64, encode_url, decode_url,
    analyze_transaction, detect_vulnerabilities, get_ai_insights, generate_ai_response
};
use proxy::ProxyServer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Create proxy server instance
    let proxy_server = Arc::new(ProxyServer::new(8080));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage::<ProxyState>(proxy_server)
        .invoke_handler(tauri::generate_handler![
            start_proxy,
            stop_proxy,
            get_transactions,
            add_filter,
            remove_filter,
            clear_transactions,
            is_proxy_running,
            search_transactions,
            toggle_favorite,
            get_favorites,
            add_rule,
            remove_rule,
            get_rules,
            export_har,
            encode_base64,
            decode_base64,
            encode_url,
            decode_url,
            analyze_transaction,
            detect_vulnerabilities,
            get_ai_insights,
            generate_ai_response
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
