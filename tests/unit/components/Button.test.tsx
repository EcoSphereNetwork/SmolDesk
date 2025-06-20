import React from 'react';
import { composeStory } from '@storybook/testing-react';
import * as stories from '../../../src/components/Button.stories';
import { render, screen } from '@testing-library/react';

declare module '@storybook/react' {
  interface StoryObj<TArgs = any> {
    args?: Partial<TArgs>;
  }
}

const Primary = composeStory(stories.Primary, stories.default);

test('renders button with label', () => {
  render(<Primary />);
  expect(screen.getByRole('button')).toHaveTextContent('Submit');
  expect(document.body.innerHTML).toMatchSnapshot();
});
