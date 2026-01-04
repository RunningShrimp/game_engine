# Storybook Component Documentation System

This document provides a comprehensive guide to using Storybook for component development and documentation in the Game Engine Editor.

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [Available Scripts](#available-scripts)
- [Project Structure](#project-structure)
- [Writing Stories](#writing-stories)
- [Best Practices](#best-practices)
- [Addons](#addons)
- [Theming](#theming)
- [Accessibility](#accessibility)
- [Testing](#testing)

## Overview

Storybook is an open source tool for developing UI components in isolation. It provides a sandbox for building components outside of the main application, making it easier to:

- Develop components in isolation
- Document component usage
- Test component variations and edge cases
- Ensure visual consistency
- Facilitate collaboration between designers and developers

## Getting Started

### Installation

All dependencies are already installed. To start the Storybook development server:

```bash
npm run storybook
```

This will start Storybook at [http://localhost:6006](http://localhost:6006).

### Building for Production

To build a static version of Storybook for deployment:

```bash
npm run build-storybook
```

This creates a `storybook-static` directory with the compiled documentation.

## Available Scripts

| Command | Description |
|---------|-------------|
| `npm run storybook` | Start Storybook development server on port 6006 |
| `npm run build-storybook` | Build Storybook as a static site |
| `npm run storybook:docs` | Start Storybook in docs-only mode |

## Project Structure

```
.storybook/
├── main.ts              # Storybook configuration
├── preview.ts           # Global decorators and parameters
├── theme.ts             # Custom theme configuration
└── manager.ts           # UI configuration

src/
├── components/
│   ├── atoms/           # Basic building blocks (Button, Spinner, etc.)
│   │   ├── Button.tsx
│   │   └── Button.stories.tsx
│   ├── molecules/       # Simple combinations (Input, Checkbox, etc.)
│   │   ├── Input/
│   │   │   ├── index.tsx
│   │   │   └── Input.stories.tsx
│   └── organisms/       # Complex components
│       └── Component.stories.tsx
```

## Writing Stories

### Basic Story Structure

Every component should have a corresponding `.stories.tsx` file:

```tsx
import type { Meta, StoryObj } from '@storybook/react';
import { Button } from './Button';

const meta: Meta<typeof Button> = {
  title: 'Atoms/Button',
  component: Button,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: 'Component description here...',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    // Control configurations
  },
};

export default meta;
type Story = StoryObj<typeof Button>;

export const Default: Story = {
  args: {
    // Component props
  },
};
```

### Story Types

#### 1. Default Story

Shows the most common usage:

```tsx
export const Default: Story = {
  args: {
    variant: 'primary',
    size: 'md',
    children: 'Click me',
  },
};
```

#### 2. Variant Showcase

Demonstrates all visual variations:

```tsx
export const Variants: Story = {
  render: () => (
    <div className="flex gap-4">
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="outline">Outline</Button>
    </div>
  ),
};
```

#### 3. State Stories

Show different states (loading, error, disabled):

```tsx
export const Loading: Story = {
  args: {
    isLoading: true,
    children: 'Loading...',
  },
};

export const Error: Story = {
  args: {
    error: true,
    errorMessage: 'Something went wrong',
  },
};
```

#### 4. Interactive Story

Enable controls for real-time experimentation:

```tsx
export const Interactive: Story = {
  args: {
    variant: 'primary',
    size: 'md',
    // All configurable props
  },
};
```

### Component Documentation Template

```tsx
const meta: Meta<typeof Component> = {
  title: 'Category/Component',
  component: Component,
  parameters: {
    layout: 'centered', // or 'padded', 'fullscreen'
    docs: {
      description: {
        component: `
**Component Purpose:**

Brief description of what this component does.

**Features:**
- Feature 1
- Feature 2
- Feature 3

**When to Use:**
- Use case 1
- Use case 2

**Accessibility:**
- A11y feature 1
- A11y feature 2
        `,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    prop: {
      control: 'select' | 'text' | 'boolean' | 'number',
      options: ['option1', 'option2'],
      description: 'Prop description',
    },
  },
};
```

## Best Practices

### 1. Story Organization

- **One component per story file**: Match the component name (e.g., `Button.tsx` → `Button.stories.tsx`)
- **Group by hierarchy**: Use title hierarchy (Atoms/Button, Molecules/Input, etc.)
- **Alphabetical order**: Keep stories in alphabetical order within files

### 2. Story Coverage

Every component should include:

- ✅ **Default story**: Most common usage
- ✅ **All variants**: Color, size, style variations
- ✅ **All states**: Normal, hover, active, disabled, loading, error
- ✅ **Edge cases**: Empty content, long text, special characters
- ✅ **Interactive story**: With controls enabled
- ✅ **Real examples**: Practical use cases

### 3. Documentation

- **Description component**: Add at meta level for component overview
- **Story descriptions**: Add context for specific stories
- **Code examples**: Include usage examples in JSDoc comments
- **Props documentation**: Document all prop options in argTypes

### 4. Layout Selection

Choose appropriate layout for each story:

- `centered`: For small components (buttons, inputs)
- `padded`: For form components, cards
- `fullscreen`: For modals, pages, full-screen components

### 5. Performance

- **Use args for variations**: Instead of multiple similar stories
- **Memoize complex examples**: For expensive computations
- **Lazy load heavy examples**: Use decorators when needed

## Addons

### Essential Addons

#### @storybook/addon-essentials

Includes:
- **Actions**: Logs click/hover events
- **Controls**: Interactive props editing
- **Docs**: Automatic documentation generation
- **Viewport**: Test responsive designs
- **Toolbar**: Configure Storybook UI

Usage:
```tsx
export const WithControls: Story = {
  args: {
    // Configurable via controls panel
  },
};
```

#### @storybook/addon-a11y

Accessibility testing with axe-core:

```tsx
parameters: {
  a11y: {
    config: {
      rules: [
        { id: 'color-contrast', enabled: true },
        { id: 'label', enabled: true },
      ],
    },
  },
},
```

Features:
- Automatic accessibility audits
- WCAG compliance checking
- Visual violation highlights
- Detailed violation explanations

#### @storybook/addon-themes

Theme switching support:

```tsx
globalTypes: {
  theme: {
    description: 'Global theme for components',
    defaultValue: 'light',
    toolbar: {
      title: 'Theme',
      items: [
        { value: 'light', icon: 'sun', title: 'Light' },
        { value: 'dark', icon: 'moon', title: 'Dark' },
      ],
    },
  },
},
```

#### @storybook/addon-interactions

Visual testing of user interactions:

```tsx
export const WithInteraction: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole('button'));
  },
};
```

## Theming

### Custom Theme

The project uses a custom Storybook theme defined in `.storybook/theme.ts`:

```tsx
export const Themes = {
  light: create({
    base: 'light',
    brandTitle: 'Game Engine Editor',
    colorPrimary: '#6366f1',
    // ... theme configuration
  }),
  dark: create({
    base: 'dark',
    // ... dark theme configuration
  }),
};
```

### Component Theming

Components automatically respond to theme changes:

```tsx
decorators: [
  withThemeByDataAttribute({
    themes: {
      light: 'light',
      dark: 'dark',
    },
    defaultTheme: 'light',
    attributeName: 'data-theme',
  }),
],
```

## Accessibility

### A11y Best Practices

1. **Semantic HTML**: Use proper HTML elements
2. **ARIA Labels**: Add labels to interactive elements
3. **Keyboard Navigation**: Ensure all interactions work via keyboard
4. **Color Contrast**: Meet WCAG AA standards (4.5:1)
5. **Focus Indicators**: Visible focus states
6. **Screen Reader Support**: Test with screen readers

### A11y Testing

Each story is automatically tested for accessibility:

```tsx
export const AccessibleButton: Story = {
  args: {
    'aria-label': 'Close dialog',
    children: 'Close',
  },
};
```

Check the "Accessibility" tab in Storybook for violations.

## Testing

### Visual Testing

Storybook provides visual regression testing:

```bash
npm run build-storybook
```

Integrate with Chromatic or similar tools for automated visual testing.

### Interaction Testing

Test component interactions with play functions:

```tsx
export const Clickable: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const button = canvas.getByRole('button');
    await userEvent.click(button);
    await expect(button).toHaveFocus();
  },
};
```

### Unit Testing

Stories can be used as test cases:

```tsx
import { composeStories } from '@storybook/react';
import * as stories from './Button.stories';

const { Default, Loading } = composeStories(stories);

test('renders default button', () => {
  render(<Default />);
  expect(screen.getByRole('button')).toBeInTheDocument();
});
```

## Component Checklist

Before marking a component as complete:

### Documentation
- [ ] Component has comprehensive description
- [ ] All props are documented
- [ ] Usage examples provided
- [ ] Accessibility notes included

### Stories
- [ ] Default story exists
- [ ] All variants showcased
- [ ] All states demonstrated
- [ ] Interactive story with controls
- [ ] Edge cases covered
- [ ] Real-world use cases included

### Quality
- [ ] No console errors
- [ ] All accessibility checks pass
- [ ] Works in both light and dark themes
- [ ] Responsive design tested
- [ ] Keyboard navigation works
- [ ] Screen reader tested

### Code Quality
- [ ] TypeScript types correct
- [ ] No ESLint warnings
- [ ] Proper JSDoc comments
- [ ] Clean, readable code
- [ ] Follows project conventions

## Troubleshooting

### Common Issues

#### Storybook won't start

```bash
# Clear cache and reinstall
rm -rf node_modules/.cache
npm install
npm run storybook
```

#### Styles not loading

Ensure `preview.ts` imports CSS:
```tsx
import '../src/index.css';
```

#### Component not found

Check import path in story file:
```tsx
import { Component } from './Component'; // Relative path
```

#### Type errors

Restart TypeScript server in VS Code:
`Cmd+Shift+P` → "TypeScript: Restart TS Server"

## Resources

- [Official Storybook Docs](https://storybook.js.org/docs)
- [Storybook for React](https://storybook.js.org/docs/react/get-started/introduction)
- [Best Practices](https://storybook.js.org/docs/writing-stories/best-practices)
- [Testing](https://storybook.js.org/docs/writing-tests)
- [Addons](https://storybook.js.org/docs/addons/addon-essentials)

## Contributing

When adding new components:

1. Create component in appropriate directory
2. Add comprehensive TypeScript types
3. Include JSDoc comments with examples
4. Create `.stories.tsx` file following this guide
5. Test in Storybook before committing
6. Verify accessibility
7. Update this checklist if needed

---

**Last Updated**: 2026-01-04
**Storybook Version**: 8.6.x
**Maintainer**: Game Engine Editor Team
