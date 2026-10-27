(function registerFigures(global) {
  'use strict';
  function render(container, plots) {
    container.replaceChildren();
      const NS = 'http://www.w3.org/2000/svg';
      const svgElement = (tag, attrs = {}, text = '') => {
        const node = document.createElementNS(NS, tag); Object.entries(attrs).forEach(([key, value]) => node.setAttribute(key, value)); node.textContent = text; node.dataset.noI18n = ''; return node;
      };
      plots.forEach((plot, index) => {
        const figure = document.createElement('figure'); figure.className = 'studio-figure';
        const caption = document.createElement('figcaption'); caption.append(document.createTextNode(`Figura ${index + 1} · `));
        const plotTitle = document.createElement('span');
        if (plot.title) plotTitle.dataset.noI18n = '';
        plotTitle.textContent = plot.title || 'Gráfico'; caption.append(plotTitle);
        const svg = svgElement('svg', { viewBox: '0 0 680 340', role: 'img', 'aria-label': plot.title || 'Gráfico do script' });
        const all = plot.series.flatMap((series) => series.points).filter((point) => point.every(Number.isFinite));
        if (!all.length) return;
        let xmin = Infinity, xmax = -Infinity, ymin = Infinity, ymax = -Infinity;
        all.forEach(([x, y]) => { xmin = Math.min(xmin, x); xmax = Math.max(xmax, x); ymin = Math.min(ymin, y); ymax = Math.max(ymax, y); });
        if (xmin === xmax) { xmin -= 1; xmax += 1; } if (ymin === ymax) { ymin -= 1; ymax += 1; }
        const x = (v) => 62 + (v - xmin) / (xmax - xmin) * 590;
        const y = (v) => 292 - (v - ymin) / (ymax - ymin) * 265;
        svg.append(svgElement('path', { d: 'M62 20V292H652', fill: 'none', class: 'plot-axis' }));
        for (let i = 0; i <= 4; i++) {
          const xv = xmin + (xmax - xmin) * i / 4, yv = ymin + (ymax - ymin) * i / 4;
          svg.append(svgElement('text', { x: x(xv), y: 310, 'text-anchor': 'middle' }, Number(xv.toPrecision(4)).toString()));
          svg.append(svgElement('text', { x: 55, y: y(yv) + 4, 'text-anchor': 'end' }, Number(yv.toPrecision(4)).toString()));
        }
        svg.append(svgElement('text', { x: 357, y: 333, 'text-anchor': 'middle' }, plot.xLabel || 'x'));
        svg.append(svgElement('text', { x: 13, y: 158, transform: 'rotate(-90 13 158)', 'text-anchor': 'middle' }, plot.yLabel || 'y'));
        plot.series.forEach((series) => {
          const stride = Math.max(1, Math.ceil(series.points.length / 3000));
          const points = series.points.filter((_, i) => i % stride === 0 || i === series.points.length - 1).map(([a, b]) => `${x(a)},${y(b)}`).join(' ');
          const curve = svgElement('polyline', { points, fill: 'none', stroke: `rgb(${series.color.join(',')})`, 'stroke-width': 2, 'stroke-linejoin': 'round' });
          curve.append(svgElement('title', {}, series.name)); svg.append(curve);
        });
        const tools = document.createElement('div'); tools.className = 'studio-figure-tools';
        const download = document.createElement('button'); download.type = 'button'; download.textContent = 'Salvar SVG';
        download.addEventListener('click', () => {
          const copy = svg.cloneNode(true); copy.setAttribute('xmlns', NS);
          // Export colors explicitly because the application's CSS is not embedded in SVG.
          copy.querySelectorAll('text').forEach((node) => { node.setAttribute('fill', '#425466'); node.setAttribute('font-size', '11'); });
          copy.querySelectorAll('.plot-axis').forEach((node) => node.setAttribute('stroke', '#a0acb8'));
          const url = URL.createObjectURL(new Blob([new XMLSerializer().serializeToString(copy)], { type: 'image/svg+xml' }));
          const link = document.createElement('a'); link.href = url; link.download = `figura-${index + 1}.svg`; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
        });
        tools.append(download); figure.append(caption, svg, tools); container.append(figure);
      });
    }
  global.ControLABFigures = { render };
})(window);
