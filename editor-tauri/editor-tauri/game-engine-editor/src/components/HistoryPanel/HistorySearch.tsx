// History Search Component

import React, { useState } from 'react';
import { HistoryManager } from '../../utils/HistoryManager';
import { HistorySearchResult } from '../../types/history';
import './HistorySearch.css';

interface HistorySearchProps {
  historyManager: HistoryManager;
  onJumpTo: (stateId: string) => void;
}

export function HistorySearch({ historyManager, onJumpTo }: HistorySearchProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<HistorySearchResult | null>(null);
  const [searching, setSearching] = useState(false);

  const handleSearch = () => {
    if (!query.trim()) {
      setResults(null);
      return;
    }

    setSearching(true);
    const searchResults = historyManager.search(query);
    setResults(searchResults);
    setSearching(false);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  return (
    <div className="history-search">
      <div className="search-input-container">
        <input
          type="text"
          className="search-input"
          placeholder="Search commands and bookmarks..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyPress={handleKeyPress}
        />
        <button
          className="search-btn"
          onClick={handleSearch}
          disabled={searching || !query.trim()}
        >
          {searching ? 'Searching...' : 'Search'}
        </button>
      </div>

      {results && (
        <div className="search-results">
          <div className="search-results-header">
            Found {results.totalResults} results in {results.searchTime}ms
          </div>

          {results.commands.length > 0 && (
            <div className="search-results-section">
              <h4>Commands ({results.commands.length})</h4>
              <div className="search-results-list">
                {results.commands.map((cmd) => (
                  <SearchResultItem
                    key={cmd.id}
                    command={cmd}
                    onJumpTo={() => cmd.id && onJumpTo(cmd.id)}
                  />
                ))}
              </div>
            </div>
          )}

          {results.bookmarks.length > 0 && (
            <div className="search-results-section">
              <h4>Bookmarks ({results.bookmarks.length})</h4>
              <div className="search-results-list">
                {results.bookmarks.map((bm) => (
                  <BookmarkResultItem
                    key={bm.id}
                    bookmark={bm}
                    onJumpTo={() => onJumpTo(bm.stateId)}
                  />
                ))}
              </div>
            </div>
          )}

          {results.totalResults === 0 && (
            <div className="search-results-empty">
              <p>No results found for "{query}"</p>
              <p className="hint">Try different keywords or check your spelling</p>
            </div>
          )}
        </div>
      )}

      {!results && query && (
        <div className="search-hint">
          <p>Enter a search query to find commands and bookmarks</p>
          <p className="hint">You can search by description, ID, or tags</p>
        </div>
      )}
    </div>
  );
}

interface SearchResultItemProps {
  command: any;
  onJumpTo: () => void;
}

function SearchResultItem({ command, onJumpTo }: SearchResultItemProps) {
  return (
    <div className="search-result-item">
      <div className="result-icon">⚙️</div>
      <div className="result-content">
        <div className="result-description">{command.description}</div>
        <div className="result-meta">
          <span className="result-id">{command.id}</span>
          {command.timestamp && (
            <span className="result-time">
              {new Date(command.timestamp).toLocaleString()}
            </span>
          )}
        </div>
      </div>
      <button className="result-jump-btn" onClick={onJumpTo}>
        Go to
      </button>
    </div>
  );
}

interface BookmarkResultItemProps {
  bookmark: any;
  onJumpTo: () => void;
}

function BookmarkResultItem({ bookmark, onJumpTo }: BookmarkResultItemProps) {
  return (
    <div className="search-result-item bookmark">
      <div className="result-icon">🔖</div>
      <div className="result-content">
        <div className="result-name">{bookmark.name}</div>
        {bookmark.description && (
          <div className="result-description">{bookmark.description}</div>
        )}
        <div className="result-meta">
          <span className="result-time">
            {new Date(bookmark.timestamp).toLocaleString()}
          </span>
          {bookmark.tags.length > 0 && (
            <div className="result-tags">
              {bookmark.tags.map((tag: string) => (
                <span key={tag} className="tag">
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>
      <button className="result-jump-btn" onClick={onJumpTo}>
        Go to
      </button>
    </div>
  );
}
