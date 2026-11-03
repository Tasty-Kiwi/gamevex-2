const canvas = "#gamevex";

// Rust publishes logical surface bounds. Convert those to CSS points by
// inverting the same radial CRT mapping used by pointer input.
export async function screenPoint(page, x, y) {
  return page.locator(canvas).evaluate((el, p) => {
    const rect = el.getBoundingClientRect();
    const k = Number(el.dataset.curvature || 0);
    const qx = (p.x / rect.width) * 2 - 1;
    const qy = (p.y / rect.height) * 2 - 1;
    const radius = Math.hypot(qx, qy);
    let lo = 0;
    let hi = Math.max(1, radius);
    for (let i = 0; i < 32; i++) {
      const r = (lo + hi) / 2;
      if (r * (1 + k * r * r) < radius) lo = r;
      else hi = r;
    }
    const scale = radius ? ((lo + hi) / 2) / radius : 1;
    return {
      x: rect.left + rect.width * (1 + qx * scale) / 2,
      y: rect.top + rect.height * (1 + qy * scale) / 2,
    };
  }, { x, y });
}
