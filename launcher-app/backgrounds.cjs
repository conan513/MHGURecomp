const backgrounds = Object.freeze(['1.webp', '2.webp', '3.webp', '4.webp']);

function selectBackground(random = Math.random) {
  return backgrounds[Math.floor(random() * backgrounds.length)];
}

module.exports = { backgrounds, selectBackground };
