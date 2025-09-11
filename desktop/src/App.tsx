import React, { useState } from 'react'
import './App.css'
import MarketplaceContent from './components/MarketplaceContent'
import ProfileContent from './components/ProfileContent'
import ChatbotContent from './components/ChatbotContent'
import LogStream from './components/LogStream'
import { clientAPI, type ResponseUserInfo } from './client/api'

// 用户类型定义
interface User {
  id: string;
  username: string;
  avatar: string;
  wallet: number;
  premium: number;
}

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
  
  // 登录状态管理
  const [isLoggedIn, setIsLoggedIn] = useState(false)
  const [currentUser, setCurrentUser] = useState<User | null>(null)
  const [showLoginModal, setShowLoginModal] = useState(false)
  const [showSignupModal, setShowSignupModal] = useState(false)



  const filteredTasks = tasks.filter(task => {
    const matchesFilter = activeFilter === 'all' || task.status === activeFilter
    const matchesSearch = task.name.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesFilter && matchesSearch
  })

  // API调用函数
  const handleLogin = async (email: string, password: string) => {
    try {
      const response: ResponseUserInfo = await clientAPI.login(email, password);

      // const status = (userData as { userId: string }).userId
      // const userData = (response as { response: LoginResponse })
      alert(JSON.stringify(response, null, 2));
      console.log('Login response:', response);
      setCurrentUser({
        id: response.user_info.user_id,
        username: response.user_info.user_name || "OpenPick",
        avatar: response.user_info.user_name.substring(0, 2).toUpperCase() || "OP",
        wallet: response.wallet_balance ? Math.round(response.wallet_balance / 1e7) / 1e2 : 0,
        premium: response.user_info.premium_balance || 0,
      });
      setIsLoggedIn(true);
      setShowLoginModal(false);
    } catch (error) {
      console.error('Login error:', error);
      alert('Login failed. ' + (error instanceof Error ? error.message : 'Please try again.'));
    }
  };

  const handleRegister = async (email: string, username: string, password: string) => {
    try {
      await clientAPI.register(email, username, password, 'gen');
      
      // 注册成功后，显示验证邮箱的弹窗
      setShowSignupModal(false);
      setVerifyEmail(email);
      setShowVerifyModal(true);
    } catch (error) {
      console.error('Registration error:', error);
      alert('Registration failed. ' + (error instanceof Error ? error.message : 'Please try again.'));
    }
  };

  const handleVerifyEmail = async (email: string, code: string) => {
    try {
      await clientAPI.verifyEmail(email, code);
      
      setShowVerifyModal(false);
      alert('Email verified successfully! Please login.');
      setShowLoginModal(true);
    } catch (error) {
      console.error('Verification error:', error);
      alert('Verification failed. ' + (error instanceof Error ? error.message : 'Please try again.'));
    }
  };

  const handleLogout = async () => {
    try {
      await clientAPI.logout();
      
      // 清除用户登录状态
      setCurrentUser(null);
      setIsLoggedIn(false);
      setShowLogoutMenu(false);
      alert('Logout successful');
    } catch (error) {
      console.error('Logout error:', error);
      alert('Logout failed. ' + (error instanceof Error ? error.message : 'Please try again.'));
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return '#10b981'
      case 'idle': return '#3b82f6'
      case 'error': return '#ef4444'
      default: return '#6b7280'
    }
  }

  // 状态管理
  const [loginEmail, setLoginEmail] = useState('');
  const [loginPassword, setLoginPassword] = useState('');
  const [signupEmail, setSignupEmail] = useState('');
  const [signupUsername, setSignupUsername] = useState('');
  const [signupPassword, setSignupPassword] = useState('');
  const [verifyEmail, setVerifyEmail] = useState('');
  const [verifyCode, setVerifyCode] = useState('');
  const [showVerifyModal, setShowVerifyModal] = useState(false);
  const [showLogoutMenu, setShowLogoutMenu] = useState(false);

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
          {isLoggedIn ? (
            <div 
              className="user-info"
              onClick={() => setShowLogoutMenu(!showLogoutMenu)}
              onMouseLeave={() => setTimeout(() => setShowLogoutMenu(false), 2000)}
            >
              <div className="user-avatar">{currentUser?.avatar || 'Us'}</div>
              <div className="user-details">
                <span className="username">{currentUser?.username || 'User'}</span>
                <div className="user-stats">
                  <span className="wallet-badge">Wallet:{currentUser?.wallet || 0}</span>
                  <span className="premium-badge">Premium:{currentUser?.premium || 0}</span>
                </div>
              </div>
              {showLogoutMenu && (
                <div className="logout-menu">
                  <button 
                    className="logout-button"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleLogout();
                    }}
                  >
                    Logout
                  </button>
                </div>
              )}
            </div>
          ) : (
            <div className="auth-buttons">
              <button 
                className="login-button"
                onClick={() => setShowLoginModal(true)}
              >
                Login
              </button>
              <button 
                className="signup-button"
                onClick={() => setShowSignupModal(true)}
              >
                Sign Up
              </button>
            </div>
          )}
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
                {/* <h1 className="page-title">My Pickers</h1> */}
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
            <MarketplaceContent />
          ) : (
            <ProfileContent />
          )}
        </div>
      </div>

      {/* Bottom Log Stream */}
      <LogStream />

      {/* Login Modal */}
      {showLoginModal && (
        <div className="modal-overlay" onClick={() => setShowLoginModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Login</h2>
              <button className="modal-close" onClick={() => setShowLoginModal(false)}>×</button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label htmlFor="login-email">Email</label>
                <input
                  type="email"
                  id="login-email"
                  value={loginEmail}
                  onChange={(e) => setLoginEmail(e.target.value)}
                  placeholder="Enter your email"
                />
              </div>
              <div className="form-group">
                <label htmlFor="login-password">Password</label>
                <input
                  type="password"
                  id="login-password"
                  value={loginPassword}
                  onChange={(e) => setLoginPassword(e.target.value)}
                  placeholder="Enter your password"
                />
              </div>
            </div>
            <div className="modal-footer">
              <button 
                className="modal-button secondary"
                onClick={() => setShowLoginModal(false)}
              >
                Cancel
              </button>
              <button 
                className="modal-button primary"
                onClick={() => handleLogin(loginEmail, loginPassword)}
              >
                Login
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Sign Up Modal */}
      {showSignupModal && (
        <div className="modal-overlay" onClick={() => setShowSignupModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Sign Up</h2>
              <button className="modal-close" onClick={() => setShowSignupModal(false)}>×</button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label htmlFor="signup-email">Email</label>
                <input
                  type="email"
                  id="signup-email"
                  value={signupEmail}
                  onChange={(e) => setSignupEmail(e.target.value)}
                  placeholder="Enter your email"
                />
              </div>
              <div className="form-group">
                <label htmlFor="signup-username">Username</label>
                <input
                  type="text"
                  id="signup-username"
                  value={signupUsername}
                  onChange={(e) => setSignupUsername(e.target.value)}
                  placeholder="Choose a username"
                />
              </div>
              <div className="form-group">
                <label htmlFor="signup-password">Password</label>
                <input
                  type="password"
                  id="signup-password"
                  value={signupPassword}
                  onChange={(e) => setSignupPassword(e.target.value)}
                  placeholder="Create a password"
                />
              </div>
            </div>
            <div className="modal-footer">
              <button 
                className="modal-button secondary"
                onClick={() => setShowSignupModal(false)}
              >
                Cancel
              </button>
              <button 
                className="modal-button primary"
                onClick={() => handleRegister(signupEmail, signupUsername, signupPassword)}
              >
                Sign Up
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Email Verification Modal */}
      {showVerifyModal && (
        <div className="modal-overlay" onClick={() => setShowVerifyModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Verify Email</h2>
              <button className="modal-close" onClick={() => setShowVerifyModal(false)}>×</button>
            </div>
            <div className="modal-body">
              <p>A verification code has been sent to {verifyEmail}</p>
              <div className="form-group">
                <label htmlFor="verify-code">Verification Code</label>
                <input
                  type="text"
                  id="verify-code"
                  value={verifyCode}
                  onChange={(e) => setVerifyCode(e.target.value)}
                  placeholder="Enter verification code"
                />
              </div>
            </div>
            <div className="modal-footer">
              <button 
                className="modal-button secondary"
                onClick={() => setShowVerifyModal(false)}
              >
                Cancel
              </button>
              <button 
                className="modal-button primary"
                onClick={() => handleVerifyEmail(verifyEmail, verifyCode)}
              >
                Verify
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default App
