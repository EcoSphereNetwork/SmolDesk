import React from 'react';
import { render } from '@testing-library/react';
import { composeStory } from '@storybook/testing-react';
import * as stories from '../../../src/components/RemoteScreen.stories';

declare module '@storybook/react' { interface StoryObj<TArgs = any> { args?: Partial<TArgs>; } }

const Default = composeStory(stories.Default, stories.default);

test.skip('renders RemoteScreen', () => {
  const { container } = render(<Default />);
  expect(container.innerHTML).toMatchSnapshot();
});
