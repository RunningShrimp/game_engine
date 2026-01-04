# Storybook Quick Start Guide

Get started with Storybook in 3 simple steps!

## Step 1: Start Storybook

```bash
npm run storybook
```

Storybook will open at [http://localhost:6006](http://localhost:6006)

## Step 2: Explore Components

Navigate through the sidebar:

- **Atoms**: Basic building blocks (Button, Spinner, EmptyState, Skeleton)
- **Molecules**: Simple combinations (Input, Checkbox)
- **Organisms**: Complex components (FormCard)

## Step 3: Interact & Test

For each component:

1. **Controls Panel**: Use the controls tab to modify props in real-time
2. **Canvas**: See the component update instantly
3. **Docs**: View auto-generated documentation
4. **Accessibility**: Check the Accessibility tab for violations

## Common Commands

```bash
# Start development server
npm run storybook

# Build static site
npm run build-storybook

# Run validation
./.storybook/test-stories.sh
```

## Creating New Stories

1. Create a file next to your component: `MyComponent.stories.tsx`
2. Use this template:

```tsx
import type { Meta, StoryObj } from '@storybook/react';
import { MyComponent } from './MyComponent';

const meta: Meta<typeof MyComponent> = {
  title: 'Category/MyComponent',
  component: MyComponent,
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof MyComponent>;

export const Default: Story = {
  args: {
    // props here
  },
};
```

3. Storybook will auto-reload with your new story!

## Resources

- 📖 Full Guide: [`STORYBOOK.md`](./STORYBOOK.md)
- 📊 Setup Summary: [`STORYBOOK_SETUP_SUMMARY.md`](./STORYBOOK_SETUP_SUMMARY.md)
- 🎨 Storybook Docs: https://storybook.js.org/docs

## Tips

- Use the **Controls** addon to test all prop combinations
- Check the **Accessibility** tab for WCAG compliance
- Switch themes using the toolbar icon (☀️/🌙)
- Test responsive design with the **Viewport** addon
- Enable **Docs mode** for detailed documentation view

Happy component building! 🚀
