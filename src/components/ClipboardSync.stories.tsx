import type { Meta, StoryObj } from '@storybook/react';
import ClipboardSync from './ClipboardSync';

const meta: Meta<typeof ClipboardSync> = {
  title: 'Components/ClipboardSync',
  component: ClipboardSync,
};
export default meta;

export type Story = StoryObj<typeof ClipboardSync>;

export const Default: Story = {};
