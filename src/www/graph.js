const A = 'A'.charCodeAt(0);
const Z = 'Z'.charCodeAt(0);

export class Grafo {
  constructor(dados = {}) {
    this.directed = Boolean(dados.directed);
    this.nodes = [];
    this.edges = [];
    if (Array.isArray(dados.nodes)) this.nodes = dados.nodes.map((n) => this.#normalizarNo(n));
    if (Array.isArray(dados.edges)) {
      this.edges = dados.edges.map((e) => ({
        from: String(e.from),
        to: String(e.to),
        weight: Number(e.weight),
      }));
    }
  }

  #normalizarNo(bruto) {
    if (typeof bruto === 'string' || typeof bruto === 'number') {
      return { id: String(bruto), label: String(bruto), x: null, y: null };
    }
    return {
      id: String(bruto.id),
      label: bruto.label === undefined || bruto.label === null ? String(bruto.id) : String(bruto.label),
      x: Number.isFinite(bruto.x) ? Number(bruto.x) : null,
      y: Number.isFinite(bruto.y) ? Number(bruto.y) : null,
    };
  }

  clonar() {
    return new Grafo(this.paraJSON());
  }

  paraJSON() {
    return {
      directed: this.directed,
      nodes: this.nodes.map((n) => ({ id: n.id, label: n.label, x: n.x, y: n.y })),
      edges: this.edges.map((e) => ({ from: e.from, to: e.to, weight: e.weight })),
    };
  }

  paraTexto() {
    return JSON.stringify(this.paraJSON(), null, 2);
  }

  static doTexto(texto) {
    let dados;
    try {
      dados = JSON.parse(texto);
    } catch (erro) {
      throw new Error(`JSON inválido: ${erro.message}`);
    }
    const grafo = new Grafo(dados);
    grafo.validar();
    return grafo;
  }

  validar() {
    if (this.nodes.length === 0) throw new Error('o grafo precisa ter pelo menos um nó');
    const vistos = new Set();
    for (const no of this.nodes) {
      if (!no.id.trim()) throw new Error('existe um nó sem identificador');
      if (vistos.has(no.id)) throw new Error(`nó duplicado: ${no.id}`);
      vistos.add(no.id);
    }
    this.edges.forEach((aresta, i) => {
      if (!vistos.has(aresta.from)) {
        throw new Error(`aresta #${i} sai do nó inexistente "${aresta.from}"`);
      }
      if (!vistos.has(aresta.to)) {
        throw new Error(`aresta #${i} chega em um nó inexistente: "${aresta.to}"`);
      }
      if (!Number.isFinite(aresta.weight) || aresta.weight < 0) {
        throw new Error(`a aresta ${aresta.from} -> ${aresta.to} precisa de um custo finito e >= 0`);
      }
    });
    return this;
  }

  ids() {
    return this.nodes.map((n) => n.id);
  }

  no(id) {
    return this.nodes.find((n) => n.id === id) ?? null;
  }

  rotulo(id) {
    return this.no(id)?.label ?? id;
  }

  existe(id) {
    return this.no(id) !== null;
  }

  novoId(base = 'N') {
    let sufixo = 1;
    let id = `${base}${sufixo}`;
    while (this.existe(id)) {
      sufixo += 1;
      id = `${base}${sufixo}`;
    }
    return id;
  }

  sufixoDeRotulo(base = 'N') {
    let i = 0;
    for (const no of this.nodes) {
      const ultimo = no.label.at(-1);
      if (no.label.length === base.length + 1 && no.label.startsWith(base) && ultimo >= '0' && ultimo <= '9') {
        i = Math.max(i, Number(ultimo));
      }
    }
    return `${base}${i + 1}`;
  }

  adicionarNo(id, x = null, y = null, label = null) {
    if (this.existe(id)) return null;
    const no = { id, label: label ?? id, x, y };
    this.nodes.push(no);
    return no;
  }

  removerNo(id) {
    this.nodes = this.nodes.filter((n) => n.id !== id);
    this.edges = this.edges.filter((e) => e.from !== id && e.to !== id);
  }

  renomear(id, novoRotulo) {
    const no = this.no(id);
    if (no) no.label = novoRotulo;
  }

  arestasDe(id) {
    return this.edges.filter((e) => e.from === id);
  }

  aresta(de, para) {
    return this.edges.find((e) => e.from === de && e.to === para) ?? null;
  }

  adicionarAresta(de, para, peso = 1) {
    if (de === para) throw new Error('não dá para criar uma aresta de um nó para ele mesmo');
    if (!this.existe(de) || !this.existe(para)) throw new Error('aresta com nó inexistente');
    const custo = Number(peso);
    if (!Number.isFinite(custo) || custo < 0) throw new Error('o custo precisa ser um número >= 0');

    const direta = this.aresta(de, para);
    if (direta) {
      direta.weight = custo;
      return direta;
    }
    const reversa = this.aresta(para, de);
    if (!this.directed && reversa) {
      reversa.weight = custo;
      return reversa;
    }
    const aresta = { from: de, to: para, weight: custo };
    this.edges.push(aresta);
    return aresta;
  }

  removerAresta(de, para) {
    this.edges = this.edges.filter((e) => !(e.from === de && e.to === para));
  }

  custoEntre(de, para) {
    if (!this.directed) {
      return this.aresta(de, para)?.weight ?? this.aresta(para, de)?.weight ?? null;
    }
    return this.aresta(de, para)?.weight ?? null;
  }

  custoDoCaminho(caminho) {
    let total = 0;
    for (let i = 1; i < caminho.length; i += 1) {
      const custo = this.custoEntre(caminho[i - 1], caminho[i]);
      if (custo === null) return Infinity;
      total += custo;
    }
    return total;
  }

  limparPosicoes() {
    for (const no of this.nodes) {
      no.x = null;
      no.y = null;
    }
  }

  posicionado() {
    return this.nodes.every((n) => Number.isFinite(n.x) && Number.isFinite(n.y));
  }

  *pares() {
    for (const aresta of this.edges) {
      yield [aresta.from, aresta.to];
    }
  }

  aplicarLayout(tipo, largura, altura) {
    const n = this.nodes.length;
    if (n === 0) return;
    const margem = 70;
    const cx = largura / 2;
    const cy = altura / 2;
    const raio = Math.max(80, Math.min(largura, altura) / 2 - margem);

    if (tipo === 'grade') {
      const colunas = Math.max(1, Math.ceil(Math.sqrt(n)));
      const linhas = Math.ceil(n / colunas);
      const larguraUtil = Math.max(0, largura - 2 * margem);
      const alturaUtil = Math.max(0, altura - 2 * margem);
      this.nodes.forEach((no, i) => {
        const coluna = i % colunas;
        const linha = Math.floor(i / colunas);
        no.x = margem + (colunas > 1 ? (coluna * larguraUtil) / (colunas - 1) : larguraUtil / 2);
        no.y = margem + (linhas > 1 ? (linha * alturaUtil) / (linhas - 1) : alturaUtil / 2);
      });
      return;
    }

    this.nodes.forEach((no, i) => {
      const angulo = (2 * Math.PI * i) / n - Math.PI / 2;
      no.x = cx + raio * Math.cos(angulo);
      no.y = cy + raio * Math.sin(angulo);
    });
  }
}

export function grafoDoPreset(preset, largura, altura) {
  const grafo = new Grafo(preset.graph);
  if (!grafo.posicionado()) grafo.aplicarLayout('circular', largura, altura);
  return grafo;
}

export function rotuloCurto(texto, maximo = 9) {
  return texto.length > maximo ? `${texto.slice(0, maximo - 1)}…` : texto;
}

export function letraInicial(base) {
  const codigo = base.charCodeAt(0);
  if (codigo >= A && codigo <= Z) return String.fromCharCode(codigo);
  return base;
}
