import type { StorybookConfig } from '@storybook/react-vite';

const config: StorybookConfig = {
  stories: ['../src/components/**/*.stories.@(ts|tsx)'],
  addons: ['@storybook/addon-interactions'],
  framework: {
    name: '@storybook/react-vite',
    options: {}
  }
};

export default config;
