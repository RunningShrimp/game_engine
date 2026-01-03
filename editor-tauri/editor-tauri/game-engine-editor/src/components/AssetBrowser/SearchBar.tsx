import React, { useState } from 'react';
import { Search, X } from 'lucide-react';
import { debounce } from './utils';

interface SearchBarProps {
  onSearchChange: (search: string) => void;
  value: string;
}

export function SearchBar({ onSearchChange, value }: SearchBarProps) {
  const [localValue, setLocalValue] = useState(value);

  const debouncedSearch = debounce((search: string) => {
    onSearchChange(search);
  }, 300);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setLocalValue(newValue);
    debouncedSearch(newValue);
  };

  const handleClear = () => {
    setLocalValue('');
    onSearchChange('');
  };

  return (
    <div className="relative">
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
      <input
        type="text"
        value={localValue}
        onChange={handleChange}
        placeholder="Search assets..."
        className="w-full pl-10 pr-10 py-2 bg-slate-800 border border-slate-700 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      {localValue && (
        <button
          onClick={handleClear}
          className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200"
        >
          <X className="w-4 h-4" />
        </button>
      )}
    </div>
  );
}
