interface Props {
  metrics: any
}

export default function AlertPanel({ metrics }: Props) {
  const alerts = [
    { level: 'critical', message: 'FPS dropped below 30', time: '2s ago' },
    { level: 'warning', message: 'Memory usage high (85%)', time: '5s ago' },
    { level: 'info', message: 'New benchmark completed', time: '10s ago' },
  ]

  const getAlertColor = (level: string) => {
    switch (level) {
      case 'critical': return 'bg-red-900 border-red-700 text-red-300'
      case 'warning': return 'bg-yellow-900 border-yellow-700 text-yellow-300'
      case 'info': return 'bg-blue-900 border-blue-700 text-blue-300'
      default: return 'bg-gray-800 border-gray-700'
    }
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
      <h2 className="text-xl font-bold mb-4">Performance Alerts</h2>
      <div className="space-y-2">
        {alerts.map((alert, index) => (
          <div key={index} className={'p-3 rounded border ' + getAlertColor(alert.level)}>
            <div className="flex justify-between items-start">
              <div>
                <div className="font-semibold">{alert.message}</div>
                <div className="text-sm opacity-75">{alert.time}</div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
