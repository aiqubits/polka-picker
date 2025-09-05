import type { Task, TaskStatus } from '../types'
import TaskCard from './TaskCard'
import './MainContent.css'

interface MainContentProps {
  tasks: Task[]
  activeFilter: TaskStatus
  onFilterChange: (filter: TaskStatus) => void
  searchQuery: string
  onSearchChange: (query: string) => void
}

const MainContent = ({ 
  tasks, 
  activeFilter, 
  onFilterChange, 
  searchQuery, 
  onSearchChange 
}: MainContentProps) => {
  const filterOptions: { key: TaskStatus; label: string; count?: number }[] = [
    { key: 'all', label: 'All' },
    { key: 'running', label: 'Running' },
    { key: 'idle', label: 'Idle' },
    { key: 'error', label: 'Error' }
  ]

  const getTaskCount = (status: TaskStatus) => {
    if (status === 'all') return tasks.length
    return tasks.filter(task => task.status === status).length
  }

  return (
    <div className="main-content">
      {/* Header */}
      <div className="content-header">
        <h1 className="page-title">My Tasks</h1>
        <div className="header-controls">
          <div className="filter-tabs">
            {filterOptions.map(option => (
              <button
                key={option.key}
                className={`filter-tab ${activeFilter === option.key ? 'active' : ''}`}
                onClick={() => onFilterChange(option.key)}
              >
                {option.label}
                {option.key !== 'all' && (
                  <span className="filter-count">{getTaskCount(option.key)}</span>
                )}
              </button>
            ))}
          </div>
          <div className="search-container">
            <input
              type="text"
              placeholder="Search tasks..."
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              className="search-input"
            />
            <span className="search-icon">🔍</span>
          </div>
        </div>
      </div>

      {/* Task Grid */}
      <div className="task-grid">
        {tasks.map(task => (
          <TaskCard key={task.id} task={task} />
        ))}
      </div>

      {/* Add Button */}
      <button className="add-button" title="Add new task">
        <span className="add-icon">+</span>
      </button>
    </div>
  )
}

export default MainContent