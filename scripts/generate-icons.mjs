import { PNG } from 'pngjs';
import { mkdirSync, writeFileSync } from 'node:fs';

mkdirSync(new URL('../web/icons/', import.meta.url), { recursive: true });
for (const [name, size] of [['icon-192', 192], ['icon-512', 512], ['maskable-512', 512], ['apple-touch-icon', 180]]) {
  const png = new PNG({ width: size, height: size });
  // All artwork fits inside the maskable icon's central safe circle.
  const lines = [
    [40, 38, 25, 38], [25, 38, 20, 44], [20, 44, 20, 58],
    [20, 58, 25, 63], [25, 63, 42, 63], [42, 63, 42, 51], [42, 51, 33, 51],
    [53, 38, 65, 63], [65, 63, 78, 38],
  ];
  const distance = (x, y, [ax, ay, bx, by]) => {
    const dx = bx - ax, dy = by - ay;
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)));
    return Math.hypot(x - ax - t * dx, y - ay - t * dy);
  };
  for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
    const px = (x + .5) * 100 / size, py = (y + .5) * 100 / size;
    const d = Math.min(...lines.map(line => distance(px, py, line)));
    const alpha = Math.max(0, Math.min(1, (2.3 - d) * size / 100));
    const fg = px < 48 ? [0, 212, 255] : [244, 95, 207];
    const i = (y * size + x) * 4;
    [8, 13, 28].forEach((v, c) => { png.data[i + c] = Math.round(v + (fg[c] - v) * alpha); });
    png.data[i + 3] = 255;
  }
  writeFileSync(new URL(`../web/icons/${name}.png`, import.meta.url), PNG.sync.write(png));
}
