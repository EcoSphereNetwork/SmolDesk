import React from 'react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { render } from '@testing-library/react';
import { composeStory } from '@storybook/testing-react';
import * as stories from '../../../src/components/Button.stories';

const Primary = composeStory(stories.Primary, stories.default);

expect.extend(toHaveNoViolations);

test('Button is accessible', async () => {
  const { container } = render(<Primary />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
