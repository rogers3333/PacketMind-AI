import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Brain, Shield, Zap, AlertTriangle, TrendingUp, Code } from 'lucide-react'

interface AIAnalysisResult {
  security_risk: 'Low' | 'Medium' | 'High' | 'Critical'
  performance_insights: string[]
  optimization_suggestions: string[]
  anomaly_detection: string[]
  api_patterns: Array<{
    pattern_type: string
    confidence: number
    description: string
  }>
  data_flow_analysis: {
    data_types: string[]
    sensitive_data_detected: boolean
    data_flow_direction: string
    compliance_issues: string[]
  }
}

interface Transaction {
  id: string
  method: string
  url: string
  status?: number
  duration?: number
  timestamp: string
}

interface AIAnalysisProps {
  transactions: Transaction[]
}

export function AIAnalysis({ transactions }: AIAnalysisProps) {
  const [selectedTransaction, setSelectedTransaction] = useState<string>('')
  const [analysisResult, setAnalysisResult] = useState<AIAnalysisResult | null>(null)
  const [vulnerabilities, setVulnerabilities] = useState<string[]>([])
  const [insights, setInsights] = useState<string[]>([])
  const [isAnalyzing, setIsAnalyzing] = useState(false)

  const analyzeTransaction = async () => {
    if (!selectedTransaction) return
    
    setIsAnalyzing(true)
    try {
      const result = await invoke<AIAnalysisResult>('analyze_transaction', {
        transactionId: selectedTransaction
      })
      setAnalysisResult(result)
    } catch (error) {
      console.error('分析失败:', error)
    } finally {
      setIsAnalyzing(false)
    }
  }

  const detectVulnerabilities = async () => {
    if (!selectedTransaction) return
    
    setIsAnalyzing(true)
    try {
      const vulns = await invoke<string[]>('detect_vulnerabilities', {
        transactionId: selectedTransaction
      })
      setVulnerabilities(vulns)
    } catch (error) {
      console.error('漏洞检测失败:', error)
    } finally {
      setIsAnalyzing(false)
    }
  }

  const getAIInsights = async () => {
    setIsAnalyzing(true)
    try {
      const aiInsights = await invoke<string[]>('get_ai_insights')
      setInsights(aiInsights)
    } catch (error) {
      console.error('获取 AI 洞察失败:', error)
    } finally {
      setIsAnalyzing(false)
    }
  }

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'Low': return 'text-green-600 bg-green-100'
      case 'Medium': return 'text-yellow-600 bg-yellow-100'
      case 'High': return 'text-orange-600 bg-orange-100'
      case 'Critical': return 'text-red-600 bg-red-100'
      default: return 'text-gray-600 bg-gray-100'
    }
  }

  return (
    <div className="space-y-6">
      {/* AI 分析控制面板 */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
          <Brain className="w-5 h-5" />
          AI 智能分析
        </h2>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              选择请求进行分析
            </label>
            <select
              value={selectedTransaction}
              onChange={(e) => setSelectedTransaction(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">请选择请求...</option>
              {transactions.map((tx) => (
                <option key={tx.id} value={tx.id}>
                  {tx.method} {tx.url.substring(0, 50)}...
                </option>
              ))}
            </select>
          </div>
          
          <div className="flex flex-col justify-end">
            <button
              onClick={analyzeTransaction}
              disabled={!selectedTransaction || isAnalyzing}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              <Brain className="w-4 h-4" />
              {isAnalyzing ? '分析中...' : '智能分析'}
            </button>
          </div>
          
          <div className="flex flex-col justify-end">
            <button
              onClick={detectVulnerabilities}
              disabled={!selectedTransaction || isAnalyzing}
              className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              <Shield className="w-4 h-4" />
              {isAnalyzing ? '检测中...' : '安全检测'}
            </button>
          </div>
        </div>
        
        <button
          onClick={getAIInsights}
          disabled={isAnalyzing}
          className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <TrendingUp className="w-4 h-4" />
          {isAnalyzing ? '分析中...' : '获取 AI 洞察'}
        </button>
      </div>

      {/* AI 分析结果 */}
      {analysisResult && (
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Brain className="w-5 h-5" />
            AI 分析结果
          </h3>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* 安全风险评估 */}
            <div>
              <h4 className="font-medium mb-2 flex items-center gap-2">
                <Shield className="w-4 h-4" />
                安全风险评估
              </h4>
              <div className={`inline-block px-3 py-1 rounded-full text-sm font-medium ${getRiskColor(analysisResult.security_risk)}`}>
                {analysisResult.security_risk} 风险
              </div>
            </div>

            {/* 性能洞察 */}
            <div>
              <h4 className="font-medium mb-2 flex items-center gap-2">
                <Zap className="w-4 h-4" />
                性能洞察
              </h4>
              <ul className="space-y-1">
                {analysisResult.performance_insights.map((insight, index) => (
                  <li key={index} className="text-sm text-gray-600">• {insight}</li>
                ))}
              </ul>
            </div>

            {/* 优化建议 */}
            <div>
              <h4 className="font-medium mb-2 flex items-center gap-2">
                <TrendingUp className="w-4 h-4" />
                优化建议
              </h4>
              <ul className="space-y-1">
                {analysisResult.optimization_suggestions.map((suggestion, index) => (
                  <li key={index} className="text-sm text-gray-600">• {suggestion}</li>
                ))}
              </ul>
            </div>

            {/* 异常检测 */}
            <div>
              <h4 className="font-medium mb-2 flex items-center gap-2">
                <AlertTriangle className="w-4 h-4" />
                异常检测
              </h4>
              <ul className="space-y-1">
                {analysisResult.anomaly_detection.map((anomaly, index) => (
                  <li key={index} className="text-sm text-red-600">• {anomaly}</li>
                ))}
              </ul>
            </div>

            {/* API 模式识别 */}
            <div className="md:col-span-2">
              <h4 className="font-medium mb-2 flex items-center gap-2">
                <Code className="w-4 h-4" />
                API 模式识别
              </h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {analysisResult.api_patterns.map((pattern, index) => (
                  <div key={index} className="border rounded p-3">
                    <div className="font-medium">{pattern.pattern_type}</div>
                    <div className="text-sm text-gray-600">置信度: {(pattern.confidence * 100).toFixed(1)}%</div>
                    <div className="text-sm text-gray-600 mt-1">{pattern.description}</div>
                  </div>
                ))}
              </div>
            </div>

            {/* 数据流分析 */}
            <div className="md:col-span-2">
              <h4 className="font-medium mb-2">数据流分析</h4>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <div className="text-sm font-medium">数据类型</div>
                  <div className="text-sm text-gray-600">
                    {analysisResult.data_flow_analysis.data_types.join(', ')}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium">敏感数据</div>
                  <div className="text-sm text-gray-600">
                    {analysisResult.data_flow_analysis.sensitive_data_detected ? '检测到' : '未检测到'}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium">数据流向</div>
                  <div className="text-sm text-gray-600">
                    {analysisResult.data_flow_analysis.data_flow_direction}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 安全漏洞检测结果 */}
      {vulnerabilities.length > 0 && (
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2 text-red-600">
            <Shield className="w-5 h-5" />
            安全漏洞检测
          </h3>
          <ul className="space-y-2">
            {vulnerabilities.map((vuln, index) => (
              <li key={index} className="flex items-start gap-2 text-red-600">
                <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0" />
                <span>{vuln}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* AI 洞察结果 */}
      {insights.length > 0 && (
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2 text-purple-600">
            <TrendingUp className="w-5 h-5" />
            AI 智能洞察
          </h3>
          <ul className="space-y-2">
            {insights.map((insight, index) => (
              <li key={index} className="flex items-start gap-2">
                <Brain className="w-4 h-4 mt-0.5 flex-shrink-0 text-purple-600" />
                <span>{insight}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}
