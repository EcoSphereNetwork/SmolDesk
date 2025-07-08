import { getTheme } from '../src/theme';

it('returns dark theme when scheme is dark', () => {
  const t: any = getTheme('dark');
  expect(t.dark).toBe(true);
});

it('returns light theme when scheme is light', () => {
  const t: any = getTheme('light');
  expect(t.dark).toBe(false);
});
