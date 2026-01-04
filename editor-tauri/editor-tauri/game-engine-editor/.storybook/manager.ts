import { addons } from '@storybook/manager-api';
import { Themes } from './theme';

/**
 * Storybook Manager Configuration
 *
 * Configures the Storybook UI sidebar and other manager-level settings.
 */
addons.setConfig({
  theme: Themes.light,
  panelPosition: 'right',
  sidebar: {
    showRoots: true,
    collapsed: false,
  },
  toolbar: {
    title: 'Game Engine Editor',
    icon: undefined,
  },
});
