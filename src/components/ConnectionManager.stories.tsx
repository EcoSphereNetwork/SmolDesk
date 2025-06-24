import type { Meta, StoryObj } from '@storybook/react';
import ConnectionManager from './ConnectionManager';

const meta: Meta<typeof ConnectionManager> = {
  title: 'Components/ConnectionManager',
  component: ConnectionManager,
  args: { signalingServer: 'ws://localhost:5173' },
};
export default meta;

export type Story = StoryObj<typeof ConnectionManager>;

export const Default: Story = {};
