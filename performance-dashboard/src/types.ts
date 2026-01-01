export interface MetricCard {
  title: string
  value: string
  unit: string
  status: 'good' | 'warning' | 'critical'
}

export interface PerformanceData {
  fps: number
  frameTime: number
  cpu: number
  memory: number
  gpu: number
  drawCalls: number
  triangleCount: number
}

export interface Alert {
  level: 'critical' | 'warning' | 'info'
  message: string
  time: string
}

export interface Suggestion {
  priority: 'high' | 'medium' | 'low'
  title: string
  description: string
  impact: string
}
