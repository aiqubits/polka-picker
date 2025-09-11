import { useState } from 'react'
import './App.css'
import MarketplaceContent from './components/MarketplaceContent'
import ProfileContent from './components/ProfileContent'
import ChatbotContent from './components/ChatbotContent'
import LogStream from './components/LogStream'
import type { Product } from './types'

// 简化的任务类型
interface Task {
  id: string
  name: string
  status: 'running' | 'idle' | 'error'
  installed: string
  runs: number
  lastRun: string
}

// 模拟任务数据
const mockTasks: Task[] = [
  {
    id: '1',
    name: 'Data Automation Pipeline',
    status: 'running',
    installed: '240128',
    runs: 128,
    lastRun: '240301'
  },
  {
    id: '2',
    name: 'Customer Data Processing',
    status: 'idle',
    installed: '240205',
    runs: 84,
    lastRun: '240228'
  },
  {
    id: '3',
    name: 'Server Monitoring Agent',
    status: 'running',
    installed: '240112',
    runs: 312,
    lastRun: '240301'
  },
  {
    id: '4',
    name: 'Backup System Task',
    status: 'error',
    installed: '240220',
    runs: 28,
    lastRun: '240229'
  },
  {
    id: '5',
    name: 'File Conversion Service',
    status: 'idle',
    installed: '240125',
    runs: 95,
    lastRun: '240227'
  },
  {
    id: '6',
    name: 'API Integration Worker',
    status: 'running',
    installed: '240218',
    runs: 43,
    lastRun: '240301'
  }
]

function App() {
  const [tasks] = useState<Task[]>(mockTasks)
  const [activeFilter, setActiveFilter] = useState<'all' | 'running' | 'idle' | 'error'>('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [activePage, setActivePage] = useState<'home' | 'chatbot' | 'marketplace' | 'profile'>('home')

  // 模拟Marketplace产品数据
  const mockProducts: Product[] = [
    {
      id: '1',
      name: 'Data Processing Tool',
      description: 'ETL tool. Transform, validate and load data with ease.',
      category: 'Tools',
      developer: 'DataTeam Inc.',
      isPremium: true,
      rating: { score: 4.5, count: 128 },
      installs: 3450,
      actionText: 'Get'
    },
    {
      id: '2',
      name: 'Server Monitoring',
      description: 'Real-time server monitoring with alerts and detailed performance metrics.',
      category: 'Popular',
      developer: 'ServerPro Systems',
      isPremium: false,
      rating: { score: 4.8, count: 312 },
      installs: 8250,
      actionText: 'Install'
    },
    {
      id: '3',
      name: 'API Integration Helper',
      description: 'Simplify API integrations with built-in connectors and templates for popular services.',
      category: 'Tools',
      developer: 'DevToolkit Labs',
      isPremium: true,
      rating: { score: 4.2, count: 89 },
      installs: 1875,
      actionText: 'Get'
    },
    {
      id: '4',
      name: 'Backup System Plugin',
      description: 'Automated backup solution with encryption, versioning, and easy restore functionality.',
      category: 'New',
      developer: 'SecureData Systems',
      isPremium: false,
      rating: { score: 4.7, count: 56 },
      installs: 2140,
      actionText: 'Install'
    },
    {
      id: '5',
      name: 'File Conversion Service',
      description: 'Convert between document formats with high-quality output and batch processing capabilities.',
      category: 'Tools',
      developer: 'FileTools Inc.',
      isPremium: true,
      rating: { score: 4.3, count: 147 },
      installs: 4320,
      actionText: 'Get'
    },
    {
      id: '6',
      name: 'AI Assistant Worker',
      description: 'AI-powered assistant for task automation, data analysis and intelligent recommendations.',
      category: 'Premium',
      developer: 'Alnova Tech',
      isPremium: true,
      rating: { score: 4.6, count: 203 },
      installs: 6790,
      actionText: 'Get'
    }
  ]

  const filteredTasks = tasks.filter(task => {
    const matchesFilter = activeFilter === 'all' || task.status === activeFilter
    const matchesSearch = task.name.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesFilter && matchesSearch
  })

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return '#10b981'
      case 'idle': return '#3b82f6'
      case 'error': return '#ef4444'
      default: return '#6b7280'
    }
  }

  return (
    <div className="app">
      {/* Top Header */}
      <div className="top-header">
        <div className="header-left">
          <div className="logo">
            <span className="logo-text">OpenPick</span>
          </div>
          <nav className="nav-menu">
            <button 
              className={`nav-item ${activePage === 'home' ? 'active' : ''}`}
              onClick={() => setActivePage('home')}
            >
              <span className="nav-icon">🏠</span>
              <span className="nav-text">Home</span>
            </button>
            <button 
              className={`nav-item ${activePage === 'chatbot' ? 'active' : ''}`}
              onClick={() => setActivePage('chatbot')}
            >
              <span className="nav-icon">🤖</span>
              <span className="nav-text">Chatbot</span>
            </button>
            <button 
              className={`nav-item ${activePage === 'marketplace' ? 'active' : ''}`}
              onClick={() => setActivePage('marketplace')}
            >
              <span className="nav-icon">🛒</span>
              <span className="nav-text">Marketplace</span>
            </button>
            <button 
              className={`nav-item ${activePage === 'profile' ? 'active' : ''}`}
              onClick={() => setActivePage('profile')}
            >
              <span className="nav-icon">👤</span>
              <span className="nav-text">Profile</span>
            </button>
          </nav>
        </div>
        <div className="header-right">
          <div className="user-info">
            <div className="user-avatar">De</div>
            <div className="user-details">
              <span className="username">Deporter</span>
              <div className="user-stats">
                <span className="free-badge">Free:10</span>
                <span className="premium-badge">Premium:28</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="app-main">
        {/* Sidebar */}
        <div className="sidebar">
        
        <div className="post-section">
          <div className="section-header">
            <span className="section-icon">📝</span>
            <span className="section-title">Post</span>
          </div>
          <div className="post-item">
            <div className="post-meta">
              <span className="post-id">240301</span>
              <span className="post-action">Update</span>
            </div>
            <div className="post-title">New Features Release</div>
            <div className="post-subtitle">Read more</div>
          </div>
        </div>

        <div className="support-section">
          <div className="section-header">
            <span className="section-icon">🛠️</span>
            <span className="section-title">Support</span>
          </div>
          <div className="qr-code">
            <div className="qr-placeholder">QR</div>
          </div>
          <div className="support-contact">
            <span className="contact-icon">📧</span>
            <span className="contact-text">Contact Support</span>
          </div>
        </div>

        </div>

        {/* Main Content */}
        <div className="main-content">
          {activePage === 'home' ? (
            <>
              <div className="content-header">
                <h1 className="page-title">My Tasks</h1>
                <div className="header-controls">
                  <div className="filter-tabs">
                    {(['all', 'running', 'idle', 'error'] as const).map(filter => (
                      <button
                        key={filter}
                        className={`filter-tab ${activeFilter === filter ? 'active' : ''}`}
                        onClick={() => setActiveFilter(filter)}
                      >
                        {filter.charAt(0).toUpperCase() + filter.slice(1)}
                      </button>
                    ))}
                  </div>
                  <div className="search-container">
                    <input
                      type="text"
                      placeholder="Search tasks..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="search-input"
                    />
                    <span className="search-icon">🔍</span>
                  </div>
                </div>
              </div>

              <div className="task-grid">
                {filteredTasks.map(task => (
                  <div key={task.id} className="task-card" data-status={task.status}>
                    <div className="task-header">
                      <h3 className="task-name">{task.name}</h3>
                      <button className="task-menu">⋮</button>
                    </div>
                    
                    <div className="task-info">
                      <div className="info-row">
                        <span className="info-icon">📅</span>
                        <span className="info-label">Installed:</span>
                        <span className="info-value">{task.installed}</span>
                      </div>
                      <div className="info-row">
                        <span className="info-icon">▶️</span>
                        <span className="info-label">Runs:</span>
                        <span className="info-value">{task.runs}</span>
                      </div>
                    </div>

                    <div className="task-status">
                      <div className="status-indicator">
                        <span 
                          className="status-dot"
                          style={{ color: getStatusColor(task.status) }}
                        >
                          ●
                        </span>
                        <span className="status-text" style={{ color: getStatusColor(task.status) }}>
                          {task.status.charAt(0).toUpperCase() + task.status.slice(1)}
                        </span>
                      </div>
                      <div className="last-run">
                        <span className="last-run-icon">🕒</span>
                        <span className="last-run-label">Last:</span>
                        <span className="last-run-value">{task.lastRun}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              <button className="add-button">
                <span className="add-icon">+</span>
              </button>
            </>
          ) : activePage === 'chatbot' ? (
            <ChatbotContent />
          ) : activePage === 'marketplace' ? (
            <MarketplaceContent products={mockProducts} />
          ) : (
            <ProfileContent />
          )}
        </div>
      </div>

      {/* Bottom Log Stream */}
      <LogStream />
    </div>
  )
}

export default App
