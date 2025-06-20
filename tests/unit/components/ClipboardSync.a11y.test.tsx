import React from 'react';
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { composeStory } from '@storybook/testing-react';
import * as stories from '../../../src/components/ClipboardSync.stories';

const Default = composeStory(stories.Default, stories.default);
expect.extend(toHaveNoViolations);

test.skip('ClipboardSync is accessible', async () => {
  const { container } = render(<Default />);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
