use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTransaction {
    pub id: String,
    pub request: HttpRequest,
    pub response: Option<HttpResponse>,
    pub duration: Option<std::time::Duration>,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub action: RuleAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Block,
    Redirect { target: String },
    Rewrite { script: String },
    Mock { response: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub keyword: String,
    pub method: Option<String>,
    pub status: Option<u16>,
    pub domain: Option<String>,
}

pub struct ProxyServer {
    port: u16,
    transactions: Arc<RwLock<Vec<HttpTransaction>>>,
    filters: Arc<RwLock<Vec<String>>>,
    rules: Arc<RwLock<Vec<RequestRule>>>,
    favorites: Arc<RwLock<Vec<String>>>,
    is_running: Arc<RwLock<bool>>,
}

impl ProxyServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            transactions: Arc::new(RwLock::new(Vec::new())),
            filters: Arc::new(RwLock::new(Vec::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            favorites: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;
        
        info!("Proxy server listening on {}", addr);
        
        *self.is_running.write().await = true;
        
        // 启动自动代理功能
        self.start_auto_proxy().await?;
        
        loop {
            let (stream, _) = listener.accept().await?;
            let transactions = self.transactions.clone();
            let filters = self.filters.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, transactions, filters).await {
                    error!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn start_auto_proxy(&self) -> Result<()> {
        info!("Starting auto proxy functionality...");
        
        // 在 macOS 上自动配置系统代理
        #[cfg(target_os = "macos")]
        {
            self.configure_macos_proxy().await?;
        }
        
        // 在 Windows 上自动配置系统代理
        #[cfg(target_os = "windows")]
        {
            self.configure_windows_proxy().await?;
        }
        
        // 在 Linux 上自动配置系统代理
        #[cfg(target_os = "linux")]
        {
            self.configure_linux_proxy().await?;
        }
        
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn configure_macos_proxy(&self) -> Result<()> {
        use std::process::Command;
        
        info!("Configuring macOS system proxy...");
        
        // 获取网络接口名称
        let get_services = Command::new("networksetup")
            .args(&["-listallnetworkservices"])
            .output();
            
        if let Ok(output) = get_services {
            let services = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = services.lines().skip(1).collect(); // 跳过第一行（标题）
            
            for service in lines {
                let service = service.trim();
                if !service.is_empty() {
                    // 设置 HTTP 代理
                    let _http_result = Command::new("networksetup")
                        .args(&["-setwebproxy", service, "127.0.0.1", &self.port.to_string()])
                        .output();
                        
                    // 设置 HTTPS 代理
                    let _https_result = Command::new("networksetup")
                        .args(&["-setsecurewebproxy", service, "127.0.0.1", &self.port.to_string()])
                        .output();
                        
                    // 启用 HTTP 代理
                    let _enable_http = Command::new("networksetup")
                        .args(&["-setwebproxystate", service, "on"])
                        .output();
                        
                    // 启用 HTTPS 代理
                    let _enable_https = Command::new("networksetup")
                        .args(&["-setsecurewebproxystate", service, "on"])
                        .output();
                        
                    info!("Configured proxy for network service: {}", service);
                }
            }
        }
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn configure_windows_proxy(&self) -> Result<()> {
        use std::process::Command;
        
        info!("Configuring Windows system proxy...");
        
        // 使用 PowerShell 配置代理
        let script = format!(
            r#"
            $proxy = "http://127.0.0.1:{}"
            Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings" -Name ProxyServer -Value $proxy
            Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings" -Name ProxyEnable -Value 1
            "#,
            self.port
        );
        
        let result = Command::new("powershell")
            .args(&["-Command", &script])
            .output();
            
        if let Ok(output) = result {
            if output.status.success() {
                info!("Windows proxy configured successfully");
            } else {
                warn!("Failed to configure Windows proxy: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn configure_linux_proxy(&self) -> Result<()> {
        use std::process::Command;
        
        info!("Configuring Linux system proxy...");
        
        // 尝试设置系统代理（需要适当的权限）
        let result = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "manual"])
            .output();
            
        if let Ok(output) = result {
            if output.status.success() {
                info!("Linux proxy mode set to manual");
            }
        }
        
        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        transactions: Arc<RwLock<Vec<HttpTransaction>>>,
        filters: Arc<RwLock<Vec<String>>>,
    ) -> Result<()> {
        let io = TokioIo::new(stream);
        
        let service = service_fn(|req: Request<Incoming>| {
            let transactions = transactions.clone();
            let filters = filters.clone();
            
            async move {
                Self::handle_request(req, transactions, filters).await
            }
        });

        http1::Builder::new()
            .serve_connection(io, service)
            .await?;
            
        Ok(())
    }

    async fn handle_request(
        req: Request<Incoming>,
        transactions: Arc<RwLock<Vec<HttpTransaction>>>,
        filters: Arc<RwLock<Vec<String>>>,
    ) -> Result<Response<String>, hyper::Error> {
        let method = req.method().to_string();
        let url = req.uri().to_string();
        
        // Check filters - 使用模糊匹配
        let filters = filters.read().await;
        let is_filtered = if !filters.is_empty() {
            let should_filter = filters.iter().any(|filter| {
                // 提取域名进行模糊匹配
                let domain = Self::extract_domain_from_url(&url);
                domain.to_lowercase().contains(&filter.to_lowercase()) || 
                url.to_lowercase().contains(&filter.to_lowercase())
            });
            
            if should_filter {
                warn!("Request filtered by '{}': {}", filters.join(", "), url);
                true
            } else {
                false
            }
        } else {
            false
        };
        
        info!("Handling request: {} {}", method, url);
        
        // Create transaction
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();
        
        let headers: HashMap<String, String> = req.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        // 读取请求体 - 暂时跳过
        let body = Vec::new();
        
        let request = HttpRequest {
            method,
            url,
            headers,
            body: body.to_vec(),
            timestamp: chrono::Utc::now(),
        };
        
        // 转发请求到目标服务器
        let response_result = Self::forward_request(&request).await;
        
        let (response, duration) = match response_result {
            Ok(resp) => (resp, start_time.elapsed()),
            Err(e) => {
                error!("Failed to forward request: {}", e);
                // 返回错误响应
                let error_response = HttpResponse {
                    status: 502,
                    headers: HashMap::new(),
                    body: format!("Proxy error: {}", e).into_bytes(),
                    timestamp: chrono::Utc::now(),
                };
                (error_response, start_time.elapsed())
            }
        };
        
        let mut tags = Vec::new();
        if is_filtered {
            tags.push("filtered".to_string());
        }
        
        let transaction = HttpTransaction {
            id: transaction_id,
            request,
            response: Some(response.clone()),
            duration: Some(duration),
            is_favorite: false,
            tags,
        };
        
        // Store transaction
        transactions.write().await.push(transaction);
        
        // Build response
        let mut response_builder = Response::builder()
            .status(StatusCode::from_u16(response.status).unwrap_or(StatusCode::OK));
            
        for (key, value) in &response.headers {
            response_builder = response_builder.header(key, value);
        }
        
        Ok(response_builder
            .body(String::from_utf8_lossy(&response.body).to_string())
            .unwrap())
    }

    fn extract_domain_from_url(url: &str) -> String {
        // 处理 CONNECT 请求格式 (CONNECT www.google.com:443)
        if url.contains(":") && !url.starts_with("http") {
            return url.split(":").next().unwrap_or(url).to_string();
        }
        
        // 处理普通 URL 格式
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                return host.to_string();
            }
        }
        
        // 如果解析失败，尝试简单的字符串提取
        if url.starts_with("http://") || url.starts_with("https://") {
            if let Some(start) = url.find("://") {
                let after_protocol = &url[start + 3..];
                if let Some(end) = after_protocol.find('/') {
                    return after_protocol[..end].to_string();
                } else if let Some(end) = after_protocol.find(':') {
                    return after_protocol[..end].to_string();
                } else {
                    return after_protocol.to_string();
                }
            }
        }
        
        url.to_string()
    }

    async fn forward_request(request: &HttpRequest) -> Result<HttpResponse> {
        // 简化的代理实现 - 返回模拟响应
        // 在实际应用中，这里会转发到真实的目标服务器
        
        let status = match request.method.as_str() {
            "GET" => 200,
            "POST" => 201,
            "PUT" => 200,
            "DELETE" => 204,
            _ => 200,
        };
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Proxy-By".to_string(), "PacketMind AI".to_string());
        
        let body = serde_json::json!({
            "message": "Proxied by PacketMind AI",
            "original_request": {
                "method": request.method,
                "url": request.url,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            },
            "proxy_info": {
                "version": "1.0.0",
                "features": ["auto_proxy", "ai_analysis", "filtering"]
            }
        }).to_string();
        
        Ok(HttpResponse {
            status,
            headers,
            body: body.into_bytes(),
            timestamp: chrono::Utc::now(),
        })
    }

    pub async fn get_transactions(&self) -> Vec<HttpTransaction> {
        self.transactions.read().await.clone()
    }

    pub async fn add_filter(&self, filter: String) {
        self.filters.write().await.push(filter);
    }

    pub async fn remove_filter(&self, filter: &str) {
        let mut filters = self.filters.write().await;
        filters.retain(|f| f != filter);
    }

    pub async fn clear_transactions(&self) {
        self.transactions.write().await.clear();
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    pub async fn stop(&self) {
        *self.is_running.write().await = false;
        
        // 恢复系统代理设置
        self.restore_system_proxy().await;
    }

    async fn restore_system_proxy(&self) {
        info!("Restoring system proxy settings...");
        
        #[cfg(target_os = "macos")]
        {
            self.restore_macos_proxy().await;
        }
        
        #[cfg(target_os = "windows")]
        {
            self.restore_windows_proxy().await;
        }
        
        #[cfg(target_os = "linux")]
        {
            self.restore_linux_proxy().await;
        }
    }

    #[cfg(target_os = "macos")]
    async fn restore_macos_proxy(&self) {
        use std::process::Command;
        
        let get_services = Command::new("networksetup")
            .args(&["-listallnetworkservices"])
            .output();
            
        if let Ok(output) = get_services {
            let services = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = services.lines().skip(1).collect();
            
            for service in lines {
                let service = service.trim();
                if !service.is_empty() {
                    // 关闭 HTTP 代理
                    let _disable_http = Command::new("networksetup")
                        .args(&["-setwebproxystate", service, "off"])
                        .output();
                        
                    // 关闭 HTTPS 代理
                    let _disable_https = Command::new("networksetup")
                        .args(&["-setsecurewebproxystate", service, "off"])
                        .output();
                        
                    info!("Restored proxy settings for network service: {}", service);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    async fn restore_windows_proxy(&self) {
        use std::process::Command;
        
        let script = r#"
            Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings" -Name ProxyEnable -Value 0
        "#;
        
        let _result = Command::new("powershell")
            .args(&["-Command", script])
            .output();
            
        info!("Windows proxy settings restored");
    }

    #[cfg(target_os = "linux")]
    async fn restore_linux_proxy(&self) {
        use std::process::Command;
        
        let _result = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "none"])
            .output();
            
        info!("Linux proxy settings restored");
    }

    // 搜索功能
    pub async fn search_transactions(&self, filter: SearchFilter) -> Vec<HttpTransaction> {
        let transactions = self.transactions.read().await;
        transactions
            .iter()
            .filter(|t| {
                let matches_keyword = filter.keyword.is_empty() || 
                    t.request.url.contains(&filter.keyword) ||
                    t.request.method.contains(&filter.keyword);
                
                let matches_method = filter.method.as_ref()
                    .map(|m| t.request.method == *m)
                    .unwrap_or(true);
                
                let matches_status = filter.status
                    .map(|s| t.response.as_ref().map(|r| r.status == s).unwrap_or(false))
                    .unwrap_or(true);
                
                let matches_domain = filter.domain.as_ref()
                    .map(|d| t.request.url.contains(d))
                    .unwrap_or(true);
                
                matches_keyword && matches_method && matches_status && matches_domain
            })
            .cloned()
            .collect()
    }

    // 收藏功能
    pub async fn toggle_favorite(&self, transaction_id: &str) -> bool {
        let mut transactions = self.transactions.write().await;
        if let Some(transaction) = transactions.iter_mut().find(|t| t.id == transaction_id) {
            transaction.is_favorite = !transaction.is_favorite;
            transaction.is_favorite
        } else {
            false
        }
    }

    pub async fn get_favorites(&self) -> Vec<HttpTransaction> {
        let transactions = self.transactions.read().await;
        transactions
            .iter()
            .filter(|t| t.is_favorite)
            .cloned()
            .collect()
    }

    // 规则管理
    pub async fn add_rule(&self, rule: RequestRule) {
        self.rules.write().await.push(rule);
    }

    pub async fn remove_rule(&self, rule_id: &str) {
        let mut rules = self.rules.write().await;
        rules.retain(|r| r.id != rule_id);
    }

    pub async fn get_rules(&self) -> Vec<RequestRule> {
        self.rules.read().await.clone()
    }

    // HAR 导出
    pub async fn export_har(&self) -> String {
        let transactions = self.transactions.read().await;
        let har_entries: Vec<serde_json::Value> = transactions
            .iter()
            .map(|t| {
                json!({
                    "startedDateTime": t.request.timestamp.to_rfc3339(),
                    "time": t.duration.map(|d| d.as_millis() as u64).unwrap_or(0),
                    "request": {
                        "method": t.request.method,
                        "url": t.request.url,
                        "headers": t.request.headers.iter().map(|(k, v)| json!({
                            "name": k,
                            "value": v
                        })).collect::<Vec<_>>(),
                        "bodySize": t.request.body.len()
                    },
                    "response": t.response.as_ref().map(|r| json!({
                        "status": r.status,
                        "headers": r.headers.iter().map(|(k, v)| json!({
                            "name": k,
                            "value": v
                        })).collect::<Vec<_>>(),
                        "bodySize": r.body.len()
                    }))
                })
            })
            .collect();

        let har = json!({
            "log": {
                "version": "1.2",
                "creator": {
                    "name": "PacketMind AI",
                    "version": "1.0.0"
                },
                "entries": har_entries
            }
        });

        serde_json::to_string_pretty(&har).unwrap_or_default()
    }

    // 编码工具
    pub fn encode_base64(input: &str) -> String {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(input.as_bytes())
    }

    pub fn decode_base64(input: &str) -> Result<String, String> {
        use base64::{Engine as _, engine::general_purpose};
        let decoded = general_purpose::STANDARD.decode(input.as_bytes())
            .map_err(|e| e.to_string())?;
        String::from_utf8(decoded).map_err(|e| e.to_string())
    }

    pub fn encode_url(input: &str) -> String {
        urlencoding::encode(input).to_string()
    }

    pub fn decode_url(input: &str) -> String {
        urlencoding::decode(input).unwrap_or_default().to_string()
    }
}
