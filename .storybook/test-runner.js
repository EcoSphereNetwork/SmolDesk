const { getStoryContext } = require('@storybook/test-runner');

module.exports = {
  async preRender(page, context) {
    const storyContext = await getStoryContext(page, context);
    await page.goto(context.parameters.fileName);
  },
  async postRender(page, context) {
    const name = context.title.replace(/\s+/g, '.');
    await page.screenshot({ path: `storybook-snapshots/${name}.png` });
  }
};
