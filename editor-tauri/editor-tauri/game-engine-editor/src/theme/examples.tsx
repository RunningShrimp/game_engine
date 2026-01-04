/**
 * Design Tokens Usage Examples
 *
 * This file demonstrates how to use design tokens in your components.
 * Refer to this file when implementing new UI components.
 */

import React from 'react';
import {
  colors,
  typography,
  spacing,
  borderRadius,
  shadows,
  zIndex,
  transitions,
} from '@/theme';

/**
 * Example 1: Using Design Tokens with Inline Styles
 */
export const InlineStyleExample: React.FC = () => {
  return (
    <div
      style={{
        backgroundColor: colors.primary[500],
        color: '#ffffff',
        padding: spacing.space[4],
        borderRadius: borderRadius.borderRadius.lg,
        boxShadow: shadows.shadows.md,
        fontFamily: typography.fontFamily.sans,
        fontSize: typography.fontSize.base,
        fontWeight: typography.fontWeight.medium,
      }}
    >
      Styled with design tokens
    </div>
  );
};

/**
 * Example 2: Using Typography Presets
 */
export const TypographyPresetExample: React.FC = () => {
  return (
    <div>
      <h1 style={{ ...typography.presets.h1, marginBottom: spacing.space[4] }}>
        Heading 1
      </h1>
      <h2 style={{ ...typography.presets.h2, marginBottom: spacing.space[3] }}>
        Heading 2
      </h2>
      <h3 style={{ ...typography.presets.h3, marginBottom: spacing.space[2] }}>
        Heading 3
      </h3>
      <p style={{ ...typography.presets.body, marginBottom: spacing.space[4] }}>
        Body text with normal styling
      </p>
      <p style={{ ...typography.presets['body-sm'], color: colors.semantic.text.secondary }}>
        Small body text with secondary color
      </p>
    </div>
  );
};

/**
 * Example 3: Using Semantic Colors
 */
export const SemanticColorExample: React.FC = () => {
  return (
    <div style={{ display: 'flex', gap: spacing.space[4], padding: spacing.space[4] }}>
      <div
        style={{
          backgroundColor: colors.semantic.background.paper,
          padding: spacing.space[4],
          borderRadius: borderRadius.borderRadius.md,
          border: `1px solid ${colors.semantic.border.DEFAULT}`,
        }}
      >
        <p style={{ color: colors.semantic.text.primary }}>Primary text</p>
        <p style={{ color: colors.semantic.text.secondary }}>Secondary text</p>
        <p style={{ color: colors.semantic.text.disabled }}>Disabled text</p>
      </div>

      <div
        style={{
          backgroundColor: colors.error[50],
          padding: spacing.space[4],
          borderRadius: borderRadius.borderRadius.md,
          border: `1px solid ${colors.semantic.border.error}`,
        }}
      >
        <p style={{ color: colors.semantic.text.primary }}>Error state</p>
      </div>

      <div
        style={{
          backgroundColor: colors.success[50],
          padding: spacing.space[4],
          borderRadius: borderRadius.borderRadius.md,
          border: `1px solid ${colors.semantic.border.success}`,
        }}
      >
        <p style={{ color: colors.semantic.text.primary }}>Success state</p>
      </div>
    </div>
  );
};

/**
 * Example 4: Using Spacing Presets
 */
export const SpacingExample: React.FC = () => {
  return (
    <div
      style={{
        padding: spacing.spacing['card-padding'],
        gap: spacing.spacing['content-gap'],
      }}
    >
      <div style={{ marginBottom: spacing.spacing['element-margin'] }}>
        Content with element margin
      </div>
      <div style={{ marginBottom: spacing.spacing['tight-gap'] }}>
        Content with tight gap
      </div>
    </div>
  );
};

/**
 * Example 5: Using Component-Specific Tokens
 */
export const ComponentTokensExample: React.FC = () => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: spacing.space[4], padding: spacing.space[8] }}>
      {/* Button example */}
      <button
        style={{
          ...typography.presets.button,
          padding: spacing.spacing['button-padding'],
          borderRadius: borderRadius.componentRadius.button,
          backgroundColor: colors.primary[500],
          color: '#ffffff',
          border: 'none',
          cursor: 'pointer',
          boxShadow: shadows.componentShadows.button,
          transition: transitions.componentTransitions.button,
        }}
      >
        Primary Button
      </button>

      {/* Input example */}
      <input
        type="text"
        placeholder="Enter text..."
        style={{
          ...typography.presets.body,
          padding: spacing.spacing['input-padding'],
          borderRadius: borderRadius.componentRadius.input,
          border: `1px solid ${colors.semantic.border.DEFAULT}`,
          fontFamily: typography.fontFamily.sans,
          fontSize: typography.fontSize.base,
          transition: transitions.componentTransitions.input,
        }}
      />

      {/* Card example */}
      <div
        style={{
          padding: spacing.spacing['card-padding'],
          borderRadius: borderRadius.componentRadius.card,
          backgroundColor: colors.semantic.background.paper,
          boxShadow: shadows.componentShadows.card,
          border: `1px solid ${colors.semantic.border.DEFAULT}`,
        }}
      >
        <h3 style={{ ...typography.presets.h4, marginBottom: spacing.space[2] }}>
          Card Title
        </h3>
        <p style={typography.presets.body}>Card content goes here</p>
      </div>
    </div>
  );
};

/**
 * Example 6: Using Z-Index Layers
 */
export const ZIndexExample: React.FC = () => {
  return (
    <div style={{ position: 'relative', padding: spacing.space[8] }}>
      <div
        style={{
          position: 'absolute',
          zIndex: zIndex.zIndex.base,
          padding: spacing.space[4],
          backgroundColor: colors.neutral[300],
        }}
      >
        Base layer
      </div>
      <div
        style={{
          position: 'absolute',
          zIndex: zIndex.zIndex.dropdown,
          padding: spacing.space[4],
          backgroundColor: colors.primary[500],
          color: '#ffffff',
          left: spacing.space[8],
        }}
      >
        Dropdown layer
      </div>
      <div
        style={{
          position: 'absolute',
          zIndex: zIndex.zIndex.modal,
          padding: spacing.space[4],
          backgroundColor: colors.secondary[500],
          color: '#ffffff',
          left: spacing.space[16],
        }}
      >
        Modal layer
      </div>
    </div>
  );
};

/**
 * Example 7: Using CSS Variables (Runtime Theme Switching)
 */
export const CssVariablesExample: React.FC = () => {
  const toggleDarkMode = () => {
    document.documentElement.classList.toggle('dark');
  };

  return (
    <div
      style={{
        padding: 'var(--spacing-4)',
        backgroundColor: 'var(--color-background-paper)',
        color: 'var(--color-text-primary)',
        borderRadius: 'var(--radius-lg)',
        border: `1px solid var(--color-border)`,
      }}
    >
      <p style={{ fontFamily: 'var(--font-family-sans)', fontSize: 'var(--font-size-base)' }}>
        This component uses CSS variables for runtime theme switching.
      </p>
      <button
        onClick={toggleDarkMode}
        style={{
          marginTop: 'var(--spacing-4)',
          padding: 'var(--spacing-2) var(--spacing-4)',
          backgroundColor: 'var(--color-primary-500)',
          color: '#ffffff',
          border: 'none',
          borderRadius: 'var(--radius-md)',
          cursor: 'pointer',
          transition: `all var(--transition-base) var(--easing-smooth)`,
        }}
      >
        Toggle Dark Mode
      </button>
    </div>
  );
};

/**
 * Example 8: Responsive Design with Breakpoints
 */
export const ResponsiveExample: React.FC = () => {
  return (
    <div
      style={{
        padding: spacing.space[4],
        backgroundColor: colors.semantic.background.paper,
      }}
    >
      <p style={{ ...typography.presets.body }}>
        This example shows how to structure responsive layouts.
        <br />
        Breakpoints: xs (375px), sm (640px), md (768px), lg (1024px), xl (1280px)
      </p>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
          gap: spacing.space[4],
          marginTop: spacing.space[4],
        }}
      >
        {[1, 2, 3, 4].map((item) => (
          <div
            key={item}
            style={{
              padding: spacing.space[4],
              backgroundColor: colors.primary[50],
              borderRadius: borderRadius.borderRadius.lg,
              border: `1px solid ${colors.primary[200]}`,
            }}
          >
            Item {item}
          </div>
        ))}
      </div>
    </div>
  );
};

/**
 * Example 9: Combining Multiple Token Systems
 */
export const CombinedExample: React.FC = () => {
  return (
    <div
      style={{
        maxWidth: spacing.container.lg,
        margin: '0 auto',
        padding: spacing.space[8],
      }}
    >
      {/* Header */}
      <header style={{ marginBottom: spacing.spacing['section-gap'] }}>
        <h1 style={{ ...typography.presets['display-md'], marginBottom: spacing.space[4] }}>
          Dashboard
        </h1>
        <p style={{ ...typography.presets['body-lg'], color: colors.semantic.text.secondary }}>
          Welcome back! Here's your overview.
        </p>
      </header>

      {/* Stats Grid */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))',
          gap: spacing.space[6],
          marginBottom: spacing.spacing['section-gap'],
        }}
      >
        {[
          { label: 'Total Users', value: '1,234', color: colors.primary[500] },
          { label: 'Revenue', value: '$12,345', color: colors.success[500] },
          { label: 'Errors', value: '23', color: colors.error[500] },
          { label: 'Warnings', value: '5', color: colors.warning[500] },
        ].map((stat) => (
          <div
            key={stat.label}
            style={{
              padding: spacing.spacing['card-padding'],
              borderRadius: borderRadius.componentRadius.card,
              backgroundColor: colors.semantic.background.paper,
              boxShadow: shadows.componentShadows.card,
              border: `1px solid ${colors.semantic.border.DEFAULT}`,
              transition: transitions.componentTransitions.card,
            }}
          >
            <p
              style={{
                ...typography.presets.caption,
                color: colors.semantic.text.secondary,
                textTransform: 'uppercase',
                letterSpacing: typography.letterSpacing.wider,
                marginBottom: spacing.space[2],
              }}
            >
              {stat.label}
            </p>
            <p
              style={{
                ...typography.presets['display-sm'],
                color: stat.color,
                fontWeight: typography.fontWeight.bold,
              }}
            >
              {stat.value}
            </p>
          </div>
        ))}
      </div>

      {/* Content Section */}
      <section
        style={{
          padding: spacing.spacing['card-padding'],
          borderRadius: borderRadius.componentRadius.card,
          backgroundColor: colors.semantic.background.paper,
          border: `1px solid ${colors.semantic.border.DEFAULT}`,
        }}
      >
        <h2 style={{ ...typography.presets.h3, marginBottom: spacing.space[4] }}>
          Recent Activity
        </h2>
        <div style={{ display: 'flex', flexDirection: 'column', gap: spacing.space[3] }}>
          {[1, 2, 3].map((item) => (
            <div
              key={item}
              style={{
                padding: spacing.space[4],
                borderRadius: borderRadius.borderRadius.md,
                backgroundColor: colors.neutral[50],
                border: `1px solid ${colors.semantic.border.DEFAULT}`,
              }}
            >
              <p style={typography.presets.body}>Activity item {item}</p>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
};

/**
 * Master Example Component
 * Demonstrates all design token categories working together
 */
export const DesignTokenShowcase: React.FC = () => {
  return (
    <div style={{ padding: spacing.space[8], backgroundColor: colors.semantic.background.DEFAULT }}>
      <h1 style={{ ...typography.presets['display-lg'], marginBottom: spacing.space[8] }}>
        Design Token Showcase
      </h1>

      <div style={{ display: 'flex', flexDirection: 'column', gap: spacing.spacing['section-gap'] }}>
        <section>
          <h2 style={{ ...typography.presets.h2, marginBottom: spacing.space[4] }}>
            Inline Styles
          </h2>
          <InlineStyleExample />
        </section>

        <section>
          <h2 style={{ ...typography.presets.h2, marginBottom: spacing.space[4] }}>
            Typography Presets
          </h2>
          <TypographyPresetExample />
        </section>

        <section>
          <h2 style={{ ...typography.presets.h2, marginBottom: spacing.space[4] }}>
            Semantic Colors
          </h2>
          <SemanticColorExample />
        </section>

        <section>
          <h2 style={{ ...typography.presets.h2, marginBottom: spacing.space[4] }}>
            Component Tokens
          </h2>
          <ComponentTokensExample />
        </section>

        <section>
          <h2 style={{ ...typography.presets.h2, marginBottom: spacing.space[4] }}>
            CSS Variables (Theme Switching)
          </h2>
          <CssVariablesExample />
        </section>

        <section>
          <h2 style={{ ...typography.presets.h2, marginBottom: spacing.space[4] }}>
            Combined Example
          </h2>
          <CombinedExample />
        </section>
      </div>
    </div>
  );
};

export default DesignTokenShowcase;
