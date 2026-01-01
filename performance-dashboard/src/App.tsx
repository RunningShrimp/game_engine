import { useState, useEffect } from 'react'
import RealTimeMetrics from './components/RealTimeMetrics'
import PerformanceCharts from './components/PerformanceCharts'
import AlertPanel from './components/AlertPanel'
import OptimizationSuggestions from './components/OptimizationSuggestions'

function App() {
  const [isConnected, setIsConnected] = useState(false)
  const [metrics, setMetrics] = useState<any>(null)

  useEffect(() => {
    const ws = new WebSocket('ws://localhost:8080/api/performance/stream')

    ws.onopen = () => {
      console.log('Connected to performance server')
      setIsConnected(true)
    }

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      setMetrics(data)
    }

    ws.onerror = (error) => {
      console.error('WebSocket error:', error)
    }

    return () => {
      ws.close()
    }
  }, [])

  const statusText = isConnected ? 'Connected' : 'Disconnected'
  const statusClass = isConnected ? 'bg-green-500' : 'bg-red-500'

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold">Game Engine Performance Dashboard</h1>
          <div className="flex items-center gap-4">
            <span className={'px-3 py-1 rounded-full text-sm ' + statusClass}>
              {statusText}
            </span>
          </div>
        </div>
      </header>

      <main className="p-6">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <RealTimeMetrics metrics={metrics} />
          <PerformanceCharts metrics={metrics} />
        </div>

        <div className="mt-6 grid grid-cols-1 lg:grid-cols-2 gap-6">
          <AlertPanel metrics={metrics} />
          <OptimizationSuggestions metrics={metrics} />
        </div>
      </main>
    </div>
  )
}

export default App
