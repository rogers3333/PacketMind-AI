use crate::proxy::{HttpTransaction, HttpRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    pub security_risk: SecurityRisk,
    pub performance_insights: Vec<String>,
    pub optimization_suggestions: Vec<String>,
    pub anomaly_detection: Vec<String>,
    pub api_patterns: Vec<ApiPattern>,
    pub data_flow_analysis: DataFlowAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRisk {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPattern {
    pub pattern_type: String,
    pub confidence: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowAnalysis {
    pub data_types: Vec<String>,
    pub sensitive_data_detected: bool,
    pub data_flow_direction: String,
    pub compliance_issues: Vec<String>,
}

pub struct AIAnalyzer {
    api_key: Option<String>,
    model: AIModel,
}

#[derive(Debug, Clone)]
pub enum AIModel {
    OpenAI { model: String },
    Anthropic { model: String },
    Local { model_path: String },
}

impl AIAnalyzer {
    pub fn new(api_key: Option<String>, model: AIModel) -> Self {
        Self { api_key, model }
    }

    pub async fn analyze_transaction(&self, transaction: &HttpTransaction) -> Result<AIAnalysisResult> {
        match &self.model {
            AIModel::OpenAI { model } => self.analyze_with_openai(transaction, model).await,
            AIModel::Anthropic { model } => self.analyze_with_anthropic(transaction, model).await,
            AIModel::Local { model_path } => self.analyze_with_local_model(transaction, model_path).await,
        }
    }

    async fn analyze_with_openai(&self, transaction: &HttpTransaction, _model: &str) -> Result<AIAnalysisResult> {
        let _prompt = self.build_analysis_prompt(transaction);
        
        // 这里需要集成 OpenAI API
        // 暂时返回模拟结果
        Ok(AIAnalysisResult {
            security_risk: SecurityRisk::Medium,
            performance_insights: vec![
                "请求响应时间较长，建议优化".to_string(),
                "可以考虑启用缓存".to_string(),
            ],
            optimization_suggestions: vec![
                "使用 CDN 加速静态资源".to_string(),
                "启用 Gzip 压缩".to_string(),
            ],
            anomaly_detection: vec![
                "检测到异常的请求频率".to_string(),
            ],
            api_patterns: vec![
                ApiPattern {
                    pattern_type: "REST API".to_string(),
                    confidence: 0.95,
                    description: "标准的 RESTful API 调用".to_string(),
                },
            ],
            data_flow_analysis: DataFlowAnalysis {
                data_types: vec!["JSON".to_string(), "User Data".to_string()],
                sensitive_data_detected: false,
                data_flow_direction: "Client to Server".to_string(),
                compliance_issues: vec![],
            },
        })
    }

    async fn analyze_with_anthropic(&self, transaction: &HttpTransaction, model: &str) -> Result<AIAnalysisResult> {
        // 集成 Anthropic Claude API
        self.analyze_with_openai(transaction, model).await
    }

    async fn analyze_with_local_model(&self, transaction: &HttpTransaction, _model_path: &str) -> Result<AIAnalysisResult> {
        // 集成本地模型 (如 ONNX, TensorFlow Lite)
        self.analyze_with_openai(transaction, "local").await
    }

    fn build_analysis_prompt(&self, transaction: &HttpTransaction) -> String {
        format!(
            r#"
分析以下 HTTP 请求并提供详细的安全、性能和优化建议：

请求信息：
- 方法: {}
- URL: {}
- 状态码: {}
- 响应时间: {}ms
- 请求头: {:?}
- 响应头: {:?}

请从以下角度进行分析：
1. 安全风险评估
2. 性能优化建议
3. 异常检测
4. API 模式识别
5. 数据流分析
6. 合规性检查
"#,
            transaction.request.method,
            transaction.request.url,
            transaction.response.as_ref().map(|r| r.status).unwrap_or(0),
            transaction.duration.map(|d| d.as_millis()).unwrap_or(0),
            transaction.request.headers,
            transaction.response.as_ref().map(|r| &r.headers),
        )
    }

    pub async fn batch_analyze(&self, transactions: &[HttpTransaction]) -> Result<Vec<AIAnalysisResult>> {
        let mut results = Vec::new();
        for transaction in transactions {
            results.push(self.analyze_transaction(transaction).await?);
        }
        Ok(results)
    }

    pub async fn detect_anomalies(&self, transactions: &[HttpTransaction]) -> Result<Vec<String>> {
        // 使用 AI 检测异常模式
        let mut anomalies = Vec::new();
        
        // 检测异常请求频率
        let mut request_counts: HashMap<String, usize> = HashMap::new();
        for transaction in transactions {
            let domain = extract_domain(&transaction.request.url);
            *request_counts.entry(domain).or_insert(0) += 1;
        }

        for (domain, count) in request_counts {
            if count > 100 {
                anomalies.push(format!("域名 {} 请求频率异常: {} 次", domain, count));
            }
        }

        // 检测异常状态码
        for transaction in transactions {
            if let Some(response) = &transaction.response {
                if response.status >= 500 {
                    anomalies.push(format!("检测到服务器错误: {} - {}", response.status, transaction.request.url));
                }
            }
        }

        Ok(anomalies)
    }

    pub async fn suggest_optimizations(&self, transactions: &[HttpTransaction]) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        // 分析响应时间
        let avg_response_time: u64 = transactions
            .iter()
            .filter_map(|t| t.duration.map(|d| d.as_millis() as u64))
            .sum::<u64>() / transactions.len().max(1) as u64;

        if avg_response_time > 1000 {
            suggestions.push("平均响应时间超过1秒，建议优化后端性能".to_string());
        }

        // 分析缓存使用情况
        let cache_hits = transactions
            .iter()
            .filter(|t| {
                t.response.as_ref()
                    .map(|r| r.headers.get("cache-control").is_some())
                    .unwrap_or(false)
            })
            .count();

        if cache_hits < transactions.len() / 2 {
            suggestions.push("缓存使用率较低，建议增加缓存策略".to_string());
        }

        Ok(suggestions)
    }
}

fn extract_domain(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string()
}

// AI 驱动的安全检测
pub struct SecurityAnalyzer {
    ai_analyzer: AIAnalyzer,
}

impl SecurityAnalyzer {
    pub fn new(ai_analyzer: AIAnalyzer) -> Self {
        Self { ai_analyzer }
    }

    pub async fn detect_vulnerabilities(&self, transaction: &HttpTransaction) -> Result<Vec<String>> {
        let mut vulnerabilities = Vec::new();
        
        // SQL 注入检测
        if self.detect_sql_injection(&transaction.request).await {
            vulnerabilities.push("潜在的 SQL 注入攻击".to_string());
        }

        // XSS 检测
        if self.detect_xss(&transaction.request).await {
            vulnerabilities.push("潜在的 XSS 攻击".to_string());
        }

        // 敏感信息泄露检测
        if self.detect_sensitive_data(&transaction).await {
            vulnerabilities.push("检测到敏感信息泄露".to_string());
        }

        Ok(vulnerabilities)
    }

    async fn detect_sql_injection(&self, request: &HttpRequest) -> bool {
        let sql_patterns = [
            "SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "UNION",
            "OR 1=1", "OR '1'='1", "'; DROP", "'; --",
        ];

        let url_lower = request.url.to_lowercase();
        let body_lower = String::from_utf8_lossy(&request.body).to_lowercase();

        sql_patterns.iter().any(|pattern| {
            url_lower.contains(&pattern.to_lowercase()) || body_lower.contains(&pattern.to_lowercase())
        })
    }

    async fn detect_xss(&self, request: &HttpRequest) -> bool {
        let xss_patterns = [
            "<script>", "javascript:", "onload=", "onerror=", "onclick=",
            "alert(", "confirm(", "prompt(",
        ];

        let url_lower = request.url.to_lowercase();
        let body_lower = String::from_utf8_lossy(&request.body).to_lowercase();

        xss_patterns.iter().any(|pattern| {
            url_lower.contains(&pattern.to_lowercase()) || body_lower.contains(&pattern.to_lowercase())
        })
    }

    async fn detect_sensitive_data(&self, transaction: &HttpTransaction) -> bool {
        let sensitive_patterns = [
            "password", "token", "key", "secret", "api_key", "auth",
            "credit_card", "ssn", "social_security",
        ];

        let url_lower = transaction.request.url.to_lowercase();
        let body_lower = String::from_utf8_lossy(&transaction.request.body).to_lowercase();

        sensitive_patterns.iter().any(|pattern| {
            url_lower.contains(pattern) || body_lower.contains(pattern)
        })
    }
}
