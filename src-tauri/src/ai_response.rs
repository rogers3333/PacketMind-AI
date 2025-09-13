use crate::proxy::{HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponseConfig {
    pub enable_ai_responses: bool,
    pub response_type: ResponseType,
    pub content_template: Option<String>,
    pub ai_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    Mock,
    Enhanced,
    ErrorSimulation,
    Custom,
}

pub struct AIResponseGenerator {
    config: AIResponseConfig,
    templates: ResponseTemplates,
}

#[derive(Debug, Clone)]
pub struct ResponseTemplates {
    pub mock_responses: HashMap<String, String>,
    pub error_responses: HashMap<u16, String>,
}

impl AIResponseGenerator {
    pub fn new(config: AIResponseConfig) -> Self {
        let templates = ResponseTemplates {
            mock_responses: Self::load_mock_templates(),
            error_responses: Self::load_error_templates(),
        };
        
        Self { config, templates }
    }

    pub async fn generate_response(&self, request: &HttpRequest) -> Result<HttpResponse> {
        match self.config.response_type {
            ResponseType::Mock => self.generate_mock_response(request).await,
            ResponseType::Enhanced => self.generate_enhanced_response(request).await,
            ResponseType::ErrorSimulation => self.generate_error_response(request).await,
            ResponseType::Custom => self.generate_custom_response(request).await,
        }
    }

    async fn generate_mock_response(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let content_type = self.detect_content_type(request);
        let mock_data = self.generate_mock_data(request, &content_type).await;
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type);
        headers.insert("Cache-Control".to_string(), "no-cache".to_string());
        
        Ok(HttpResponse {
            status: 200,
            headers,
            body: mock_data.into_bytes(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn generate_enhanced_response(&self, request: &HttpRequest) -> Result<HttpResponse> {
        // 使用 AI 增强响应内容
        let enhanced_content = self.enhance_with_ai(request).await?;
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Enhanced-By".to_string(), "PacketMind AI".to_string());
        
        Ok(HttpResponse {
            status: 200,
            headers,
            body: enhanced_content.into_bytes(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn generate_error_response(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let error_code = self.select_error_code(request);
        let error_message = self.templates.error_responses.get(&error_code)
            .unwrap_or(&"Unknown error".to_string())
            .clone();
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        let error_body = serde_json::json!({
            "error": {
                "code": error_code,
                "message": error_message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "request_id": uuid::Uuid::new_v4().to_string(),
            }
        });
        
        Ok(HttpResponse {
            status: error_code,
            headers,
            body: serde_json::to_string(&error_body)?.into_bytes(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn generate_custom_response(&self, request: &HttpRequest) -> Result<HttpResponse> {
        if let Some(template) = &self.config.content_template {
            let custom_content = self.render_template(template, request).await?;
            
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            
            Ok(HttpResponse {
                status: 200,
                headers,
                body: custom_content.into_bytes(),
                timestamp: chrono::Utc::now(),
            })
        } else {
            self.generate_mock_response(request).await
        }
    }

    fn detect_content_type(&self, request: &HttpRequest) -> String {
        // 根据 URL 和请求头检测内容类型
        if request.url.contains(".json") || request.url.contains("/api/") {
            "application/json".to_string()
        } else if request.url.contains(".xml") {
            "application/xml".to_string()
        } else if request.url.contains(".html") {
            "text/html".to_string()
        } else {
            "application/json".to_string()
        }
    }

    async fn generate_mock_data(&self, request: &HttpRequest, content_type: &str) -> String {
        match content_type {
            "application/json" => self.generate_json_mock(request).await,
            "application/xml" => self.generate_xml_mock(request).await,
            "text/html" => self.generate_html_mock(request).await,
            _ => self.generate_json_mock(request).await,
        }
    }

    async fn generate_json_mock(&self, request: &HttpRequest) -> String {
        let endpoint = self.extract_endpoint(request);
        
        match endpoint.as_str() {
            "users" => serde_json::json!({
                "users": [
                    {
                        "id": 1,
                        "name": "John Doe",
                        "email": "john@example.com",
                        "created_at": chrono::Utc::now().to_rfc3339(),
                    },
                    {
                        "id": 2,
                        "name": "Jane Smith",
                        "email": "jane@example.com",
                        "created_at": chrono::Utc::now().to_rfc3339(),
                    }
                ],
                "total": 2,
                "page": 1,
                "per_page": 10,
            }).to_string(),
            
            "products" => serde_json::json!({
                "products": [
                    {
                        "id": 1,
                        "name": "Sample Product",
                        "price": 99.99,
                        "description": "This is a sample product",
                        "category": "electronics",
                    }
                ],
                "total": 1,
            }).to_string(),
            
            _ => serde_json::json!({
                "message": "Mock response generated by PacketMind AI",
                "endpoint": endpoint,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "request_id": uuid::Uuid::new_v4().to_string(),
                "data": {
                    "sample_field": "sample_value",
                    "count": 1,
                }
            }).to_string(),
        }
    }

    async fn generate_xml_mock(&self, request: &HttpRequest) -> String {
        let endpoint = self.extract_endpoint(request);
        
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<response>
    <message>Mock XML response generated by PacketMind AI</message>
    <endpoint>{}</endpoint>
    <timestamp>{}</timestamp>
    <data>
        <item>
            <id>1</id>
            <name>Sample Item</name>
            <value>sample_value</value>
        </item>
    </data>
</response>"#,
            endpoint,
            chrono::Utc::now().to_rfc3339(),
        )
    }

    async fn generate_html_mock(&self, request: &HttpRequest) -> String {
        let endpoint = self.extract_endpoint(request);
        
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Mock Response - PacketMind AI</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .container {{ max-width: 800px; margin: 0 auto; }}
        .header {{ background: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .content {{ margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Mock HTML Response</h1>
            <p>Generated by PacketMind AI</p>
        </div>
        <div class="content">
            <h2>Endpoint: {}</h2>
            <p>Timestamp: {}</p>
            <p>This is a mock HTML response for testing purposes.</p>
        </div>
    </div>
</body>
</html>"#,
            endpoint,
            chrono::Utc::now().to_rfc3339(),
        )
    }

    async fn enhance_with_ai(&self, request: &HttpRequest) -> Result<String> {
        // 这里可以集成 AI 模型来增强响应内容
        let _prompt = format!(
            "基于以下请求生成一个智能响应：\n方法: {}\nURL: {}\n请生成一个符合 RESTful API 规范的 JSON 响应。",
            request.method,
            request.url,
        );
        
        // 模拟 AI 增强的响应
        Ok(serde_json::json!({
            "ai_enhanced": true,
            "original_request": {
                "method": request.method,
                "url": request.url,
            },
            "enhanced_data": {
                "message": "AI 增强的响应内容",
                "suggestions": [
                    "建议使用缓存优化性能",
                    "考虑添加分页支持",
                    "建议实现数据验证",
                ],
                "predicted_usage": "high",
                "optimization_tips": [
                    "使用 CDN 加速",
                    "启用压缩",
                    "实现缓存策略",
                ],
            },
            "metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "ai_model": self.config.ai_model,
                "confidence": 0.95,
            }
        }).to_string())
    }

    fn select_error_code(&self, request: &HttpRequest) -> u16 {
        // 根据请求特征选择错误码
        if request.url.contains("unauthorized") {
            401
        } else if request.url.contains("forbidden") {
            403
        } else if request.url.contains("notfound") {
            404
        } else if request.url.contains("server") {
            500
        } else {
            400
        }
    }

    async fn render_template(&self, template: &str, request: &HttpRequest) -> Result<String> {
        // 简单的模板渲染
        let rendered = template
            .replace("{{method}}", &request.method)
            .replace("{{url}}", &request.url)
            .replace("{{timestamp}}", &chrono::Utc::now().to_rfc3339())
            .replace("{{request_id}}", &uuid::Uuid::new_v4().to_string());
        
        Ok(rendered)
    }

    fn extract_endpoint(&self, request: &HttpRequest) -> String {
        request.url
            .split('/')
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or("default")
            .to_string()
    }

    fn load_mock_templates() -> HashMap<String, String> {
        let mut templates = HashMap::new();
        templates.insert("users".to_string(), "users_template".to_string());
        templates.insert("products".to_string(), "products_template".to_string());
        templates.insert("orders".to_string(), "orders_template".to_string());
        templates
    }

    fn load_error_templates() -> HashMap<u16, String> {
        let mut templates = HashMap::new();
        templates.insert(400, "Bad Request".to_string());
        templates.insert(401, "Unauthorized".to_string());
        templates.insert(403, "Forbidden".to_string());
        templates.insert(404, "Not Found".to_string());
        templates.insert(500, "Internal Server Error".to_string());
        templates
    }
}

// AI 驱动的智能路由
pub struct AIRouter {
    response_generator: AIResponseGenerator,
    routing_rules: Vec<RoutingRule>,
}

#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub pattern: String,
    pub response_type: ResponseType,
    pub priority: u32,
}

impl AIRouter {
    pub fn new(response_generator: AIResponseGenerator) -> Self {
        Self {
            response_generator,
            routing_rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: RoutingRule) {
        self.routing_rules.push(rule);
        self.routing_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub async fn route_request(&self, request: &HttpRequest) -> Result<HttpResponse> {
        // 查找匹配的路由规则
        for rule in &self.routing_rules {
            if self.matches_pattern(&request.url, &rule.pattern) {
                let mut config = self.response_generator.config.clone();
                config.response_type = rule.response_type.clone();
                let generator = AIResponseGenerator::new(config);
                return generator.generate_response(request).await;
            }
        }
        
        // 默认响应
        self.response_generator.generate_response(request).await
    }

    fn matches_pattern(&self, url: &str, pattern: &str) -> bool {
        // 简单的模式匹配
        if pattern.starts_with('/') && pattern.ends_with('/') {
            // 正则表达式模式
            let regex_pattern = &pattern[1..pattern.len()-1];
            regex::Regex::new(regex_pattern)
                .map(|re| re.is_match(url))
                .unwrap_or(false)
        } else {
            // 简单字符串匹配
            url.contains(pattern)
        }
    }
}
