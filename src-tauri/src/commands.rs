use crate::proxy::{ProxyServer, RequestRule, SearchFilter};
use crate::ai_analyzer::{AIAnalyzer, AIAnalysisResult, SecurityAnalyzer, AIModel};
use crate::ai_response::{AIResponseGenerator, AIResponseConfig, ResponseType};
use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub id: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub duration: Option<u64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRequest {
    pub filter: String,
}

pub type ProxyState = Arc<ProxyServer>;

#[tauri::command]
pub async fn start_proxy(proxy: State<'_, ProxyState>) -> Result<String, String> {
    let proxy_clone = proxy.inner().clone();
    
    tokio::spawn(async move {
        if let Err(e) = proxy_clone.start().await {
            eprintln!("Failed to start proxy: {}", e);
        }
    });
    
    Ok("Proxy server started".to_string())
}

#[tauri::command]
pub async fn stop_proxy(proxy: State<'_, ProxyState>) -> Result<String, String> {
    proxy.stop().await;
    Ok("Proxy server stopped".to_string())
}

#[tauri::command]
pub async fn get_transactions(proxy: State<'_, ProxyState>) -> Result<Vec<TransactionData>, String> {
    let transactions = proxy.get_transactions().await;
    
    let transaction_data: Vec<TransactionData> = transactions
        .into_iter()
        .map(|t| TransactionData {
            id: t.id,
            method: t.request.method,
            url: t.request.url,
            status: t.response.as_ref().map(|r| r.status),
            duration: t.duration.map(|d| d.as_millis() as u64),
            timestamp: t.request.timestamp.to_rfc3339(),
        })
        .collect();
    
    Ok(transaction_data)
}

#[tauri::command]
pub async fn add_filter(
    proxy: State<'_, ProxyState>,
    filter_req: FilterRequest,
) -> Result<String, String> {
    proxy.add_filter(filter_req.filter).await;
    Ok("Filter added".to_string())
}

#[tauri::command]
pub async fn remove_filter(
    proxy: State<'_, ProxyState>,
    filter: String,
) -> Result<String, String> {
    proxy.remove_filter(&filter).await;
    Ok("Filter removed".to_string())
}

#[tauri::command]
pub async fn clear_transactions(proxy: State<'_, ProxyState>) -> Result<String, String> {
    proxy.clear_transactions().await;
    Ok("Transactions cleared".to_string())
}

#[tauri::command]
pub async fn is_proxy_running(proxy: State<'_, ProxyState>) -> Result<bool, String> {
    Ok(proxy.is_running().await)
}

// 搜索功能
#[tauri::command]
pub async fn search_transactions(
    proxy: State<'_, ProxyState>,
    filter: SearchFilter,
) -> Result<Vec<TransactionData>, String> {
    let transactions = proxy.search_transactions(filter).await;
    
    let transaction_data: Vec<TransactionData> = transactions
        .into_iter()
        .map(|t| TransactionData {
            id: t.id,
            method: t.request.method,
            url: t.request.url,
            status: t.response.as_ref().map(|r| r.status),
            duration: t.duration.map(|d| d.as_millis() as u64),
            timestamp: t.request.timestamp.to_rfc3339(),
        })
        .collect();
    
    Ok(transaction_data)
}

// 收藏功能
#[tauri::command]
pub async fn toggle_favorite(
    proxy: State<'_, ProxyState>,
    transaction_id: String,
) -> Result<bool, String> {
    Ok(proxy.toggle_favorite(&transaction_id).await)
}

#[tauri::command]
pub async fn get_favorites(proxy: State<'_, ProxyState>) -> Result<Vec<TransactionData>, String> {
    let transactions = proxy.get_favorites().await;
    
    let transaction_data: Vec<TransactionData> = transactions
        .into_iter()
        .map(|t| TransactionData {
            id: t.id,
            method: t.request.method,
            url: t.request.url,
            status: t.response.as_ref().map(|r| r.status),
            duration: t.duration.map(|d| d.as_millis() as u64),
            timestamp: t.request.timestamp.to_rfc3339(),
        })
        .collect();
    
    Ok(transaction_data)
}

// 规则管理
#[tauri::command]
pub async fn add_rule(
    proxy: State<'_, ProxyState>,
    rule: RequestRule,
) -> Result<String, String> {
    proxy.add_rule(rule).await;
    Ok("Rule added".to_string())
}

#[tauri::command]
pub async fn remove_rule(
    proxy: State<'_, ProxyState>,
    rule_id: String,
) -> Result<String, String> {
    proxy.remove_rule(&rule_id).await;
    Ok("Rule removed".to_string())
}

#[tauri::command]
pub async fn get_rules(proxy: State<'_, ProxyState>) -> Result<Vec<RequestRule>, String> {
    Ok(proxy.get_rules().await)
}

// HAR 导出
#[tauri::command]
pub async fn export_har(proxy: State<'_, ProxyState>) -> Result<String, String> {
    Ok(proxy.export_har().await)
}

// 编码工具
#[tauri::command]
pub fn encode_base64(input: String) -> Result<String, String> {
    Ok(ProxyServer::encode_base64(&input))
}

#[tauri::command]
pub fn decode_base64(input: String) -> Result<String, String> {
    ProxyServer::decode_base64(&input)
}

#[tauri::command]
pub fn encode_url(input: String) -> Result<String, String> {
    Ok(ProxyServer::encode_url(&input))
}

#[tauri::command]
pub fn decode_url(input: String) -> Result<String, String> {
    Ok(ProxyServer::decode_url(&input))
}

// AI 分析命令
#[tauri::command]
pub async fn analyze_transaction(
    proxy: State<'_, ProxyState>,
    transaction_id: String,
) -> Result<AIAnalysisResult, String> {
    let transactions = proxy.get_transactions().await;
    let transaction = transactions
        .iter()
        .find(|t| t.id == transaction_id)
        .ok_or("Transaction not found")?;
    
    let ai_analyzer = AIAnalyzer::new(
        None,
        AIModel::OpenAI { model: "gpt-3.5-turbo".to_string() }
    );
    
    ai_analyzer.analyze_transaction(transaction).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn detect_vulnerabilities(
    proxy: State<'_, ProxyState>,
    transaction_id: String,
) -> Result<Vec<String>, String> {
    let transactions = proxy.get_transactions().await;
    let transaction = transactions
        .iter()
        .find(|t| t.id == transaction_id)
        .ok_or("Transaction not found")?;
    
    let ai_analyzer = AIAnalyzer::new(
        None,
        AIModel::OpenAI { model: "gpt-3.5-turbo".to_string() }
    );
    let security_analyzer = SecurityAnalyzer::new(ai_analyzer);
    
    security_analyzer.detect_vulnerabilities(transaction).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_ai_insights(
    proxy: State<'_, ProxyState>,
) -> Result<Vec<String>, String> {
    let transactions = proxy.get_transactions().await;
    
    let ai_analyzer = AIAnalyzer::new(
        None,
        AIModel::OpenAI { model: "gpt-3.5-turbo".to_string() }
    );
    
    let mut insights = Vec::new();
    
    // 获取异常检测
    let anomalies = ai_analyzer.detect_anomalies(&transactions).await
        .map_err(|e| e.to_string())?;
    insights.extend(anomalies);
    
    // 获取优化建议
    let optimizations = ai_analyzer.suggest_optimizations(&transactions).await
        .map_err(|e| e.to_string())?;
    insights.extend(optimizations);
    
    Ok(insights)
}

// AI 响应生成命令
#[tauri::command]
pub async fn generate_ai_response(
    _request_data: serde_json::Value,
) -> Result<String, String> {
    let config = AIResponseConfig {
        enable_ai_responses: true,
        response_type: ResponseType::Enhanced,
        content_template: None,
        ai_model: "gpt-3.5-turbo".to_string(),
    };
    
    let _generator = AIResponseGenerator::new(config);
    
    // 这里需要从 request_data 构建 HttpRequest
    // 暂时返回模拟响应
    Ok(serde_json::json!({
        "ai_generated": true,
        "message": "AI 生成的响应",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }).to_string())
}
