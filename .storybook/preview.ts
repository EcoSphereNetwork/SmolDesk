import type { Preview } from '@storybook/react';

// expose version and commit info inside Storybook docs
const commit = import.meta.env.VITE_COMMIT_SHA ?? 'dev';
const version = import.meta.env.VITE_VERSION ?? '0.0.0';

const preview: Preview = {
  parameters: {
    actions: { argTypesRegex: '^on.*' },
    docs: {
      description: {
        component: `Version: ${version} (${commit.slice(0, 7)})`,
      },
    },
  },
};

export default preview;
