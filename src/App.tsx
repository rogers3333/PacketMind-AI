import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Play, Square, Trash2, Eye, EyeOff, Brain } from 'lucide-react'
import { AIAnalysis } from './components/AIAnalysis'

interface Transaction {
  id: string
  method: string
  url: string
  status?: number
  duration?: number
  timestamp: string
  tags?: string[]
}

function App() {
  const [isRunning, setIsRunning] = useState(false)
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [filters, setFilters] = useState<string[]>([])
  const [newFilter, setNewFilter] = useState('')
  const [showFilters, setShowFilters] = useState(false)
  const [activeTab, setActiveTab] = useState<'transactions' | 'ai'>('transactions')

  useEffect(() => {
    checkStatus()
    const interval = setInterval(checkStatus, 1000)
    return () => clearInterval(interval)
  }, [])

  useEffect(() => {
    if (isRunning) {
      loadTransactions()
      const interval = setInterval(loadTransactions, 2000)
      return () => clearInterval(interval)
    }
  }, [isRunning])

  const checkStatus = async () => {
    try {
      const running = await invoke<boolean>('is_proxy_running')
      setIsRunning(running)
    } catch (error) {
      console.error('Failed to check status:', error)
    }
  }

  const loadTransactions = async () => {
    try {
      const data = await invoke<Transaction[]>('get_transactions')
      setTransactions(data)
    } catch (error) {
      console.error('Failed to load transactions:', error)
    }
  }

  const startProxy = async () => {
    try {
      await invoke('start_proxy')
      setIsRunning(true)
    } catch (error) {
      console.error('Failed to start proxy:', error)
    }
  }

  const stopProxy = async () => {
    try {
      await invoke('stop_proxy')
      setIsRunning(false)
    } catch (error) {
      console.error('Failed to stop proxy:', error)
    }
  }

  const clearTransactions = async () => {
    try {
      await invoke('clear_transactions')
      setTransactions([])
    } catch (error) {
      console.error('Failed to clear transactions:', error)
    }
  }

  const addFilter = async () => {
    if (!newFilter.trim()) return
    
    try {
      await invoke('add_filter', { filterReq: { filter: newFilter.trim() } })
      setFilters([...filters, newFilter.trim()])
      setNewFilter('')
    } catch (error) {
      console.error('Failed to add filter:', error)
    }
  }

  const removeFilter = async (filter: string) => {
    try {
      await invoke('remove_filter', { filter })
      setFilters(filters.filter(f => f !== filter))
    } catch (error) {
      console.error('Failed to remove filter:', error)
    }
  }

  const getStatusColor = (status?: number) => {
    if (!status) return 'text-gray-500'
    if (status >= 200 && status < 300) return 'text-green-500'
    if (status >= 300 && status < 400) return 'text-blue-500'
    if (status >= 400 && status < 500) return 'text-yellow-500'
    return 'text-red-500'
  }

  const getMethodColor = (method: string) => {
    switch (method.toUpperCase()) {
      case 'GET': return 'bg-green-100 text-green-800'
      case 'POST': return 'bg-blue-100 text-blue-800'
      case 'PUT': return 'bg-yellow-100 text-yellow-800'
      case 'DELETE': return 'bg-red-100 text-red-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            PacketMind AI
          </h1>
          <p className="text-gray-600">
            HTTP(S) Traffic Capture Tool - 基于 Tauri 和 Rust 构建，集成 AI 智能分析
          </p>
          {isRunning && (
            <div className="mt-2 p-3 bg-blue-50 border border-blue-200 rounded-md">
              <p className="text-sm text-blue-800">
                🎉 <strong>自动代理已启用！</strong> 现在所有网络流量都会自动通过 PacketMind AI 进行捕获和分析。
                您可以正常浏览网页，所有 HTTP 请求都会被自动记录。
              </p>
            </div>
          )}
        </div>

        {/* Tab Navigation */}
        <div className="mb-6">
          <nav className="flex space-x-8">
            <button
              onClick={() => setActiveTab('transactions')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'transactions'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              请求记录
            </button>
            <button
              onClick={() => setActiveTab('ai')}
              className={`py-2 px-1 border-b-2 font-medium text-sm flex items-center gap-2 ${
                activeTab === 'ai'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <Brain className="w-4 h-4" />
              AI 智能分析
            </button>
          </nav>
        </div>

        {/* Control Panel */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-gray-900">控制面板</h2>
            <div className="flex items-center space-x-2">
              <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                isRunning ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
              }`}>
                {isRunning ? '运行中' : '已停止'}
              </span>
              {isRunning && (
                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                  自动代理已启用
                </span>
              )}
            </div>
          </div>

          <div className="flex items-center space-x-4">
            <button
              onClick={isRunning ? stopProxy : startProxy}
              className={`inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white ${
                isRunning 
                  ? 'bg-red-600 hover:bg-red-700' 
                  : 'bg-green-600 hover:bg-green-700'
              }`}
            >
              {isRunning ? <Square className="w-4 h-4 mr-2" /> : <Play className="w-4 h-4 mr-2" />}
              {isRunning ? '停止代理' : '启动代理'}
            </button>

            <button
              onClick={clearTransactions}
              className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
            >
              <Trash2 className="w-4 h-4 mr-2" />
              清空记录
            </button>

            <button
              onClick={() => setShowFilters(!showFilters)}
              className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
            >
              {showFilters ? <EyeOff className="w-4 h-4 mr-2" /> : <Eye className="w-4 h-4 mr-2" />}
              域名过滤
            </button>
          </div>

          {/* Filter Panel */}
          {showFilters && (
            <div className="mt-4 p-4 bg-gray-50 rounded-md">
              <h3 className="text-sm font-medium text-gray-900 mb-2">域名过滤</h3>
              <div className="flex items-center space-x-2 mb-3">
                <input
                  type="text"
                  value={newFilter}
                  onChange={(e) => setNewFilter(e.target.value)}
                  placeholder="输入域名关键词..."
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  onKeyPress={(e) => e.key === 'Enter' && addFilter()}
                />
                <button
                  onClick={addFilter}
                  className="px-3 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700"
                >
                  添加
                </button>
              </div>
              <div className="flex flex-wrap gap-2">
                {filters.map((filter, index) => (
                  <span
                    key={index}
                    className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
                  >
                    {filter}
                    <button
                      onClick={() => removeFilter(filter)}
                      className="ml-1 text-blue-600 hover:text-blue-800"
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Content based on active tab */}
        {activeTab === 'transactions' ? (
          <>
            {/* Transactions Table */}
            <div className="bg-white rounded-lg shadow-sm border border-gray-200">
              <div className="px-6 py-4 border-b border-gray-200">
                <h2 className="text-xl font-semibold text-gray-900">
                  请求记录 ({transactions.length})
                </h2>
              </div>
              
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        方法
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        URL
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        状态
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        耗时
                      </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    时间
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    状态
                  </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {transactions.length === 0 ? (
                      <tr>
                        <td colSpan={6} className="px-6 py-4 text-center text-gray-500">
                          暂无请求记录
                        </td>
                      </tr>
                    ) : (
                      transactions.map((transaction) => (
                        <tr key={transaction.id} className={`hover:bg-gray-50 ${transaction.tags?.includes('filtered') ? 'bg-yellow-50' : ''}`}>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getMethodColor(transaction.method)}`}>
                              {transaction.method}
                            </span>
                          </td>
                          <td className="px-6 py-4 text-sm text-gray-900 max-w-md truncate">
                            <div className="flex items-center gap-2">
                              {transaction.url}
                              {transaction.tags?.includes('filtered') && (
                                <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-800">
                                  已过滤
                                </span>
                              )}
                            </div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className={`text-sm font-medium ${getStatusColor(transaction.status)}`}>
                              {transaction.status || '-'}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {transaction.duration ? `${transaction.duration}ms` : '-'}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {new Date(transaction.timestamp).toLocaleString()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            {transaction.tags?.includes('filtered') ? (
                              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                                已过滤
                              </span>
                            ) : (
                              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                                正常
                              </span>
                            )}
                          </td>
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          </>
        ) : (
          <AIAnalysis transactions={transactions} />
        )}
      </div>
    </div>
  )
}

export default App
