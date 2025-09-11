import './ProfileContent.css'
import type { Activity, InstalledTool, ProfileStats } from '../types'
import './ProfileContent.css'

interface ProfileContentProps {
  // 可以添加从父组件传入的属性
}

const ProfileContent = ({}: ProfileContentProps) => {
  // 模拟用户数据
  const userData = {
    name: 'Deporter',
    role: 'DEV',
    bio: 'Full-stack developer with expertise in cloud architecture and DevOps',
    location: 'San Francisco, CA',
    email: 'johndoe@example.com',
    memberSince: 'Jan 2022',
    position: 'Senior Developer',
    skills: ['React', 'Node.js', 'Python', 'AWS', 'Docker', 'CI/CD']
  }

  // 模拟最近活动数据
  const recentActivities: Activity[] = [
    {
      id: '1',
      type: 'installation',
      title: 'Installed Server Monitoring',
      description: 'Tool Server Monitoring',
      timestamp: '2024-05-01 01:32 AM'
    },
    {
      id: '2',
      type: 'purchase',
      title: 'Purchased Premium Plan',
      description: 'Subscription',
      timestamp: '2024-04-28 2:45 PM'
    },
    {
      id: '3',
      type: 'usage',
      title: 'Used API Integration Helper',
      description: 'Tool',
      timestamp: '2024-04-27 11:20 AM'
    },
    {
      id: '4',
      type: 'contribution',
      title: 'Submitted feedback for Data Processing Tool',
      description: 'Community',
      timestamp: '2024-04-25 4:12 PM'
    }
  ]

  // 模拟已安装工具数据
  const installedTools: InstalledTool[] = [
    {
      id: '1',
      name: 'Server Monitoring',
      type: 'performance',
      installedDate: '2024-04-01'
    },
    {
      id: '2',
      name: 'Backup System Plugin',
      type: 'security',
      installedDate: '2024-04-22'
    }
  ]

  // 模拟账户统计数据
  const profileStats: ProfileStats = {
    toolsUsed: 12,
    contributions: 5,
    tasksCompleted: 87,
    monthsActive: 8,
    storageUsed: 1.2,
    storageTotal: 5,
    walletBalance: 10,
    premiumCredits: 50
  }

  // 获取活动类型对应的图标
  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'installation': return '🖥️'
      case 'purchase': return '💰'
      case 'usage': return '🔧'
      case 'contribution': return '📝'
      default: return '•'
    }
  }

  // 获取工具类型对应的图标
  const getToolTypeIcon = (type: string) => {
    switch (type) {
      case 'performance': return '⚡'
      case 'security': return '🛡️'
      default: return '🔧'
    }
  }

  return (
    <div className="profile-content">
      {/* 个人信息卡片 */}
      <div className="profile-card">
        <div className="profile-avatar">De</div>
        <h2 className="profile-name">{userData.name}</h2>
        <span className="profile-role">{userData.role}</span>
        <p className="profile-bio">{userData.bio}</p>
        
        <div className="personal-info">
          <div className="info-item">
            <span className="info-icon">📍</span>
            <span className="info-text">{userData.location}</span>
          </div>
          <div className="info-item">
            <span className="info-icon">✉️</span>
            <span className="info-text">{userData.email}</span>
          </div>
          <div className="info-item">
            <span className="info-icon">📅</span>
            <span className="info-text">Member since {userData.memberSince}</span>
          </div>
          <div className="info-item">
            <span className="info-icon">💼</span>
            <span className="info-text">{userData.position}</span>
          </div>
        </div>

        <div className="skills-section">
          <h3 className="section-subtitle">Skills & Expertise</h3>
          <div className="skills-tags">
            {userData.skills.map((skill, index) => (
              <span key={index} className="skill-tag">{skill}</span>
            ))}
          </div>
        </div>

        <div className="profile-actions">
          <button className="edit-profile-btn">
            <span className="btn-icon">✏️</span>
            Edit Profile
          </button>
          <button className="settings-btn">⚙️</button>
        </div>
      </div>

      {/* 右侧内容区域 */}
      <div className="profile-right">
        {/* 最近活动 */}
        <div className="recent-activity">
          <div className="section-header">
            <h3 className="section-title">Recent Activity</h3>
            <button className="section-menu">⋮</button>
          </div>
          <div className="activity-list">
            {recentActivities.map(activity => (
              <div key={activity.id} className="activity-item">
                <div className="activity-icon">
                  {getActivityIcon(activity.type)}
                </div>
                <div className="activity-content">
                  <div className="activity-title">{activity.title}</div>
                  <div className="activity-description">{activity.description}</div>
                </div>
                <div className="activity-time">{activity.timestamp}</div>
              </div>
            ))}
          </div>
        </div>

        {/* 已安装工具 */}
        <div className="installed-tools">
          <div className="section-header">
            <h3 className="section-title">Installed Tools</h3>
            <button className="section-menu">⋮</button>
          </div>
          <div className="tools-list">
            {installedTools.map(tool => (
              <div key={tool.id} className="tool-item">
                <div className="tool-icon">{getToolTypeIcon(tool.type)}</div>
                <div className="tool-info">
                  <div className="tool-name">{tool.name}</div>
                  <div className="tool-date">Installed on {tool.installedDate}</div>
                </div>
                <button className="tool-remove">🗑️</button>
              </div>
            ))}
          </div>
        </div>

        {/* 账户统计 */}
        <div className="account-stats">
          <div className="section-header">
            <h3 className="section-title">Account Statistics</h3>
            <button className="section-menu">⋮</button>
          </div>
          
          <div className="stats-wallet">
            <div className="wallet-item">
              <span className="wallet-label">Wallet Balance</span>
              <span className="wallet-value">{profileStats.walletBalance} CFX</span>
            </div>
            <div className="wallet-item">
              <span className="wallet-label">Premium Credits</span>
              <span className="wallet-value">{profileStats.premiumCredits}</span>
            </div>
            <div className="storage-item">
              <span className="storage-label">Storage Used</span>
              <div className="storage-bar">
                <div 
                  className="storage-fill" 
                  style={{ width: `${(profileStats.storageUsed / profileStats.storageTotal) * 100}%` }}
                ></div>
              </div>
              <span className="storage-value">{profileStats.storageUsed}/{profileStats.storageTotal} GB</span>
            </div>
          </div>

          <div className="stats-grid">
            <div className="stat-card">
              <span className="stat-value">{profileStats.toolsUsed}</span>
              <span className="stat-label">Tools Used</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{profileStats.contributions}</span>
              <span className="stat-label">Contributions</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{profileStats.tasksCompleted}</span>
              <span className="stat-label">Tasks Completed</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{profileStats.monthsActive}</span>
              <span className="stat-label">Months Active</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ProfileContent