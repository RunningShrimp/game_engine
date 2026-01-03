/**
 * PluginCard Component
 *
 * Displays a plugin card in the marketplace grid
 */

import React from 'react';
import { Link } from 'next/link';
import Image from 'next/image';
import { Plugin } from '../types';
import { Star, Download, Tag, Package } from 'lucide-react';

interface PluginCardProps {
  plugin: Plugin;
}

export const PluginCard: React.FC<PluginCardProps> = ({ plugin }) => {
  const formatDownloads = (num: number): string => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  const getPricingBadge = () => {
    switch (plugin.pricing.pricingType) {
      case 'free':
        return <span className="badge badge-success">Free</span>;
      case 'paid':
        return (
          <span className="badge badge-primary">
            {plugin.pricing.price && plugin.pricing.currency
              ? `${plugin.pricing.currency}${plugin.pricing.price}`
              : 'Paid'}
          </span>
        );
      case 'freemium':
        return <span className="badge badge-info">Freemium</span>;
      case 'subscription':
        return <span className="badge badge-warning">Subscription</span>;
      default:
        return null;
    }
  };

  return (
    <Link href={`/plugins/${plugin.slug}`} className="block">
      <div className="plugin-card">
        {/* Plugin Image/Icon */}
        <div className="plugin-card__image">
          {plugin.screenshots[0] ? (
            <Image
              src={plugin.screenshots[0]}
              alt={plugin.name}
              fill
              className="object-cover"
            />
          ) : (
            <div className="plugin-card__placeholder">
              <Package size={48} />
            </div>
          )}

          {/* Pricing Badge */}
          <div className="plugin-card__badge">
            {getPricingBadge()}
          </div>
        </div>

        {/* Plugin Info */}
        <div className="plugin-card__content">
          <h3 className="plugin-card__title">{plugin.name}</h3>

          <p className="plugin-card__description">
            {plugin.description.substring(0, 120)}
            {plugin.description.length > 120 && '...'}
          </p>

          {/* Author */}
          <div className="plugin-card__author">
            <span>by {plugin.author.name}</span>
          </div>

          {/* Categories */}
          {plugin.categories.length > 0 && (
            <div className="plugin-card__categories">
              {plugin.categories.slice(0, 2).map((category) => (
                <span key={category} className="category-tag">
                  <Tag size={12} />
                  {category}
                </span>
              ))}
            </div>
          )}

          {/* Stats */}
          <div className="plugin-card__stats">
            {/* Rating */}
            <div className="stat-item">
              <Star size={16} className="fill-yellow-400 text-yellow-400" />
              <span className="stat-value">
                {plugin.rating.average.toFixed(1)}
              </span>
              <span className="stat-count">({plugin.rating.count})</span>
            </div>

            {/* Downloads */}
            <div className="stat-item">
              <Download size={16} />
              <span className="stat-value">
                {formatDownloads(plugin.downloads)}
              </span>
            </div>
          </div>

          {/* Version */}
          <div className="plugin-card__version">
            v{plugin.latestVersion}
          </div>
        </div>
      </div>
    </Link>
  );
};
