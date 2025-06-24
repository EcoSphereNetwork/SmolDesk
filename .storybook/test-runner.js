module.exports = {
  async postRender(page, context) {
    const name = context.title.replace(/[\s/]+/g, '.');
    await page.screenshot({ path: `storybook-snapshots/${name}.png` });
  }
};
