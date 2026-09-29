import { Grafo, grafoDoPreset } from './graph.js';
import { PRESETS } from './presets.js';
import { Renderizador, criarMarcadores } from './render.js';
import { Player } from './player.js';

const ALGORITMOS = {
  ucs: {
    nome: 'UCS',
    descricao: 'Busca de custo uniforme: expande sempre o nó aberto de menor g. Devolve o caminho de menor custo, mas pode expandir muitos nós.',
  },
  ids: {
    nome: 'IDS',
    descricao: 'Busca iterativa em profundidade: repete a DFS com limites 0, 1, 2… até achar o objetivo. Não é completa sem limite de profundidade e repete trabalho.',
  },
  dls: {
    nome: 'DLS',
    descricao: 'Busca em profundidade limitada: igual à DFS, porém cada repetição para no limite de profundidade escolhido.',
  },
  bfs: {
    nome: 'BFS',
    descricao: 'Busca em largura: não olha custos, garante o menor número de saltos. Pode achar um caminho mais caro que o do UCS.',
  },
  dfs: {
    nome: 'DFS',
    descricao: 'Busca em profundidade: desce pelo primeiro vizinho e volta quando trava. Rápida, sem garantia de completude nem de menor custo.',
  },
};

const FASE_ROTULO = {
  init: 'início',
  push: 'empilhar',
  pop: 'desempilhar',
  expand: 'expandir',
  visit: 'visitar',
  goal: 'objetivo',
  backtrack: 'voltar',
  cut: 'podar',
  fail: 'falha',
};

const el = (id) => document.getElementById(id);

const estado = {
  grafo: null,
  resultado: null,
  podados: new Set(),
  iteracaoAtual: 0,
  origemAresta: null,
  inicio: '',
  objetivo: '',
};

const ui = {
  preset: el('preset'),
  descricaoPreset: el('descricao-preset'),
  descricaoAlgoritmo: el('descricao-algoritmo'),
  algoritmo: el('algoritmo'),
  inicio: el('inicio'),
  objetivo: el('objetivo'),
  maxIteracoes: el('max-iteracoes'),
  limite: el('limite'),
  campoIteracoes: el('campo-iteracoes'),
  campoLimite: el('campo-limite'),
  executar: el('btn-executar'),
  erro: el('erro-busca'),
  modoEdicao: el('modo-edicao'),
  dirigido: el('grafo-dirigido'),
  novoNo: el('novo-no'),
  novaAresta: el('nova-aresta'),
  json: el('json-grafo'),
  aplicarJson: el('btn-aplicar-json'),
  copiarJson: el('btn-copiar-json'),
  svg: el('palco-svg'),
  contagemNos: el('contagem-nos'),
  tocar: el('btn-tocar'),
  velocidade: el('velocidade'),
  valorVelocidade: el('valor-velocidade'),
  passoAtual: el('passo-atual'),
  passoTotal: el('passo-total'),
  progresso: el('progresso'),
  fase: el('fase'),
  custo: el('custo-atual'),
  mensagem: el('mensagem'),
  profundidade: el('m-profundidade'),
  iteracao: el('m-iteracao'),
  limitePasso: el('m-limite'),
  chipsCaminho: el('chips-caminho'),
  tabela: el('tabela-fronteira').querySelector('tbody'),
  notaFronteira: el('nota-fronteira'),
  tamanhoFronteira: el('tamanho-fronteira'),
  chipsVisitados: el('chips-visitados'),
  totalVisitados: el('total-visitados'),
  chipsPilha: el('chips-pilha'),
  totalPilha: el('total-pilha'),
  log: el('log'),
  iteracoes: el('iteracoes'),
  iteracoesBarra: el('iteracoes-barra'),
  resSituacao: el('res-situacao'),
  resCaminho: el('res-caminho'),
  resCusto: el('res-custo'),
  resExpandidos: el('res-expandidos'),
  resPassos: el('res-passos'),
  badge: el('badge-backend'),
  url: el('url-backend'),
};

const renderizador = new Renderizador(ui.svg, {
  aoSelecionarNo: () => {},
  aoClicNo: aoClicNo,
  aoClicVazio: aoClicVazio,
  aoMoverNo: sincronizarJson,
  aoDuploNo: aoDuploNo,
  aoDuploAresta: aoDuploAresta,
  aoRemoverNo: aoRemoverNo,
});

const player = new Player({
  aoMudar: pintarPasso,
  aoFim: () => {
    if (!estado.resultado) return;
    const r = estado.resultado;
    ui.mensagem.textContent = r.found
      ? `Busca terminada: ${r.path.join(' → ')} com custo ${formatar(r.total_cost)}.`
      : 'Busca terminada sem solução: o objetivo não é alcançável ou o limite foi insuficiente.';
  },
});

criarMarcadores(ui.svg);
preencherPresets();
ligarControles();
verificarBackend();
carregarPreset(PRESETS[0].id, { executarBusca: true });

// ---------------------------------------------------------------- backend

async function verificarBackend() {
  ui.url.textContent = window.location.origin;
  try {
    const resposta = await fetch('/api/health');
    const dados = await resposta.json();
    ui.badge.textContent = dados.ok ? 'backend conectado' : 'backend indisponível';
    ui.badge.className = `badge ${dados.ok ? 'badge-online' : 'badge-offline'}`;
  } catch {
    ui.badge.textContent = 'sem conexão com o backend';
    ui.badge.className = 'badge badge-offline';
  }
}

async function executarBusca() {
  const algoritmo = ui.algoritmo.value;
  const corpo = {
    graph: estado.grafo.paraJSON(),
    algorithm: algoritmo,
    start: ui.inicio.value,
    goal: ui.objetivo.value,
    max_iterations: Number(ui.maxIteracoes.value) || 20,
    limit: algoritmo === 'dls' ? Number(ui.limite.value) || 0 : null,
  };

  ui.executar.disabled = true;
  ui.erro.hidden = true;
  try {
    const resposta = await fetch('/api/search', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(corpo),
    });
    const dados = await resposta.json();
    if (!resposta.ok) throw new Error(dados.erro ?? 'falha na busca');
    receberResultado(dados);
  } catch (erro) {
    ui.erro.textContent = erro.message;
    ui.erro.hidden = false;
    player.carregar([]);
    ui.log.innerHTML = '<li class="log-vazio">nenhum passo executado</li>';
    ui.mensagem.textContent = 'Não foi possível executar a busca.';
  } finally {
    ui.executar.disabled = false;
  }
}

function receberResultado(dados) {
  estado.resultado = dados;
  estado.podados = new Set();
  estado.iteracaoAtual = 0;

  ui.resSituacao.textContent = dados.found ? 'solução encontrada' : 'sem solução';
  ui.resSituacao.className = dados.found ? 'ok' : 'ruim';
  ui.resCaminho.textContent = dados.found ? dados.path.join(' → ') : '—';
  ui.resCusto.textContent = dados.found ? `${formatar(dados.total_cost)}` : '—';
  ui.resExpandidos.textContent = String(dados.expanded);
  ui.resPassos.textContent = String(dados.steps);
  ui.passoTotal.textContent = String(dados.trace.length);

  montarLog(dados.trace);
  montarIteracoes(dados);
  player.carregar(dados.trace);
  if (dados.trace.length > 0) player.tocar();
}

function montarIteracoes(dados) {
  ui.iteracoes.hidden = !dados.iterations?.length;
  ui.iteracoesBarra.replaceChildren();
  if (!dados.iterations?.length) return;
  dados.iterations.forEach((iteracao, i) => {
    const bloco = document.createElement('button');
    bloco.type = 'button';
    bloco.className = 'iter-bloco';
    if (iteracao.found) bloco.classList.add('found');
    bloco.textContent = `L${iteracao.limit} · ${iteracao.steps} passos`;
    const primeiro = dados.trace.findIndex((p) => p.iteration === i + 1);
    bloco.addEventListener('click', () => {
      player.pausar();
      if (primeiro >= 0) player.irPara(primeiro);
    });
    ui.iteracoesBarra.append(bloco);
  });
}

function montarLog(trace) {
  ui.log.replaceChildren();
  if (trace.length === 0) {
    const li = document.createElement('li');
    li.className = 'log-vazio';
    li.textContent = 'a busca não produziu passos';
    ui.log.append(li);
    return;
  }
  const fragmento = document.createDocumentFragment();
  let iteracaoAnterior = 0;
  trace.forEach((passo, i) => {
    const li = document.createElement('li');
    li.dataset.i = String(i);
    if (passo.iteration && passo.iteration !== iteracaoAnterior) {
      li.classList.add('iteracao-nova');
      iteracaoAnterior = passo.iteration;
    }
    const idx = document.createElement('span');
    idx.className = 'idx';
    idx.textContent = String(i);
    const fase = document.createElement('span');
    fase.className = `fase-tag ${passo.phase}`;
    fase.textContent = FASE_ROTULO[passo.phase] ?? passo.phase;
    const texto = document.createElement('span');
    texto.textContent = passo.iteration ? `L${passo.limit} · ${passo.message}` : passo.message;
    li.append(idx, fase, texto);
    li.addEventListener('click', () => {
      player.pausar();
      player.irPara(i);
    });
    fragmento.append(li);
  });
  ui.log.append(fragmento);
}

// ---------------------------------------------------------------- pintura

function pintarPasso(indice, passo) {
  ui.passoAtual.textContent = String(indice + 1);
  ui.progresso.style.width = `${player.total ? ((indice + 1) / player.total) * 100 : 0}%`;
  ui.tocar.textContent = player.tocando ? '⏸ Pausar' : '▶ Tocar';
  if (!passo) {
    renderizador.atualizar({});
    return;
  }

  if (passo.phase === 'init') {
    estado.podados = new Set();
    estado.iteracaoAtual = passo.iteration ?? 0;
  }
  if (passo.phase === 'cut' && passo.node && !(passo.visited ?? []).includes(passo.node)) {
    estado.podados.add(passo.node);
  }

  const fronteira = new Set((passo.frontier ?? []).map((i) => i.node));
  renderizador.atualizar({
    inicio: estado.inicio,
    objetivo: estado.objetivo,
    visitados: new Set(passo.visited ?? []),
    fronteira,
    atual: passo.node ?? null,
    caminho: passo.path ?? [],
    falhos: new Set(estado.podados),
    solucao: estado.resultado?.found ? estado.resultado.path : [],
  });

  ui.fase.textContent = FASE_ROTULO[passo.phase] ?? passo.phase;
  ui.fase.className = `fase ${passo.phase}`;
  const custoTexto = passo.phase === 'init' ? 'g = 0' : `g = ${formatar(passo.g)}`;
  ui.custo.textContent = custoTexto;
  ui.mensagem.textContent = passo.message;
  ui.profundidade.textContent = passo.node ? String(passo.depth) : '—';
  ui.iteracao.textContent = passo.iteration ? `${passo.iteration}` : '—';
  ui.limitePasso.textContent = passo.limit === null || passo.limit === undefined ? '—' : String(passo.limit);

  pintarCaminho(passo.path ?? []);
  pintarFronteira(passo.frontier ?? []);
  pintarChips(ui.chipsVisitados, passo.visited ?? [], 'total-visitados', 'nenhum nó fechado ainda');
  pintarChips(ui.chipsPilha, passo.call_stack ?? [], 'total-pilha', 'busca em largura não usa pilha de chamada');
  marcarLog(indice);
  marcarIteracao(passo.iteration);
}

function pintarCaminho(caminho) {
  ui.chipsCaminho.replaceChildren();
  if (caminho.length === 0) {
    ui.chipsCaminho.innerHTML = '<em>vazio</em>';
    return;
  }
  caminho.forEach((id, i) => {
    if (i > 0) {
      const seta = document.createElement('span');
      seta.className = 'chip seta';
      seta.textContent = '→';
      ui.chipsCaminho.append(seta);
    }
    const chip = document.createElement('span');
    chip.className = `chip ${i === caminho.length - 1 ? 'atual' : 'fim'}`;
    chip.textContent = estado.grafo.rotulo(id);
    ui.chipsCaminho.append(chip);
  });
}

function pintarFronteira(fronteira) {
  ui.tamanhoFronteira.textContent = String(fronteira.length);
  ui.tabela.replaceChildren();
  ui.notaFronteira.textContent = estado.resultado?.algorithm === 'bfs' || estado.resultado?.algorithm === 'ucs'
    ? 'Fila em ordem: o primeiro item é o próximo a ser expandido.'
    : '';
  if (fronteira.length === 0) {
    const linha = document.createElement('tr');
    const celula = document.createElement('td');
    celula.colSpan = 5;
    celula.className = 'vazia';
    celula.textContent = 'fronteira vazia — a busca não usa fila (profundidade) ou acabou';
    linha.append(celula);
    ui.tabela.append(linha);
    return;
  }
  const fragmento = document.createDocumentFragment();
  fronteira.forEach((item, i) => {
    const linha = document.createElement('tr');
    if (i === 0) linha.className = 'atual';
    const celulas = [
      String(i),
      estado.grafo.rotulo(item.node),
      formatar(item.g),
      formatar(item.weight),
      item.path.map((id) => estado.grafo.rotulo(id)).join(' → '),
    ];
    celulas.forEach((texto, j) => {
      const celula = document.createElement('td');
      if (j === 4) celula.className = 'dirigida';
      celula.textContent = texto;
      linha.append(celula);
    });
    fragmento.append(linha);
  });
  ui.tabela.append(fragmento);
}

function pintarChips(container, ids, contadorId, vazio) {
  container.replaceChildren();
  if (ids.length === 0) {
    container.innerHTML = `<em>${vazio}</em>`;
    document.getElementById(contadorId).textContent = '0';
    return;
  }
  document.getElementById(contadorId).textContent = String(ids.length);
  ids.forEach((id, i) => {
    if (i > 0) {
      const seta = document.createElement('span');
      seta.className = 'chip seta';
      seta.textContent = '→';
      container.append(seta);
    }
    const chip = document.createElement('span');
    chip.className = `chip ${i === ids.length - 1 ? 'atual' : 'fim'}`;
    chip.textContent = estado.grafo.rotulo(id);
    container.append(chip);
  });
}

function marcarLog(indice) {
  const anterior = ui.log.querySelector('li.atual');
  if (anterior) anterior.classList.remove('atual');
  const alvo = ui.log.querySelector(`li[data-i="${indice}"]`);
  if (alvo) {
    alvo.classList.add('atual');
    alvo.scrollIntoView?.({ block: 'nearest' });
  }
}

function marcarIteracao(iteracao) {
  if (!iteracao) return;
  const blocos = ui.iteracoesBarra.querySelectorAll('.iter-bloco');
  blocos.forEach((bloco, i) => bloco.classList.toggle('atual', i === iteracao - 1));
}

// ---------------------------------------------------------------- grafo

function preencherPresets() {
  for (const preset of PRESETS) {
    const opcao = document.createElement('option');
    opcao.value = preset.id;
    opcao.textContent = preset.nome;
    ui.preset.append(opcao);
  }
}

function carregarPreset(id, { executarBusca: executarDepois = false } = {}) {
  const preset = PRESETS.find((p) => p.id === id) ?? PRESETS[0];
  ui.preset.value = preset.id;
  ui.descricaoPreset.textContent = preset.descricao;
  const { largura, altura } = renderizador.tamanho();
  aplicarGrafo(grafoDoPreset(preset, largura, altura), { inicio: preset.start, objetivo: preset.goal });
  if (executarDepois) executarBusca();
}

function aplicarGrafo(grafo, { inicio, objetivo } = {}) {
  const ids = grafo.ids();
  estado.grafo = grafo;
  estado.inicio = [inicio, ui.inicio.value].find((v) => v && grafo.existe(v)) ?? ids[0];
  estado.objetivo = [objetivo, ui.objetivo.value].find((v) => v && grafo.existe(v)) ?? ids.at(-1) ?? estado.inicio;
  estado.resultado = null;
  estado.podados = new Set();
  estado.origemAresta = null;

  ui.dirigido.checked = grafo.directed;
  ui.contagemNos.textContent = String(grafo.nodes.length);
  preencherSelectNos();
  limparResumo();
  ui.log.innerHTML = '<li class="log-vazio">nenhum passo ainda</li>';
  ui.iteracoes.hidden = true;
  ui.passoTotal.textContent = '0';
  ui.passoAtual.textContent = '0';
  ui.progresso.style.width = '0%';
  player.carregar([]);
  renderizador.novoPreset(grafo);
  sincronizarJson();
  ui.mensagem.textContent = 'Grafo carregado. Ajuste os parâmetros e execute a busca.';
  ui.fase.textContent = '—';
  ui.custo.textContent = 'g = —';
  ui.profundidade.textContent = '—';
  ui.iteracao.textContent = '—';
  ui.limitePasso.textContent = '—';
  ui.tamanhoFronteira.textContent = '0';
  ui.totalVisitados.textContent = '0';
  ui.totalPilha.textContent = '0';
  ui.tabela.replaceChildren();
  ui.chipsCaminho.innerHTML = '<em>vazio</em>';
  ui.chipsVisitados.innerHTML = '<em>—</em>';
  ui.chipsPilha.innerHTML = '<em>—</em>';
}

function preencherSelectNos() {
  for (const select of [ui.inicio, ui.objetivo]) {
    const anterior = select.value;
    select.replaceChildren();
    for (const id of estado.grafo.ids()) {
      const opcao = document.createElement('option');
      opcao.value = id;
      opcao.textContent = estado.grafo.rotulo(id);
      select.append(opcao);
    }
    if (estado.grafo.existe(anterior)) select.value = anterior;
  }
  ui.inicio.value = estado.inicio;
  ui.objetivo.value = estado.objetivo;
}

function limparResumo() {
  ui.resSituacao.textContent = '—';
  ui.resSituacao.className = '';
  ui.resCaminho.textContent = '—';
  ui.resCusto.textContent = '—';
  ui.resExpandidos.textContent = '—';
  ui.resPassos.textContent = '—';
}

function sincronizarJson() {
  ui.json.value = estado.grafo.paraTexto();
}

function aoClicVazio(ponto) {
  if (!ui.modoEdicao.checked) {
    estado.origemAresta = null;
    renderizador.selecao = null;
    renderizador.desenhar(estado.grafo, {});
    return false;
  }
  const rotulo = (ui.novoNo.value || '').trim() || estado.grafo.sufixoDeRotulo('N');
  const id = estado.grafo.existe(rotulo) ? estado.grafo.novoId('N') : rotulo;
  estado.grafo.adicionarNo(id, ponto.x, ponto.y, rotulo);
  ui.novoNo.value = '';
  aposEditar();
  return true;
}

function aoClicNo(id) {
  if (!ui.modoEdicao.checked) {
    estado.inicio = id;
    ui.inicio.value = id;
    return;
  }
  if (!estado.origemAresta) {
    estado.origemAresta = id;
    renderizador.selecao = id;
    renderizador.desenhar(estado.grafo, {});
    return;
  }
  const de = estado.origemAresta;
  estado.origemAresta = null;
  renderizador.selecao = null;
  if (de === id) {
    renderizador.desenhar(estado.grafo, {});
    return;
  }
  try {
    estado.grafo.adicionarAresta(de, id, Number(ui.novaAresta.value) || 1);
    aposEditar();
  } catch (erro) {
    ui.erro.textContent = erro.message;
    ui.erro.hidden = false;
    renderizador.desenhar(estado.grafo, {});
  }
}

async function aoDuploNo(id) {
  if (!ui.modoEdicao.checked) return;
  const atual = estado.grafo.rotulo(id);
  const novo = await perguntar(`Novo rótulo para ${atual}`, atual);
  if (novo === null || !novo.trim() || novo === atual) return;
  estado.grafo.renomear(id, novo.trim());
  aposEditar();
}

async function aoDuploAresta(de, para) {
  if (!ui.modoEdicao.checked) return;
  const aresta = estado.grafo.aresta(de, para) ?? estado.grafo.aresta(para, de);
  if (!aresta) return;
  const novo = await perguntar(`Custo de ${estado.grafo.rotulo(aresta.from)} → ${estado.grafo.rotulo(aresta.to)}`, String(aresta.weight));
  if (novo === null) return;
  const custo = Number(novo);
  if (!Number.isFinite(custo) || custo < 0) {
    ui.erro.textContent = 'o custo precisa ser um número >= 0';
    ui.erro.hidden = false;
    return;
  }
  aresta.weight = custo;
  aposEditar();
}

async function aoRemoverNo(id) {
  if (!ui.modoEdicao.checked) return;
  const ok = await perguntar(`Remover o nó ${estado.grafo.rotulo(id)} e suas arestas?`, 'remover');
  if (ok === null) return;
  estado.grafo.removerNo(id);
  if (estado.inicio === id) estado.inicio = estado.grafo.ids()[0];
  if (estado.objetivo === id) estado.objetivo = estado.grafo.ids().at(-1) ?? '';
  aposEditar();
}

function aposEditar() {
  ui.erro.hidden = true;
  ui.contagemNos.textContent = String(estado.grafo.nodes.length);
  preencherSelectNos();
  estado.resultado = null;
  limparResumo();
  renderizador.desenhar(estado.grafo, {});
  sincronizarJson();
  ui.mensagem.textContent = 'Grafo editado. Execute a busca de novo para ver o novo percurso.';
}

// ---------------------------------------------------------------- modal

function perguntar(titulo, valorInicial = '') {
  return new Promise((resolve) => {
    const fundo = el('modal');
    const entrada = el('modal-entrada');
    el('modal-titulo').textContent = titulo;
    entrada.value = valorInicial;
    fundo.hidden = false;
    entrada.focus();
    entrada.select();

    const fechar = (resultado) => {
      fundo.hidden = true;
      form.removeEventListener('submit', aoEnviar);
      el('modal-cancelar').removeEventListener('click', aoCancelar);
      entrada.removeEventListener('keydown', aoTecla);
      resolve(resultado);
    };
    const aoEnviar = (evento) => {
      evento.preventDefault();
      fechar(entrada.value);
    };
    const aoCancelar = () => fechar(null);
    const aoTecla = (evento) => {
      if (evento.key === 'Escape') fechar(null);
    };
    const form = el('modal-form');
    form.addEventListener('submit', aoEnviar);
    el('modal-cancelar').addEventListener('click', aoCancelar);
    entrada.addEventListener('keydown', aoTecla);
  });
}

// ---------------------------------------------------------------- controles

function ligarControles() {
  ui.preset.addEventListener('change', () => carregarPreset(ui.preset.value));
  ui.algoritmo.addEventListener('change', atualizarDescricaoAlgoritmo);
  ui.executar.addEventListener('click', executarBusca);
  ui.inicio.addEventListener('change', () => {
    estado.inicio = ui.inicio.value;
  });
  ui.objetivo.addEventListener('change', () => {
    estado.objetivo = ui.objetivo.value;
  });
  ui.modoEdicao.addEventListener('change', () => {
    ui.svg.classList.toggle('editando', ui.modoEdicao.checked);
    el('ajuda-edicao').style.opacity = ui.modoEdicao.checked ? '1' : '0.6';
  });
  ui.dirigido.addEventListener('change', () => {
    estado.grafo.directed = ui.dirigido.checked;
    aposEditar();
  });

  el('btn-layout-circular').addEventListener('click', () => {
    const { largura, altura } = renderizador.tamanho();
    estado.grafo.aplicarLayout('circular', largura, altura);
    renderizador.desenhar(estado.grafo, {});
    sincronizarJson();
  });
  el('btn-layout-grade').addEventListener('click', () => {
    const { largura, altura } = renderizador.tamanho();
    estado.grafo.aplicarLayout('grade', largura, altura);
    renderizador.desenhar(estado.grafo, {});
    sincronizarJson();
  });

  ui.aplicarJson.addEventListener('click', () => {
    try {
      const grafo = Grafo.doTexto(ui.json.value);
      const { largura, altura } = renderizador.tamanho();
      if (!grafo.posicionado()) grafo.aplicarLayout('circular', largura, altura);
      aplicarGrafo(grafo);
      ui.erro.textContent = '';
      ui.erro.hidden = true;
      ui.mensagem.textContent = 'JSON aplicado.';
    } catch (erro) {
      ui.erro.textContent = erro.message;
      ui.erro.hidden = false;
    }
  });

  ui.copiarJson.addEventListener('click', async () => {
    sincronizarJson();
    try {
      await navigator.clipboard.writeText(ui.json.value);
      ui.mensagem.textContent = 'JSON do grafo copiado para a área de transferência.';
    } catch {
      ui.json.select();
      ui.mensagem.textContent = 'Não consegui copiar: o JSON está selecionado, use ctrl+c.';
    }
  });

  ui.tocar.addEventListener('click', () => player.alternar());
  el('btn-proximo').addEventListener('click', () => {
    player.pausar();
    player.proximo();
  });
  el('btn-anterior').addEventListener('click', () => {
    player.pausar();
    player.anterior();
  });
  el('btn-inicio').addEventListener('click', () => {
    player.pausar();
    player.primeiro();
  });
  el('btn-fim').addEventListener('click', () => {
    player.pausar();
    player.ultimo();
  });
  ui.velocidade.addEventListener('input', () => {
    ui.valorVelocidade.textContent = `${ui.velocidade.value}x`;
    player.definirVelocidade(ui.velocidade.value);
  });

  document.addEventListener('keydown', (evento) => {
    if (evento.target.matches('input, textarea, select') || !el('modal').hidden) return;
    if (evento.code === 'Space') {
      evento.preventDefault();
      player.alternar();
    } else if (evento.key === 'ArrowRight') {
      evento.preventDefault();
      player.pausar();
      player.proximo();
    } else if (evento.key === 'ArrowLeft') {
      evento.preventDefault();
      player.pausar();
      player.anterior();
    } else if (evento.key === 'Home') {
      player.pausar();
      player.primeiro();
    } else if (evento.key === 'End') {
      player.pausar();
      player.ultimo();
    }
  });

  let temporizador = null;
  window.addEventListener('resize', () => {
    if (temporizador) clearTimeout(temporizador);
    temporizador = setTimeout(() => renderizador.desenhar(estado.grafo, renderizador.estado ?? {}), 150);
  });

  atualizarDescricaoAlgoritmo();
}

function atualizarDescricaoAlgoritmo() {
  const algoritmo = ui.algoritmo.value;
  ui.descricaoAlgoritmo.textContent = ALGORITMOS[algoritmo].descricao;
  ui.campoIteracoes.hidden = algoritmo !== 'ids';
  ui.campoLimite.hidden = algoritmo !== 'dls';
}

function formatar(valor) {
  if (!Number.isFinite(valor)) return '∞';
  const arredondado = Math.round(valor * 100) / 100;
  return Number.isInteger(arredondado) ? String(arredondado) : arredondado.toFixed(2);
}
