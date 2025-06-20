import type { Meta, StoryObj } from '@storybook/react';
import FileTransfer from './FileTransfer';

const meta: Meta<typeof FileTransfer> = {
  title: 'Components/FileTransfer',
  component: FileTransfer,
};
export default meta;

export type Story = StoryObj<typeof FileTransfer>;

export const Default: Story = {};
