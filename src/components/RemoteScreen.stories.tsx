import type { Meta, StoryObj } from '@storybook/react';
import RemoteScreen from './RemoteScreen';

const meta: Meta<typeof RemoteScreen> = {
  title: 'Components/RemoteScreen',
  component: RemoteScreen,
  args: { isConnected: false },
};
export default meta;

export type Story = StoryObj<typeof RemoteScreen>;

export const Default: Story = {};
