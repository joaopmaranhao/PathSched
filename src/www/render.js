import { rotuloCurto } from './graph.js';

const RAIO = 24;
const SVG_NS = 'http://www.w3.org/2000/svg';

const CORES = {
  base: { preenchimento: '#1c2431', contorno: '#4a5a70', texto: '#e6edf3' },
  inicio: { preenchimento: '#2b1f4a', contorno: '#a371f7', texto: '#e6edf3' },
  objetivo: { preenchimento: '#4a2c14', contorno: '#f0883e', texto: '#e6edf3' },
  fronteira: { preenchimento: '#453113', contorno: '#d29922', texto: '#f2e5c4' },
  expandido: { preenchimento: '#12301c', contorno: '#2ea043', texto: '#c8f0d4' },
  atual: { preenchimento: '#14304f', contorno: '#58a6ff', texto: '#eaf2ff' },
  caminho: { preenchimento: '#0f3336', contorno: '#39c5cf', texto: '#d6f6f8' },
  solucao: { preenchimento: '#14532d', contorno: '#3fb950', texto: '#e6ffe9' },
  falha: { preenchimento: '#3d1518', contorno: '#f85149', texto: '#ffdcd9' },
};

export class Renderizador {
  constructor(svg, callbacks = {}) {
    this.svg = svg;
    this.callbacks = callbacks;
    this.grafo = null;
    this.estado = null;
    this.escala = 1;
    this.panorama = { x: 0, y: 0 };
    this.selecao = null;
    this.#ligarEventos();
  }

  tamanho() {
    const caixa = this.svg.getBoundingClientRect();
    return {
      largura: Math.max(320, caixa.width || 800),
      altura: Math.max(240, caixa.height || 520),
    };
  }

  desenhar(grafo, estado) {
    this.grafo = grafo;
    this.estado = estado ?? {};
    const { largura, altura } = this.tamanho();
    if (grafo && !grafo.posicionado()) grafo.aplicarLayout('circular', largura, altura);
    this.svg.setAttribute('viewBox', this.#viewBox(largura, altura));
    this.svg.replaceChildren();
    if (!grafo) return;
    this.svg.append(this.#camadaArestas(), this.#camadaNos(), this.#camadaTXT());
  }

  atualizar(estado) {
    this.estado = estado ?? {};
    if (this.grafo) this.desenhar(this.grafo, this.estado);
  }

  centralizar() {
    this.escala = 1;
    this.panorama = { x: 0, y: 0 };
    this.desenhar(this.grafo, this.estado);
  }

  novoPreset(grafo) {
    this.escala = 1;
    this.panorama = { x: 0, y: 0 };
    this.desenhar(grafo, {});
  }

  #viewBox(largura, altura) {
    const w = largura / this.escala;
    const h = altura / this.escala;
    const x = this.panorama.x + (largura - w) / 2;
    const y = this.panorama.y + (altura - h) / 2;
    return `${x} ${y} ${w} ${h}`;
  }

  #estadoDoNo(id) {
    const e = this.estado;
    if (e.solucao?.includes(id)) return 'solucao';
    if (e.atual === id) return 'atual';
    if (e.falhos?.has(id)) return 'falha';
    if (e.caminho?.includes(id)) return 'caminho';
    if (e.visitados?.has(id)) return 'expandido';
    if (e.fronteira?.has(id)) return 'fronteira';
    if (id === e.inicio) return 'inicio';
    if (id === e.objetivo) return 'objetivo';
    return 'base';
  }

  #noCaminho(de, para) {
    const c = this.estado.caminho ?? [];
    for (let i = 1; i < c.length; i += 1) {
      if (c[i - 1] === de && c[i] === para) return true;
      if (c[i - 1] === para && c[i] === de) return true;
    }
    return false;
  }

  #noSolucao(de, para) {
    const c = this.estado.solucao ?? [];
    for (let i = 1; i < c.length; i += 1) {
      if (c[i - 1] === de && c[i] === para) return true;
      if (c[i - 1] === para && c[i] === de) return true;
    }
    return false;
  }

  #camadaArestas() {
    const camada = this.#elemento('g', { class: 'arestas' });
    if (!this.grafo) return camada;

    const grupos = new Map();
    for (const aresta of this.grafo.edges) {
      const chave = this.grafo.directed
        ? `${aresta.from}>${aresta.to}`
        : [aresta.from, aresta.to].sort().join('|');
      if (!grupos.has(chave)) grupos.set(chave, []);
      grupos.get(chave).push(aresta);
    }

    for (const arestas of grupos.values()) {
      arestas.forEach((aresta, indice) => {
        const a = this.grafo.no(aresta.from);
        const b = this.grafo.no(aresta.to);
        if (!a || !b) return;
        const curva = arestas.length > 1 ? 26 * (indice % 2 === 0 ? 1 : -1) * (indice + 1) : 0;
        const { d, meio } = this.#geometria(a, b, curva);
        const naCaminho = this.#noCaminho(aresta.from, aresta.to);
        const naSolucao = this.#noSolucao(aresta.from, aresta.to);
        const cor = naSolucao ? CORES.solucao.contorno : naCaminho ? CORES.caminho.contorno : '#3d4a5e';
        const destaque = naSolucao || naCaminho;
        const seta = this.grafo.directed ? 'url(#seta)' : '';
        camada.append(
          this.#elemento('path', {
            d,
            fill: 'none',
            stroke: cor,
            'stroke-width': destaque ? 4 : 2,
            'stroke-dasharray': naCaminho && !naSolucao ? '7 5' : 'none',
            'marker-end': seta,
            opacity: destaque ? 1 : 0.75,
            class: 'aresta',
            'data-from': aresta.from,
            'data-to': aresta.to,
          }),
        );
        const larguraRotulo = 26;
        camada.append(
          this.#elemento('rect', {
            x: meio.x - larguraRotulo / 2,
            y: meio.y - 9,
            width: larguraRotulo,
            height: 17,
            rx: 5,
            fill: '#151b24',
            class: 'aresta-custo-bg',
          }),
        );
        camada.append(
          this.#elemento('text', {
            x: meio.x,
            y: meio.y + 3.5,
            'text-anchor': 'middle',
            fill: destaque ? cor : '#9aa7b8',
            class: 'aresta-rotulo',
          }, String(aresta.weight)),
        );
      });
    }
    return camada;
  }

  #geometria(a, b, curva) {
    const ax = a.x;
    const ay = a.y;
    const bx = b.x;
    const by = b.y;
    const dx = bx - ax;
    const dy = by - ay;
    const dist = Math.hypot(dx, dy) || 1;
    const ux = dx / dist;
    const uy = dy / dist;
    const x1 = ax + ux * (RAIO + 3);
    const y1 = ay + uy * (RAIO + 3);
    const x2 = bx - ux * (RAIO + 9);
    const y2 = by - uy * (RAIO + 9);
    if (curva === 0) {
      return { d: `M ${x1} ${y1} L ${x2} ${y2}`, meio: { x: (x1 + x2) / 2, y: (y1 + y2) / 2 } };
    }
    const mx = (x1 + x2) / 2;
    const my = (y1 + y2) / 2;
    const px = -uy * curva;
    const py = ux * curva;
    const cx = mx + px;
    const cy = my + py;
    return {
      d: `M ${x1} ${y1} Q ${cx} ${cy} ${x2} ${y2}`,
      meio: {
        x: (x1 + 2 * cx + x2) / 4,
        y: (y1 + 2 * cy + y2) / 4,
      },
    };
  }

  #camadaNos() {
    const camada = this.#elemento('g', { class: 'nos' });
    if (!this.grafo) return camada;
    for (const no of this.grafo.nodes) {
      const estilo = CORES[this.#estadoDoNo(no.id)];
      const grupo = this.#elemento('g', {
        class: no.id === this.estado.atual ? 'no novo' : 'no',
        'data-no': no.id,
        transform: `translate(${no.x} ${no.y})`,
      });
      grupo.append(
        this.#elemento('circle', {
          r: RAIO,
          fill: estilo.preenchimento,
          stroke: estilo.contorno,
          'stroke-width': no.id === this.estado.atual ? 3 : 2,
        }),
      );
      if (this.selecao === no.id) {
        grupo.append(
          this.#elemento('circle', {
            r: RAIO + 5,
            fill: 'none',
            stroke: '#ffffff',
            'stroke-width': 1.5,
            'stroke-dasharray': '4 3',
          }),
        );
      }
      camada.append(grupo);
    }
    return camada;
  }

  #camadaTXT() {
    const camada = this.#elemento('g', { class: 'textos', style: 'pointer-events:none' });
    if (!this.grafo) return camada;
    for (const no of this.grafo.nodes) {
      const estilo = CORES[this.#estadoDoNo(no.id)];
      const rotulo = rotuloCurto(no.label);
      const texto = this.#elemento('text', {
        x: no.x,
        y: no.y + 4,
        'text-anchor': 'middle',
        fill: estilo.texto,
        'font-size': rotulo.length > 6 ? 10 : 12.5,
        'font-family': 'inherit',
        class: 'nome',
      }, rotulo);
      if (rotulo !== no.label) {
        texto.append(this.#elemento('title', {}, no.label));
      }
      camada.append(texto);
    }
    return camada;
  }

  #elemento(tag, atributos = {}, texto = null) {
    const el = document.createElementNS(SVG_NS, tag);
    for (const [chave, valor] of Object.entries(atributos)) {
      if (valor !== null && valor !== undefined) el.setAttribute(chave, String(valor));
    }
    if (texto !== null) el.textContent = texto;
    return el;
  }

  #ligarEventos() {
    let arrastando = null;
    let movido = false;
    let inicioPanorama = null;

    const paraTela = (evento) => {
      const caixa = this.svg.getBoundingClientRect();
      return {
        x: this.panorama.x + ((evento.clientX - caixa.left) / caixa.width) * caixa.width / this.escala,
        y: this.panorama.y + ((evento.clientY - caixa.top) / caixa.height) * caixa.height / this.escala,
      };
    };

    this.svg.addEventListener('pointerdown', (evento) => {
      if (evento.button !== 0) return;
      const no = evento.target.closest('g.no');
      movido = false;
      if (no) {
        arrastando = no.dataset.no;
        this.svg.setPointerCapture(evento.pointerId);
        this.selecao = arrastando;
        this.callbacks.aoSelecionarNo?.(arrastando);
      } else {
        inicioPanorama = { x: evento.clientX, y: evento.clientY, origem: { ...this.panorama } };
        this.svg.classList.add('arrastando');
      }
    });

    this.svg.addEventListener('pointermove', (evento) => {
      if (arrastando) {
        movido = true;
        const ponto = paraTela(evento);
        const no = this.grafo?.no(arrastando);
        if (no) {
          no.x = ponto.x;
          no.y = ponto.y;
          this.desenhar(this.grafo, this.estado);
        }
        return;
      }
      if (inicioPanorama) {
        const caixa = this.svg.getBoundingClientRect();
        this.panorama.x = inicioPanorama.origem.x - ((evento.clientX - inicioPanorama.x) / caixa.width) * (caixa.width / this.escala);
        this.panorama.y = inicioPanorama.origem.y - ((evento.clientY - inicioPanorama.y) / caixa.height) * (caixa.height / this.escala);
        this.svg.setAttribute('viewBox', this.#viewBox(caixa.width, caixa.height));
      }
    });

    const soltar = (evento) => {
      if (arrastando) {
        if (movido) this.callbacks.aoMoverNo?.(arrastando);
        else this.callbacks.aoClicNo?.(arrastando);
      } else if (inicioPanorama && !movido) {
        if (!this.callbacks.aoClicVazio?.(paraTela(evento), evento)) this.selecao = null;
      }
      arrastando = null;
      inicioPanorama = null;
      this.svg.classList.remove('arrastando');
    };

    this.svg.addEventListener('pointerup', soltar);
    this.svg.addEventListener('pointercancel', soltar);

    this.svg.addEventListener('dblclick', (evento) => {
      const no = evento.target.closest('g.no');
      if (no) {
        this.callbacks.aoDuploNo?.(no.dataset.no);
        return;
      }
      const aresta = evento.target.closest('path.aresta');
      if (aresta) {
        this.callbacks.aoDuploAresta?.(aresta.dataset.from, aresta.dataset.to);
      }
    });

    this.svg.addEventListener(
      'wheel',
      (evento) => {
        evento.preventDefault();
        const fator = evento.deltaY < 0 ? 1.12 : 1 / 1.12;
        this.escala = Math.min(3.5, Math.max(0.35, this.escala * fator));
        this.desenhar(this.grafo, this.estado);
      },
      { passive: false },
    );

    this.svg.addEventListener('contextmenu', (evento) => {
      const no = evento.target.closest('g.no');
      if (no) {
        evento.preventDefault();
        this.callbacks.aoRemoverNo?.(no.dataset.no);
      }
    });
  }
}

export function criarMarcadores(svg) {
  const defs = document.createElementNS(SVG_NS, 'defs');
  const marcador = document.createElementNS(SVG_NS, 'marker');
  marcador.setAttribute('id', 'seta');
  marcador.setAttribute('viewBox', '0 0 10 10');
  marcador.setAttribute('refX', '8');
  marcador.setAttribute('refY', '5');
  marcador.setAttribute('markerWidth', '7');
  marcador.setAttribute('markerHeight', '7');
  marcador.setAttribute('orient', 'auto-start-reverse');
  const caminho = document.createElementNS(SVG_NS, 'path');
  caminho.setAttribute('d', 'M 0 1 L 9 5 L 0 9 z');
  caminho.setAttribute('fill', '#8b9bb4');
  marcador.append(caminho);
  defs.append(marcador);
  svg.prepend(defs);
}
