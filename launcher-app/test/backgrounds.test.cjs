const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const { backgrounds, selectBackground } = require('../backgrounds.cjs');

test('selects each packaged background across the random range', () => {
  assert.deepEqual(
    backgrounds.map((_, index) => selectBackground(() => index / backgrounds.length)),
    backgrounds,
  );
  assert.equal(new Set(backgrounds).size, 4);
});

test('all selected backgrounds and launcher artwork are present', () => {
  const assetsDirectory = path.resolve(__dirname, '..', '..', 'backgrounds');
  const images = [...backgrounds, 'logo.webp', 'launcher-icon.png'];
  let totalBytes = 0;

  for (const filename of images) {
    const imagePath = path.join(assetsDirectory, filename);
    const image = fs.readFileSync(imagePath);
    totalBytes += image.length;
    assert.ok(fs.statSync(imagePath).isFile(), `${filename} must exist`);
    if (filename.endsWith('.webp')) {
      assert.equal(image.toString('ascii', 0, 4), 'RIFF');
      assert.equal(image.toString('ascii', 8, 12), 'WEBP');
    }
  }

  assert.ok(totalBytes < 2_500_000, `optimized artwork must stay compact (${totalBytes} bytes)`);
});
