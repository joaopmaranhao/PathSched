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
    this.modoMapa = false;
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
    const defs = this.svg.querySelector('defs');
    this.svg.replaceChildren(...(defs ? [defs] : []));
    if (!grafo) return;
    if (this.estado.mapa || this.modoMapa) this.svg.append(this.#camadaMapa());
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

  novoPreset(grafo, estado = {}) {
    this.escala = 1;
    this.panorama = { x: 0, y: 0 };
    this.desenhar(grafo, estado);
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

  #camadaMapa() {
    const { largura, altura } = this.tamanho();
    const camada = this.#elemento('g', { class: 'mapa-base', transform: `scale(${largura / 1000} ${altura / 700})` });
    camada.append(this.#elemento('rect', { x: 0, y: 0, width: 1000, height: 700, fill: '#0b0a11' }));

    // Áreas abertas sem ruas: parque, mata urbana e uma faixa d'água.
    camada.append(
      this.#elemento('path', { d: 'M 678 28 Q 810 4 990 45 L 1000 255 Q 925 286 830 248 Q 750 225 679 258 Z', fill: '#111c1c', stroke: '#263633', 'stroke-width': 2 }),
      this.#elemento('path', { d: 'M 0 494 Q 96 453 188 491 Q 244 526 224 700 L 0 700 Z', fill: '#101b22', stroke: '#293744', 'stroke-width': 2 }),
      this.#elemento('path', { d: 'M 705 450 Q 822 403 1000 443 L 1000 700 L 767 700 Q 724 622 705 450 Z', fill: '#14191c', stroke: '#292e31', 'stroke-width': 2 }),
    );

    // Quarteirões e lotes; as arestas do grafo são a única rede de ruas.
    const colunas = [28, 139, 250, 361, 472, 583, 694, 805, 916];
    const linhas = [28, 124, 220, 316, 412, 508, 604];
    for (let row = 0; row < linhas.length - 1; row += 1) {
      for (let col = 0; col < colunas.length - 1; col += 1) {
        const x = colunas[col]; const y = linhas[row];
        const cx = x + 53; const cy = y + 46;
        const parque = cx > 690 && cy < 270;
        const agua = cx < 225 && cy > 490;
        const areaAberta = cx > 720 && cy > 455;
        if (parque || agua || areaAberta) continue;
        const corte = ((row * 3 + col * 5) % 4) * 5;
        const d = `M ${x + 8} ${y + 10} L ${x + 99 - corte} ${y + 4} L ${x + 103} ${y + 77 - corte} L ${x + 62} ${y + 87} L ${x + 5} ${y + 77} Z`;
        camada.append(this.#elemento('path', { d, fill: (row + col) % 3 === 0 ? '#171521' : '#14131d', stroke: '#2b2738', 'stroke-width': 1.5 }));
        // Divisões internas sugerem lotes e edifícios dentro do quarteirão.
        for (let lot = 1; lot <= 2; lot += 1) {
          const lx = x + 12 + lot * 27;
          camada.append(this.#elemento('path', { d: `M ${lx} ${y + 17} L ${lx + 2} ${y + 71}`, fill: 'none', stroke: '#252232', 'stroke-width': 1 }));
        }
      }
    }

    camada.append(
      this.#elemento('text', { x: 808, y: 112, class: 'mapa-rotulo-area' }, 'PARQUE'),
      this.#elemento('text', { x: 84, y: 641, class: 'mapa-rotulo-area' }, 'RESERVA'),
      this.#elemento('text', { x: 834, y: 624, class: 'mapa-rotulo-area' }, 'ÁREA INDUSTRIAL'),
    );
    return camada;
  }

  #noCaminho(de, para) {
    const c = this.estado.caminho ?? [];
    for (let i = 1; i < c.length; i += 1) {
      if (c[i - 1] === de && c[i] === para) return true;
      if (!this.grafo.directed && c[i - 1] === para && c[i] === de) return true;
    }
    return false;
  }

  #noSolucao(de, para) {
    const c = this.estado.solucao ?? [];
    for (let i = 1; i < c.length; i += 1) {
      if (c[i - 1] === de && c[i] === para) return true;
      if (!this.grafo.directed && c[i - 1] === para && c[i] === de) return true;
    }
    return false;
  }

  #camadaArestas() {
    const camada = this.#elemento('g', { class: 'arestas' });
    if (!this.grafo) return camada;
    const mapa = this.estado.mapa ?? this.modoMapa;
    const camadaRuas = this.#elemento('g', { class: 'ruas-do-grafo' });

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
        const chaveTrecho = this.grafo.directed ? `${aresta.from}>${aresta.to}` : [aresta.from, aresta.to].sort().join('|');
        const explorada = mapa && this.estado.caminhosExplorados?.has(chaveTrecho);
        const rotaFinal = mapa && this.estado.finalizado && naSolucao;
        const destaque = mapa ? explorada || rotaFinal : naSolucao || naCaminho;
        const seta = this.grafo.directed ? 'url(#seta)' : '';
        if (mapa) {
          const mostrarRua = !this.estado.finalizado || rotaFinal;
          const opacidadeRua = mostrarRua ? 1 : 0;
          camadaRuas.append(
            this.#elemento('path', { d, fill: 'none', stroke: '#07070c', 'stroke-width': 23, 'stroke-linecap': 'round', 'stroke-linejoin': 'round', opacity: opacidadeRua, class: 'mapa-aresta borda-rua' }),
            this.#elemento('path', { d, fill: 'none', stroke: '#514b5c', 'stroke-width': 19, 'stroke-linecap': 'round', 'stroke-linejoin': 'round', opacity: opacidadeRua, class: 'mapa-aresta margem-rua' }),
            this.#elemento('path', { d, fill: 'none', stroke: '#292733', 'stroke-width': 15, 'stroke-linecap': 'round', 'stroke-linejoin': 'round', opacity: opacidadeRua, class: 'mapa-aresta piso-rua' }),
            this.#elemento('path', { d, fill: 'none', stroke: '#625b6b', 'stroke-width': 1, 'stroke-linecap': 'round', 'stroke-dasharray': '5 8', opacity: opacidadeRua * 0.6, class: 'mapa-aresta faixa-rua' }),
          );
        }
        camada.append(
          this.#elemento('path', {
            d,
            fill: 'none',
            stroke: mapa ? (rotaFinal ? '#b8ffec' : explorada ? '#a98cff' : '#303044') : cor,
            'stroke-width': mapa ? (destaque ? (rotaFinal ? 5 : 3.5) : 1.5) : (destaque ? 4 : 2),
            'stroke-dasharray': !mapa && naCaminho && !naSolucao ? '7 5' : 'none',
            filter: mapa && destaque ? `url(#${rotaFinal ? 'brilho-rota' : 'brilho-exploracao'})` : 'none',
            'marker-end': seta,
            opacity: mapa ? (this.estado.finalizado ? (rotaFinal ? 1 : 0) : (destaque ? 1 : 0.45)) : (destaque ? 1 : 0.75),
            class: `aresta${explorada ? ' explorada' : ''}${rotaFinal ? ' rota-final' : ''}`,
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
            opacity: mapa ? (this.estado.finalizado ? (rotaFinal ? 1 : 0) : 0.72) : 1,
            class: 'aresta-custo-bg',
          }),
        );
        camada.append(
          this.#elemento('text', {
            x: meio.x,
            y: meio.y + 3.5,
            'text-anchor': 'middle',
            fill: destaque ? cor : '#9aa7b8',
            opacity: mapa ? (this.estado.finalizado ? (rotaFinal ? 1 : 0) : 0.82) : 1,
            class: 'aresta-rotulo',
          }, String(aresta.weight)),
        );
      });
    }
    if (mapa) camada.prepend(camadaRuas);
    return camada;
  }

  #geometria(a, b, curva) {
    const raioNo = (this.estado.mapa ?? this.modoMapa) ? 9 : RAIO;
    const ax = a.x;
    const ay = a.y;
    const bx = b.x;
    const by = b.y;
    const dx = bx - ax;
    const dy = by - ay;
    const dist = Math.hypot(dx, dy) || 1;
    const ux = dx / dist;
    const uy = dy / dist;
    const x1 = ax + ux * (raioNo + 3);
    const y1 = ay + uy * (raioNo + 3);
    const recuoFinal = raioNo === 9 ? raioNo + 3 : raioNo + 9;
    const x2 = bx - ux * recuoFinal;
    const y2 = by - uy * recuoFinal;
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
    const mapa = this.estado.mapa ?? this.modoMapa;
    for (const no of this.grafo.nodes) {
      const estilo = CORES[this.#estadoDoNo(no.id)];
      const grupo = this.#elemento('g', {
        class: no.id === this.estado.atual ? 'no novo' : 'no',
        'data-no': no.id,
        transform: `translate(${no.x} ${no.y})`,
      });
      grupo.append(
        this.#elemento('circle', {
          r: mapa ? 9 : RAIO,
          fill: mapa ? (no.id === this.estado.inicio ? '#291c46' : no.id === this.estado.objetivo ? '#3b2917' : '#151521') : estilo.preenchimento,
          stroke: mapa ? (no.id === this.estado.inicio ? '#a98cff' : no.id === this.estado.objetivo ? '#ffc477' : '#56516b') : estilo.contorno,
          'stroke-width': mapa ? (no.id === this.estado.inicio || no.id === this.estado.objetivo ? 2.5 : 1.2) : (no.id === this.estado.atual ? 3 : 2),
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
    const mapa = this.estado.mapa ?? this.modoMapa;
    for (const no of this.grafo.nodes) {
      const estilo = CORES[this.#estadoDoNo(no.id)];
      const rotulo = rotuloCurto(no.label);
      const texto = this.#elemento('text', {
        x: no.x,
        y: no.y + (mapa ? 3 : 4),
        'text-anchor': 'middle',
        fill: mapa ? '#eee8ff' : estilo.texto,
        'font-size': mapa ? 9 : (rotulo.length > 6 ? 10 : 12.5),
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
    let inicioPointer = null;
    let alvoDblClique = null;

    const paraTela = (evento) => {
      const matriz = this.svg.getScreenCTM();
      if (!matriz) return { x: 0, y: 0 };
      const ponto = this.svg.createSVGPoint();
      ponto.x = evento.clientX;
      ponto.y = evento.clientY;
      const local = ponto.matrixTransform(matriz.inverse());
      return { x: local.x, y: local.y };
    };

    this.svg.addEventListener('pointerdown', (evento) => {
      if (evento.button !== 0) return;
      alvoDblClique = evento.target;
      const no = evento.target.closest('g.no');
      movido = false;
      inicioPointer = { x: evento.clientX, y: evento.clientY };
      this.svg.setPointerCapture(evento.pointerId);
      if (no) {
        arrastando = no.dataset.no;
        this.selecao = arrastando;
        this.callbacks.aoSelecionarNo?.(arrastando);
      } else {
        inicioPanorama = {
          x: evento.clientX,
          y: evento.clientY,
          origem: { ...this.panorama },
          clicouAresta: Boolean(evento.target.closest('path.aresta')),
        };
        this.svg.classList.add('arrastando');
      }
    });

    this.svg.addEventListener('pointermove', (evento) => {
      if (arrastando) {
        movido ||= Math.hypot(evento.clientX - inicioPointer.x, evento.clientY - inicioPointer.y) > 3;
        if (!movido) return;
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
        movido ||= Math.hypot(evento.clientX - inicioPanorama.x, evento.clientY - inicioPanorama.y) > 3;
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
        if (!inicioPanorama.clicouAresta && !this.callbacks.aoClicVazio?.(paraTela(evento), evento)) this.selecao = null;
      }
      arrastando = null;
      inicioPanorama = null;
      inicioPointer = null;
      this.svg.classList.remove('arrastando');
      if (this.svg.hasPointerCapture(evento.pointerId)) this.svg.releasePointerCapture(evento.pointerId);
    };

    this.svg.addEventListener('pointerup', soltar);
    this.svg.addEventListener('pointercancel', soltar);

    this.svg.addEventListener('dblclick', (evento) => {
      const alvo = evento.target.closest('g.no, path.aresta') ?? alvoDblClique?.closest('g.no, path.aresta');
      alvoDblClique = null;
      if (alvo?.matches('g.no')) {
        this.callbacks.aoDuploNo?.(alvo.dataset.no);
        return;
      }
      if (alvo?.matches('path.aresta')) {
        this.callbacks.aoDuploAresta?.(alvo.dataset.from, alvo.dataset.to);
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
  for (const [id, cor, desfoque] of [['brilho-exploracao', '#8d63ff', 5], ['brilho-rota', '#64ffd0', 6]]) {
    const filtro = document.createElementNS(SVG_NS, 'filter');
    filtro.setAttribute('id', id); filtro.setAttribute('x', '-80%'); filtro.setAttribute('y', '-80%');
    filtro.setAttribute('width', '260%'); filtro.setAttribute('height', '260%');
    const blur = document.createElementNS(SVG_NS, 'feGaussianBlur');
    blur.setAttribute('stdDeviation', String(desfoque)); blur.setAttribute('result', 'blur');
    const flood = document.createElementNS(SVG_NS, 'feFlood');
    flood.setAttribute('flood-color', cor); flood.setAttribute('flood-opacity', '0.9'); flood.setAttribute('result', 'color');
    const comp = document.createElementNS(SVG_NS, 'feComposite');
    comp.setAttribute('in', 'color'); comp.setAttribute('in2', 'blur'); comp.setAttribute('operator', 'in'); comp.setAttribute('result', 'glow');
    const merge = document.createElementNS(SVG_NS, 'feMerge');
    for (const entrada of ['glow', 'SourceGraphic']) { const no = document.createElementNS(SVG_NS, 'feMergeNode'); no.setAttribute('in', entrada); merge.append(no); }
    filtro.append(blur, flood, comp, merge); defs.append(filtro);
  }
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
