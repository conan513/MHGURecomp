import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import pngToIco from 'png-to-ico';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const iconPng = path.join(root, 'backgrounds', 'launcher-icon.png');
const iconIco = path.join(root, 'src-tauri', 'icons', 'icon.ico');

fs.mkdirSync(path.dirname(iconIco), { recursive: true });
fs.writeFileSync(iconIco, await pngToIco(iconPng));
