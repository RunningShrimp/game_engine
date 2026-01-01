import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts'

interface Props {
  metrics: any
}

export default function PerformanceCharts({ metrics }: Props) {
  const sampleData = [
    { time: '0s', fps: 60, cpu: 45, memory: 256 },
    { time: '1s', fps: 58, cpu: 52, memory: 268 },
    { time: '2s', fps: 55, cpu: 61, memory: 285 },
    { time: '3s', fps: 60, cpu: 48, memory: 272 },
    { time: '4s', fps: 62, cpu: 43, memory: 260 },
  ]

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
      <h2 className="text-xl font-bold mb-4">Performance Charts</h2>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={sampleData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
          <XAxis dataKey="time" stroke="#9CA3AF" />
          <YAxis stroke="#9CA3AF" />
          <Tooltip
            contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151' }}
          />
          <Legend />
          <Line type="monotone" dataKey="fps" stroke="#10B981" strokeWidth={2} name="FPS" />
          <Line type="monotone" dataKey="cpu" stroke="#F59E0B" strokeWidth={2} name="CPU %" />
          <Line type="monotone" dataKey="memory" stroke="#3B82F6" strokeWidth={2} name="Memory (MB)" />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
