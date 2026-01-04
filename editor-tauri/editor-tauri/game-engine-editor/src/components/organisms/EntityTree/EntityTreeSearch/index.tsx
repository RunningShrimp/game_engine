import React from 'react';

export interface EntityTreeSearchProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

/**
 * EntityTreeSearch - Search/filter input component
 * Provides search functionality for entity tree
 */
export const EntityTreeSearch: React.FC<EntityTreeSearchProps> = ({
  value,
  onChange,
  placeholder = 'Search entities...',
  className = '',
}) => {
  return (
    <div className={`relative ${className}`}>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full px-2 py-1 text-sm bg-slate-800 border border-slate-700 rounded text-slate-200 placeholder-slate-500 focus:border-blue-500 focus:outline-none"
        aria-label="Search entities"
      />
      {value && (
        <button
          className="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200"
          onClick={() => onChange('')}
          aria-label="Clear search"
        >
          <svg
            className="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      )}
    </div>
  );
};

export default EntityTreeSearch;
