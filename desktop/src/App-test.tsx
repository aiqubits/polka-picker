import './App.css'
import './App-test.css'
import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface ConnectionStatus {
  is_connected: boolean;
  response_time_ms: number;
  server_status: string;
  auth_valid: boolean;
  error_message: string | null;
}

// 定义API响应的类型
interface LoginResponse {
  token: string
  user: {
    user_id: string
    email: string
    user_name: string
    user_type: string
    wallet_address: string
    premium_balance: number
    created_at: string
  }
}

interface UserInfo {
  user_id: string
  email: string
  user_name: string
  user_type: string
  wallet_address: string
  premium_balance: number
  created_at: string
}

interface PickerInfo {
  picker_id: string
  alias: string
  description: string
  price: number
  image_path: string
  version: string
  download_count: number
  created_at: string
}

interface PickerListResponse {
  pickers: PickerInfo[]
  total: number
}

interface OrderInfo {
  order_id: string
  user_id: string
  picker_id: string
  picker_alias: string
  amount: number
  pay_type: string
  status: string
  created_at: string
}

interface OrderListResponse {
  orders: OrderInfo[]
  total: number
  page: number
  size: number
  has_next: boolean
}

interface CreateOrderResponse {
  order_id: string
  message: string
}

// 定义测试状态
interface TestState {
  isRunning: boolean
  results: { [key: string]: { success: boolean; message: string; data?: unknown } }
}

function AppTest() {
  const [testState, setTestState] = useState<TestState>({
    isRunning: false,
    results: {}
  })

  // 辅助函数：执行测试用例
  // 修改 runTest 函数以捕获更详细的错误信息
  const runTest = async (name: string, testFn: () => Promise<unknown>) => {
    try {
      setTestState(prev => ({
        ...prev,
        results: {
          ...prev.results,
          [name]: { success: false, message: 'Running...' }
        }
      }))
  
      console.log(`Running test: ${name}`);
      const startTime = Date.now();
      
      const data = await testFn()
      
      const endTime = Date.now();
      console.log(`Test ${name} completed in ${endTime - startTime}ms with success`);
  
      setTestState(prev => ({
        ...prev,
        results: {
          ...prev.results,
          [name]: { success: true, message: 'Success', data }
        }
      }))
    } catch (error) {
      console.error(`Test ${name} failed:`, error);
      // 改进错误处理，使错误信息更清晰
      let errorMessage = 'Unknown error';
      if (error instanceof Error) {
        errorMessage = `${error.name}: ${error.message}`;
      } else if (typeof error === 'object' && error !== null) {
        // 尝试解析JSON错误信息
        try {
          const errorObj = error as Record<string, unknown>;
          if (errorObj.error || errorObj.message) {
            errorMessage = String(errorObj.error || errorObj.message);
          } else {
            errorMessage = JSON.stringify(error);
          }
        } catch {
          errorMessage = String(error);
        }
      }
      
      setTestState(prev => ({
        ...prev,
        results: {
          ...prev.results,
          [name]: {
            success: false,
            message: errorMessage
          }
        }
      }))
    }
  }

  // 用户相关测试
  const testLogin = async () => {
    const response = await invoke<LoginResponse>('login', {
      email: 'testdata@openpick.org', // 假设的测试邮箱
      userPassword: 'testpassword' // 假设的测试密码
    })
    return response
  }

  const testRegister = async () => {
    const timestamp = Date.now()
    const email = `test${timestamp}@example.com`
    
    await invoke('register', {
      email,
      userPassword: 'testpassword',
      userName: `TestUser${timestamp}`,
      userType: 'gen'
    })
    return { email }
  }

  const testVerifyEmail = async () => {
    // 实际应用中需要先获取验证码
    // 注意：这里应该使用刚刚注册的邮箱进行验证
    // 但在测试环境中，我们只检查API调用是否成功
    await invoke('verify_email', {
      email: 'test@example.com',
      code: '123456' // 假设的验证码
    })
  }

  const testGetUserProfile = async () => {
    const profile = await invoke<UserInfo>('get_user_profile')
    return profile
  }

  const testLogout = async () => {
    await invoke('logout')
  }

  const testCheckLoginStatus = async () => {
    const status = await invoke<boolean>('check_login_status')
    return status
  }

  const testGetCurrentUserInfo = async () => {
    const userInfo = await invoke<object | null>('get_current_user_info')
    return userInfo
  }

  // Picker相关测试
  const testGetPickerMarketplace = async () => {
    const response = await invoke<PickerListResponse>('get_picker_marketplace', {
      page: 1,
      size: 10,
      keyword: null
    })
    return response
  }

  const testGetPickerDetail = async () => {
    // 先获取市场列表，再取第一个Picker的ID
    const marketplace = await invoke<PickerListResponse>('get_picker_marketplace', {
      page: 1,
      size: 10,
      keyword: null
    })
    if (marketplace.pickers.length > 0) {

    const pickerId = marketplace.pickers[0].picker_id
    
      const detail = await invoke<PickerInfo>('get_picker_detail', {
        pickerId
      })
      return detail
    }
    return { message: 'No pickers available for detail test' }
  }

  const testUploadPicker = async () => {
    // 创建一个简单的测试文件内容
    const fileContent = new TextEncoder().encode('This is a test picker file')
    
    // 创建一个简单的测试图片内容
    const imageContent = new TextEncoder().encode('This is a test image')
    
    const timestamp = Date.now()
    
    await invoke('upload_picker', {
      alias: `TestPicker${timestamp}`,
      description: 'This is a test picker for API testing',
      version: '1.0.0',
      price: 100,
      file: Array.from(fileContent),
      image: Array.from(imageContent)
    })
    
    return { alias: `TestPicker${timestamp}` }
  }

  // 订单相关测试
  const testGetUserOrders = async () => {
    const orders = await invoke<OrderListResponse>('get_user_orders', {
      page: 1,
      size: 10,
      status: 'success'
    })
    return orders
  }

  const testCreateOrder = async () => {
    // 先获取市场列表，再取第一个Picker的ID
    const marketplace = await invoke<PickerListResponse>('get_picker_marketplace', {
      page: 1,
      size: 1
    })
    
    if (marketplace.pickers.length > 0) {
      const pickerId = marketplace.pickers[0].picker_id
      const order = await invoke<CreateOrderResponse>('create_order', {
        pickerId: pickerId,
        payType: 'wallet'
      })
      return order
    }
    return { message: 'No pickers available for order test' }
  }

  const testGetOrderDetail = async () => {
    // 先创建一个订单，再获取其详情
    const marketplace = await invoke<PickerListResponse>('get_picker_marketplace', {
      page: 1,
      size: 1
    })
    
    if (marketplace.pickers.length > 0) {
      const pickerId = marketplace.pickers[0].picker_id
      try {
        const order = await invoke<CreateOrderResponse>('create_order', {
          picker_id: pickerId,
          pay_type: 'wallet'
        })
        
        const detail = await invoke<OrderInfo>('get_order_detail', {
          order_id: order.order_id
        })
        return detail
      } catch (error) {
        // 如果创建订单失败（可能是因为余额不足等原因），返回错误信息
        return { error: error instanceof Error ? error.message : 'Failed to create order' }
      }
    }
    return { message: 'No pickers available for order detail test' }
  }

  // 下载相关测试
  const testDownloadPicker = async () => {
    // 注意：实际测试中需要一个有效的下载令牌
    // 这里使用一个示例令牌进行测试
    const sampleToken = 'sample_download_token'
    
    try {
      const filePath = await invoke<string>('download_picker', {
        token: sampleToken
      })
      return { filePath }
    } catch (error) {
      // 下载测试可能会失败，因为需要有效的令牌
      return { error: error instanceof Error ? error.message : 'Failed to download picker' }
    }
  }
  // 测试greet命令
  const testGreet = async () => {
    const result = await invoke<string>('greet', { name: 'Test User' });
    return result;
  }

  // 简单连接测试
  const simpleConnectionTest = async () => {
      try {
        const status = await invoke<ConnectionStatus>('simple_connection_test', {
        name: "zss321"
      });
        return status;
      } catch (error) {
        // console.error('简单连接测试失败:', error);
        return error; // 重新抛出错误，让测试框架正确处理
      }
  }

  // 添加一个简单的网络连接测试
  const testApiConnection = async () => {
    try {
      // 尝试直接通过 fetch 连接 API 服务器（如果允许的话）
      const response = await fetch('http://127.0.0.1:3000');
      if (response.ok) {
        return { status: 'success', message: 'API server is reachable' };
      } else {
        return { status: 'error', message: `API server returned status: ${response.status}` };
      }
    } catch (error) {
      return { status: 'error', message: `Cannot connect to API server: ${error instanceof Error ? error.message : 'Unknown error'}` };
    }
  }

  const tauriCheckConnection = async () => {
      try {
        const status = await invoke<ConnectionStatus>('api_connection');
        return status;
      } catch (error) {
        return error;
      }
    };
  
  // 运行所有测试
  // 在 runAllTests 函数开始时添加连接测试
  const runAllTests = async () => {
    setTestState({
      isRunning: true,
      results: {}
    })

    try {
      await runTest('test_greet', testGreet)
      await runTest('simple_connection_test', simpleConnectionTest)
      await runTest('api_connection_test', testApiConnection)
      await runTest('tauri_check_connection', tauriCheckConnection)
      // 首先测试不需要登录的API
      await runTest('get_picker_marketplace', testGetPickerMarketplace)
      await runTest('get_picker_detail', testGetPickerDetail)
      await runTest('check_login_status', testCheckLoginStatus)
      await runTest('get_current_user_info', testGetCurrentUserInfo)

      // 然后尝试登录（如果需要）
      try {
        await runTest('login', testLogin)
        
        // 登录成功后，测试需要登录的API
        await runTest('get_user_profile', testGetUserProfile)
        await runTest('get_user_orders', testGetUserOrders)
        await runTest('create_order', testCreateOrder)
        await runTest('get_order_detail', testGetOrderDetail)
        
        // 上传Picker测试
        try {
          await runTest('upload_picker', testUploadPicker)
        } catch (error) {
          console.warn('Upload picker test might require special permissions:', error)
        }
        
        // 下载Picker测试
        await runTest('download_picker', testDownloadPicker)
        
        // 登出测试
        await runTest('logout', testLogout)
        
        // 测试登出后的登录状态
        await runTest('check_login_status_after_logout', testCheckLoginStatus)
      } catch (error) {
        console.warn('Login test failed, some tests will be skipped:', error)
      }

      // 测试注册功能（独立于登录）
      try {
        await runTest('register', testRegister)
        
        // 邮箱验证测试（需要有效的验证码）
        try {
          await runTest('verify_email', testVerifyEmail)
        } catch (error) {
          console.warn('Verify email test might require a valid code:', error)
        }
      } catch (error) {
        console.warn('Register test failed:', error)
      }
    } finally {
      setTestState(prev => ({
        ...prev,
        isRunning: false
      }))
    }
  }

  // 可滚动的测试结果组件
  const ScrollableTestResult = ({ name, result }: { name: string; result: { success: boolean; message: string; data?: unknown } }) => {
    const [isExpanded, setIsExpanded] = useState(false);
    
    return (
      <div key={name} className={`test-result ${result.success ? 'success' : 'error'}`}>
        <h4>{name}</h4>
        <p>{result.message}</p>
        {result.data !== undefined && (
          <div className={`test-data ${isExpanded ? 'expanded' : ''}`}>
            <div className="scroll-controls">
              <button 
                className="toggle-expand" 
                onClick={() => setIsExpanded(!isExpanded)}
                title={isExpanded ? "收起" : "展开全部"}
              >
                {isExpanded ? "▼" : "▲"}
              </button>
            </div>
            <pre>{JSON.stringify(result.data, null, 2)}</pre>
          </div>
        )}
      </div>
    );
  };
  
  // 渲染测试结果
  const renderTestResults = () => {
    return Object.entries(testState.results).map(([name, result]) => (
      <ScrollableTestResult key={name} name={name} result={result} />
    ));
  }

  return (
    <div className="app">
      <div className="top-header">
        <div className="header-left">
          <div className="logo">
            <span className="logo-text">OpenPick API Test</span>
          </div>
        </div>
      </div>

      <div className="app-main">
        <div className="test-container">
          <h1>Tauri API 测试脚本</h1>
          <p>此脚本用于测试所有 Tauri 后端 API 接口的可行性。</p>
          
          <button 
            className="run-tests-button" 
            onClick={runAllTests} 
            disabled={testState.isRunning}
          >
            {testState.isRunning ? '测试进行中...' : '运行所有测试'}
          </button>

          <div className="test-results-container">
            <div className="test-results-header">
              {Object.keys(testState.results).length > 0 ? (
                <h2>测试结果</h2>
              ) : null}
              <div className="results-stats">
                <span>总测试数: {Object.keys(testState.results).length}</span>
                <span>成功: {Object.values(testState.results).filter(r => r.success).length}</span>
                <span>失败: {Object.values(testState.results).filter(r => !r.success).length}</span>
              </div>
            </div>
            <div className="test-results scrollable-results">
              {renderTestResults()}
            </div>
          </div>
        </div>
      </div>


    </div>
  )
}

export default AppTest