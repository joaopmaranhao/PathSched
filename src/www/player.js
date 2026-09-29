export class Player {
  constructor({ aoMudar, aoFim } = {}) {
    this.aoMudar = aoMudar ?? (() => {});
    this.aoFim = aoFim ?? (() => {});
    this.passos = [];
    this.indice = 0;
    this.velocidade = 8;
    this.tocando = false;
    this.relogio = null;
  }

  get total() {
    return this.passos.length;
  }

  get passo() {
    return this.passos[this.indice] ?? null;
  }

  get indiceValido() {
    return this.total > 0;
  }

  carregar(passos, { reiniciar = true } = {}) {
    this.pausar();
    this.passos = Array.isArray(passos) ? passos : [];
    if (reiniciar || this.indice > this.total - 1) this.indice = 0;
    this.aoMudar(this.indice, this.passo);
  }

  definirVelocidade(valor) {
    this.velocidade = Math.max(1, Math.min(60, Number(valor) || 8));
    if (this.tocando) {
      this.pausar();
      this.tocar();
    }
  }

  intervalo() {
    return Math.max(16, Math.round(1000 / this.velocidade));
  }

  tocar() {
    if (!this.indiceValido || this.tocando) return;
    if (this.indice >= this.total - 1) this.indice = 0;
    this.tocando = true;
    this.aoMudar(this.indice, this.passo);
    this.relogio = setInterval(() => this.proximo(), this.intervalo());
  }

  pausar() {
    if (this.relogio) clearInterval(this.relogio);
    this.relogio = null;
    this.tocando = false;
    this.aoMudar(this.indice, this.passo);
  }

  alternar() {
    if (this.tocando) this.pausar();
    else this.tocar();
  }

  proximo() {
    if (!this.indiceValido) return;
    if (this.indice >= this.total - 1) {
      this.pausar();
      this.aoFim();
      return;
    }
    this.indice += 1;
    this.aoMudar(this.indice, this.passo);
  }

  anterior() {
    if (!this.indiceValido) return;
    this.indice = Math.max(0, this.indice - 1);
    this.aoMudar(this.indice, this.passo);
  }

  irPara(indice) {
    if (!this.indiceValido) return;
    this.indice = Math.max(0, Math.min(this.total - 1, Math.round(indice)));
    this.aoMudar(this.indice, this.passo);
  }

  primeiro() {
    this.irPara(0);
  }

  ultimo() {
    this.irPara(this.total - 1);
  }

  destruir() {
    if (this.relogio) clearInterval(this.relogio);
    this.relogio = null;
  }
}
