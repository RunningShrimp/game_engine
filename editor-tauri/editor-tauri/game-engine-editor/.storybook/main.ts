import type { StorybookConfig } from '@storybook/react-vite';
import { mergeConfig } from 'vite';

const config: StorybookConfig = {
  stories: [
    '../src/**/*.mdx',
    '../src/**/*.stories.@(js|jsx|mjs|ts|tsx)',
  ],
  addons: [
    '@storybook/addon-essentials',
    '@storybook/addon-a11y',
    '@storybook/addon-themes',
    '@storybook/addon-interactions',
  ],
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
  docs: {
    autodocs: 'tag',
  },
  async viteFinal(config, { configType }) {
    if (configType === 'DEVELOPMENT') {
      // Development-specific config
    }

    // Merge custom Vite config with Storybook's Vite config
    return mergeConfig(config, {
      // Add custom Vite config here
      resolve: {
        alias: {
          '@': '/src',
          '@components': '/src/components',
          '@utils': '/src/utils',
          '@hooks': '/src/hooks',
        },
      },
    });
  },
  typescript: {
    reactDocgen: 'react-docgen-typescript',
    reactDocgenTypescriptOptions: {
      shouldExtractLiteralValuesFromEnum: true,
      propFilter: (prop) => {
        return prop.parent ? !/node_modules/.test(prop.parent.fileName) : true;
      },
    },
  },
};

export default config;
