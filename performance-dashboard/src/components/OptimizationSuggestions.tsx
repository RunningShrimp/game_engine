interface Props {
  metrics: any
}

export default function OptimizationSuggestions({ metrics }: Props) {
  const suggestions = [
    { priority: 'high', title: 'Reduce draw calls', description: 'Batch similar objects to reduce draw calls from 150 to <50', impact: '+15% FPS' },
    { priority: 'medium', title: 'Optimize shadows', description: 'Use cascade shadow maps instead of PCSS for distant objects', impact: '+8% FPS' },
    { priority: 'low', title: 'Pool entities', description: 'Use object pooling for frequently spawned entities', impact: '+5% FPS' },
  ]

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'bg-red-900 border-red-700'
      case 'medium': return 'bg-yellow-900 border-yellow-700'
      case 'low': return 'bg-blue-900 border-blue-700'
      default: return 'bg-gray-800 border-gray-700'
    }
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
      <h2 className="text-xl font-bold mb-4">Optimization Suggestions</h2>
      <div className="space-y-3">
        {suggestions.map((suggestion, index) => (
          <div key={index} className={'p-4 rounded border ' + getPriorityColor(suggestion.priority)}>
            <div className="flex justify-between items-start mb-2">
              <h3 className="font-semibold">{suggestion.title}</h3>
              <span className="text-sm bg-green-900 px-2 py-1 rounded">{suggestion.impact}</span>
            </div>
            <p className="text-sm text-gray-300">{suggestion.description}</p>
          </div>
        ))}
      </div>
    </div>
  )
}
