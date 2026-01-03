import React from 'react';

interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
  language?: string;
  height?: string;
  readOnly?: boolean;
}

const CodeEditor: React.FC<CodeEditorProps> = ({
  value,
  onChange,
  language = 'rust',
  height = '300px',
  readOnly = false
}) => {
  return (
    <div className="relative w-full h-full bg-gray-900 dark:bg-gray-950">
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        readOnly={readOnly}
        className="w-full h-full px-4 py-3 bg-transparent text-gray-100 font-mono text-sm resize-none focus:outline-none"
        style={{ height }}
        spellCheck={false}
        placeholder="// 在这里输入代码..."
      />

      {/* 语言标签 */}
      <div className="absolute top-2 right-2 px-2 py-1 bg-gray-800 text-gray-400 text-xs rounded">
        {language}
      </div>
    </div>
  );
};

export default CodeEditor;
