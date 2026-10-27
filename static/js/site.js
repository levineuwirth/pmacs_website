// site.js — the site's one script: the ceiling switch and the midnight ring.
// Without it the site stays at night, every page reads whole, and the ring
// is simply not drawn.
(() => {
  const root = document.documentElement;
  const KEY = "pmacs-ceiling";

  const current = () => (root.dataset.ceiling === "day" || root.dataset.ceiling === "grime" ? root.dataset.ceiling : "night");
  const buttons = document.querySelectorAll("[data-ceiling-choice]");
  const sync = () => buttons.forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.ceilingChoice === current())));
  buttons.forEach((b) =>
    b.addEventListener("click", () => {
      const choice = b.dataset.ceilingChoice;
      if (choice === "night") delete root.dataset.ceiling;
      else root.dataset.ceiling = choice;
      try {
        localStorage.setItem(KEY, choice);
      } catch (e) {
        // Storage refused: the choice lasts this visit.
      }
      sync();
    }),
  );
  sync();

  // The ring: the point on the ecliptic due south at local midnight tonight,
  // opposite the sun, worked out from the date alone.
  const OBLIQUITY = (23.4393 * Math.PI) / 180;
  const days = Date.now() / 86400000 + 2440587.5 - 2451545.0;
  const meanLongitude = 280.46 + 0.9856474 * days;
  const anomaly = ((357.528 + 0.9856003 * days) * Math.PI) / 180;
  const sun = meanLongitude + 1.915 * Math.sin(anomaly) + 0.02 * Math.sin(2 * anomaly);
  const lambda = (((((sun + 180) % 360) + 360) % 360) * Math.PI) / 180;
  const ra = ((((Math.atan2(Math.sin(lambda) * Math.cos(OBLIQUITY), Math.cos(lambda)) * 180) / Math.PI / 15) % 24) + 24) % 24;
  const dec = (Math.asin(Math.sin(OBLIQUITY) * Math.sin(lambda)) * 180) / Math.PI;

  document.querySelectorAll("svg.sky[data-ra0]").forEach((svg) => {
    const ring = svg.querySelector(".ring");
    if (!ring) return;
    const d = svg.dataset;
    let u = ((((ra - Number(d.ra0)) % 24) + 24) % 24) * 15;
    if (u > 300) u -= 360;
    const x = (u * Number(d.width)) / Number(d.span);
    const y = (Number(d.decTop) - dec) * Number(d.pyd);
    if (x < 8 || x > Number(d.width) - 8) return;
    ring.setAttribute("transform", `translate(${x.toFixed(1)} ${y.toFixed(1)})`);
    ring.style.display = "";
  });
})();
