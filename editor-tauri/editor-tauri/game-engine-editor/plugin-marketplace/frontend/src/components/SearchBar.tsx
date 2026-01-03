/**
 * SearchBar Component
 *
 * Search input with filters
 */

import React, { useState } from 'react';
import { Search, SlidersHorizontal, X } from 'lucide-react';
import { SearchFilters as Filters, PricingType } from '../types';

interface SearchBarProps {
  onSearch: (filters: Filters) => void;
  loading?: boolean;
}

export const SearchBar: React.FC<SearchBarProps> = ({ onSearch, loading }) => {
  const [query, setQuery] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<Filters>({});

  const handleSearch = () => {
    onSearch({
      ...filters,
      query: query || undefined,
    });
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const clearFilters = () => {
    setFilters({});
    setQuery('');
    onSearch({});
  };

  const hasActiveFilters = () => {
    return (
      query ||
      filters.categories?.length ||
      filters.tags?.length ||
      filters.pricingType ||
      filters.minRating
    );
  };

  return (
    <div className="search-bar">
      <div className="search-bar__main">
        <div className="search-bar__input-wrapper">
          <Search size={20} className="search-bar__icon" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Search plugins..."
            className="search-bar__input"
            disabled={loading}
          />

          {hasActiveFilters() && (
            <button
              onClick={clearFilters}
              className="search-bar__clear"
              title="Clear filters"
            >
              <X size={20} />
            </button>
          )}
        </div>

        <button
          onClick={handleSearch}
          disabled={loading}
          className="search-bar__submit"
        >
          {loading ? 'Searching...' : 'Search'}
        </button>

        <button
          onClick={() => setShowFilters(!showFilters)}
          className="search-bar__filter-toggle"
          title="Toggle filters"
        >
          <SlidersHorizontal size={20} />
        </button>
      </div>

      {showFilters && (
        <div className="search-bar__filters">
          {/* Pricing Type */}
          <div className="filter-group">
            <label className="filter-label">Pricing</label>
            <select
              value={filters.pricingType || ''}
              onChange={(e) =>
                setFilters({
                  ...filters,
                  pricingType: e.target.value as PricingType || undefined,
                })
              }
              className="filter-select"
            >
              <option value="">All</option>
              <option value="free">Free</option>
              <option value="paid">Paid</option>
              <option value="freemium">Freemium</option>
              <option value="subscription">Subscription</option>
            </select>
          </div>

          {/* Minimum Rating */}
          <div className="filter-group">
            <label className="filter-label">Min Rating</label>
            <select
              value={filters.minRating?.toString() || ''}
              onChange={(e) =>
                setFilters({
                  ...filters,
                  minRating: e.target.value ? parseFloat(e.target.value) : undefined,
                })
              }
              className="filter-select"
            >
              <option value="">Any</option>
              <option value="4">4+ Stars</option>
              <option value="3">3+ Stars</option>
              <option value="2">2+ Stars</option>
              <option value="1">1+ Stars</option>
            </select>
          </div>

          {/* Sort By */}
          <div className="filter-group">
            <label className="filter-label">Sort By</label>
            <select
              value={filters.sortBy || 'relevance'}
              onChange={(e) =>
                setFilters({
                  ...filters,
                  sortBy: e.target.value as any,
                })
              }
              className="filter-select"
            >
              <option value="relevance">Relevance</option>
              <option value="downloads">Downloads</option>
              <option value="rating">Rating</option>
              <option value="updated">Recently Updated</option>
              <option value="name">Name</option>
            </select>
          </div>

          {/* Categories would be loaded dynamically */}
          <div className="filter-group">
            <label className="filter-label">Category</label>
            <select
              value={filters.categories?.[0] || ''}
              onChange={(e) =>
                setFilters({
                  ...filters,
                  categories: e.target.value ? [e.target.value] : [],
                })
              }
              className="filter-select"
            >
              <option value="">All Categories</option>
              <option value="rendering">Rendering</option>
              <option value="physics">Physics</option>
              <option value="ai">AI</option>
              <option value="audio">Audio</option>
              <option value="tools">Tools</option>
              <option value="ui">UI</option>
            </select>
          </div>
        </div>
      )}
    </div>
  );
};
