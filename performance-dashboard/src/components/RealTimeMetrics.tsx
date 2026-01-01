import { MetricCard } from '../types'

interface Props {
  metrics: any
}

export default function RealTimeMetrics({ metrics }: Props) {
  if (!metrics) {
    return (
      <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h2 className="text-xl font-bold mb-4">Real-Time Metrics</h2>
        <div className="text-gray-400">Waiting for data...</div>
      </div>
    )
  }

  const cards: MetricCard[] = [
    { title: 'FPS', value: metrics.fps?.toFixed(1) || '0', unit: '', status: metrics.fps > 55 ? 'good' : metrics.fps > 30 ? 'warning' : 'critical' },
    { title: 'Frame Time', value: metrics.frameTime?.toFixed(2) || '0', unit: 'ms', status: metrics.frameTime < 16.67 ? 'good' : metrics.frameTime < 33.33 ? 'warning' : 'critical' },
    { title: 'CPU', value: metrics.cpu?.toFixed(1) || '0', unit: '%', status: metrics.cpu < 60 ? 'good' : metrics.cpu < 80 ? 'warning' : 'critical' },
    { title: 'Memory', value: (metrics.memory / 1024 / 1024).toFixed(1) || '0', unit: 'MB', status: metrics.memory < 512_000_000 ? 'good' : metrics.memory < 1024_000_000 ? 'warning' : 'critical' },
  ]

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
      <h2 className="text-xl font-bold mb-4">Real-Time Metrics</h2>
      <div className="grid grid-cols-2 gap-4">
        {cards.map((card, index) => (
          <div
            key={index}
            className={'p-4 rounded-lg border ' +
              (card.status === 'good' ? 'bg-green-900 border-green-700' :
               card.status === 'warning' ? 'bg-yellow-900 border-yellow-700' :
               'bg-red-900 border-red-700')
            }
          >
            <div className="text-sm text-gray-400">{card.title}</div>
            <div className="text-2xl font-bold">
              {card.value}<span className="text-sm font-normal text-gray-400"> {card.unit}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
