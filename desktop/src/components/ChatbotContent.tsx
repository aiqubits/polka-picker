import { useState, useEffect, useRef } from 'react'
import type { ChatMessage, ChatbotSession } from '../types'
import './ChatbotContent.css'

interface ChatbotContentProps {
  // 可以添加从父组件传入的属性
}

const ChatbotContent = ({}: ChatbotContentProps) => {
  const [activeSession, setActiveSession] = useState<string>('1')
  const [sessions, setSessions] = useState<ChatbotSession[]>([
    {
      id: '1',
      title: 'Current Conversation',
      createdAt: new Date().toISOString(),
      lastMessage: 'Hello! How can I help you today?'
    }
  ])
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      id: '1',
      content: 'Hello! I\'m your AI assistant. How can I help you today?',
      sender: 'bot',
      timestamp: new Date().toISOString(),
      type: 'text',
      buttons: [
        { id: 'btn1', text: 'Show me available tools', action: 'show_tools' },
        { id: 'btn2', text: 'Help with a task', action: 'help_task' },
        { id: 'btn3', text: 'Explain features', action: 'explain_features' }
      ]
    }
  ])
  const [inputMessage, setInputMessage] = useState('')
  const [isTyping, setIsTyping] = useState(false)
  const [showSessions] = useState(true)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  // 自动滚动到最新消息
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  // 模拟LLM API响应
  const generateBotResponse = (userMessage: string) => {
    setIsTyping(true)
    
    // 简单的响应逻辑，根据用户输入生成不同的回复
    let botResponse = ''
    let responseButtons: { id: string; text: string; action: string }[] = []
    
    if (userMessage.toLowerCase().includes('help')) {
      botResponse = 'I can help you with various tasks. What specific issue are you facing?'
      responseButtons = [
        { id: 'task_help', text: 'Task troubleshooting', action: 'task_troubleshoot' },
        { id: 'tool_help', text: 'Tool installation', action: 'tool_install' },
        { id: 'account_help', text: 'Account issues', action: 'account_issues' }
      ]
    } else if (userMessage.toLowerCase().includes('tools')) {
      botResponse = 'We have various tools available in the marketplace. Here are some popular ones:'
      responseButtons = [
        { id: 'data_tool', text: 'Data Processing Tool', action: 'tool_data' },
        { id: 'monitor_tool', text: 'Server Monitoring', action: 'tool_monitor' },
        { id: 'backup_tool', text: 'Backup System Plugin', action: 'tool_backup' }
      ]
    } else if (userMessage.toLowerCase().includes('hello') || userMessage.toLowerCase().includes('hi')) {
      botResponse = 'Hello! How can I assist you today?'
      responseButtons = [
        { id: 'feature_req', text: 'Request a feature', action: 'feature_req' },
        { id: 'feedback', text: 'Provide feedback', action: 'feedback' }
      ]
    } else if (userMessage.toLowerCase().includes('task') || userMessage.toLowerCase().includes('pipeline')) {
      botResponse = 'I can help you create, manage, or troubleshoot tasks. What do you need help with specifically?'
      responseButtons = [
        { id: 'create_task', text: 'Create a new task', action: 'create_task' },
        { id: 'task_error', text: 'Fix task error', action: 'task_error' },
        { id: 'task_optimize', text: 'Optimize task', action: 'task_optimize' }
      ]
    } else {
      botResponse = `Thank you for your message: "${userMessage}". I'm here to help you with any questions or issues you might have.`
      responseButtons = [
        { id: 'more_help', text: 'More assistance', action: 'more_help' },
        { id: 'contact_support', text: 'Contact support', action: 'contact_support' }
      ]
    }

    // 模拟网络延迟
    setTimeout(() => {
      const newMessage: ChatMessage = {
        id: Date.now().toString(),
        content: botResponse,
        sender: 'bot',
        timestamp: new Date().toISOString(),
        type: 'text',
        buttons: responseButtons
      }
      
      setMessages(prevMessages => [...prevMessages, newMessage])
      setIsTyping(false)
      
      // 更新会话的最后消息
      setSessions(prevSessions => 
        prevSessions.map(session => 
          session.id === activeSession 
            ? { ...session, lastMessage: botResponse.substring(0, 50) + (botResponse.length > 50 ? '...' : '') }
            : session
        )
      )
    }, 1500)
  }

  // 处理用户发送消息
  const handleSendMessage = () => {
    if (!inputMessage.trim()) return

    const newUserMessage: ChatMessage = {
      id: Date.now().toString(),
      content: inputMessage.trim(),
      sender: 'user',
      timestamp: new Date().toISOString(),
      type: 'text'
    }

    setMessages(prevMessages => [...prevMessages, newUserMessage])
    setInputMessage('')

    // 更新会话的最后消息
    setSessions(prevSessions => 
      prevSessions.map(session => 
        session.id === activeSession 
          ? { ...session, lastMessage: inputMessage.substring(0, 50) + (inputMessage.length > 50 ? '...' : '') }
          : session
      )
    )

    // 生成机器人响应
    generateBotResponse(inputMessage.trim())
  }

  // 处理按钮点击
  const handleButtonClick = (action: string, text: string) => {
    // 创建一个模拟的用户消息
    const buttonMessage: ChatMessage = {
      id: Date.now().toString(),
      content: text,
      sender: 'user',
      timestamp: new Date().toISOString(),
      type: 'button'
    }

    setMessages(prevMessages => [...prevMessages, buttonMessage])

    // 根据按钮动作生成不同的响应
    let responseMessage = ''
    
    if (action === 'show_tools') {
      responseMessage = 'Here are some popular tools available in our marketplace. You can find more in the Marketplace tab.'
    } else if (action === 'help_task') {
      responseMessage = 'I can help you with task creation, management, and troubleshooting. What seems to be the issue?'
    } else if (action === 'explain_features') {
      responseMessage = 'Our platform offers task automation, tool integration, monitoring, and analytics. Let me know which feature you\'d like to learn more about.'
    } else {
      responseMessage = `You selected: "${text}". How can I assist you further with this?`
    }

    // 更新会话的最后消息
    setSessions(prevSessions => 
      prevSessions.map(session => 
        session.id === activeSession 
          ? { ...session, lastMessage: responseMessage.substring(0, 50) + (responseMessage.length > 50 ? '...' : '') }
          : session
      )
    )

    // 模拟机器人响应
    setIsTyping(true)
    setTimeout(() => {
      const botResponse: ChatMessage = {
        id: Date.now().toString(),
        content: responseMessage,
        sender: 'bot',
        timestamp: new Date().toISOString(),
        type: 'text'
      }
      
      setMessages(prevMessages => [...prevMessages, botResponse])
      setIsTyping(false)
    }, 1000)
  }

  // 创建新会话
  const handleNewSession = () => {
    const newSession: ChatbotSession = {
      id: Date.now().toString(),
      title: `New Conversation`,
      createdAt: new Date().toISOString()
    }

    setSessions(prevSessions => [...prevSessions, newSession])
    setActiveSession(newSession.id)
    setMessages([
      {
        id: Date.now().toString(),
        content: 'Hello! I\'m your AI assistant. How can I help you today?',
        sender: 'bot',
        timestamp: new Date().toISOString(),
        type: 'text'
      }
    ])
  }

  // 删除会话
  const handleDeleteSession = (sessionId: string, event: React.MouseEvent) => {
    event.stopPropagation(); // 阻止事件冒泡，避免触发会话切换
    
    // 如果删除的是当前活动会话，需要切换到另一个会话
    if (sessionId === activeSession) {
      const remainingSessions = sessions.filter(s => s.id !== sessionId);
      if (remainingSessions.length > 0) {
        setActiveSession(remainingSessions[0].id);
        // 实际应用中这里应该加载该会话的消息
      }
    }
    
    setSessions(prevSessions => prevSessions.filter(session => session.id !== sessionId));
  }

  // 格式化时间戳
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  return (
    <div className="chatbot-content">
      {/* Chat Container */}
      <div className="chat-container">
        {/* Sessions Sidebar */}
        {showSessions && (
          <div className="sessions-sidebar">
            <div className="session-list">
              {sessions.map(session => (
                <div
                  key={session.id}
                  className={`session-item ${activeSession === session.id ? 'active' : ''}`}
                  onClick={() => {
                    setActiveSession(session.id)
                    // 实际应用中这里应该加载该会话的消息
                  }}
                >
                  <div className="session-header">
                    <div className="session-title">{session.title}</div>
                    <button 
                      className="session-delete-btn"
                      onClick={(e) => handleDeleteSession(session.id, e)}
                      title="Delete session"
                    >
                      ×
                    </button>
                  </div>
                  {session.lastMessage && (
                    <div className="session-last-message">{session.lastMessage}</div>
                  )}
                </div>
              ))}
            </div>
            
            {/* New Session Button at the bottom - outside of scrollable list */}
            <div className="session-list-bottom">
              <button 
                className="new-session-btn-bottom"
                onClick={handleNewSession}
              >
                New Session
              </button>
            </div>
          </div>
        )}

        {/* Chat Messages */}
        <div className="chat-messages">
          {messages.map(message => (
            <div key={message.id} className={`chat-message ${message.sender}`}>
              {message.sender === 'bot' && (
                <div className="message-avatar">AI</div>
              )}
              <div className="message-bubble">
                {message.content}
                <div className="message-timestamp">{formatTimestamp(message.timestamp)}</div>
                
                {/* Display buttons if available */}
                {message.buttons && message.buttons.length > 0 && (
                  <div className="buttons-container">
                    {message.buttons.map(button => (
                      <button
                        key={button.id}
                        className="chat-button"
                        onClick={() => handleButtonClick(button.action, button.text)}
                      >
                        {button.text}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              {message.sender === 'user' && (
                <div className="message-avatar">U</div>
              )}
            </div>
          ))}
          
          {/* Typing Indicator */}
          {isTyping && (
            <div className="chat-message bot">
              <div className="message-avatar">AI</div>
              <div className="typing-indicator">
                <div className="typing-dot"></div>
                <div className="typing-dot"></div>
                <div className="typing-dot"></div>
              </div>
            </div>
          )}
          
          {/* Scroll reference */}
          <div ref={messagesEndRef} />
        </div>
      </div>

      {/* Input Area */}
      <div className="chat-input-area">
        <div className="chat-input-container">
          <input
            type="text"
            className="chat-input"
            placeholder="Type your message..."
            value={inputMessage}
            onChange={(e) => setInputMessage(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
            disabled={isTyping}
          />
          <button
            className="send-btn"
            onClick={handleSendMessage}
            disabled={!inputMessage.trim() || isTyping}
          >
            ➤
          </button>
        </div>
      </div>
    </div>
  )
}

export default ChatbotContent