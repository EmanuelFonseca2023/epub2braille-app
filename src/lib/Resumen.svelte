<script lang="ts">
  export let bytes: number;

  const LINEAS_POR_PAGINA = 25;
  const PAGINAS_POR_MIN = 12;

  $: lineas = Math.floor(bytes / 31);
  $: paginas = Math.ceil(lineas / LINEAS_POR_PAGINA);
  $: minutos = Math.ceil(paginas / PAGINAS_POR_MIN);
</script>

<div class="resumen">
  <div class="fila">
    <span class="label">Líneas Braille</span>
    <span class="valor">{lineas.toLocaleString()}</span>
  </div>
  <div class="fila">
    <span class="label">Páginas</span>
    <span class="valor">{paginas.toLocaleString()}</span>
  </div>
  <div class="fila">
    <span class="label">Hojas (1 cara)</span>
    <span class="valor">{paginas.toLocaleString()}</span>
  </div>
  <div class="fila">
    <span class="label">Hojas (2 caras)</span>
    <span class="valor">~{Math.ceil(paginas / 2).toLocaleString()} <span class="tag">beta</span></span>
  </div>
  <div class="fila">
    <span class="label">Tiempo estimado</span>
    <span class="valor">{minutos} min</span>
  </div>
</div>

<style>
  .resumen {
    width: 320px;
    background: #16161d;
    border: 1px solid #2a2a35;
    border-radius: 12px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .fila {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.82rem;
  }
  .label { color: #555; }
  .valor { color: #e8e8e8; font-family: monospace; display: flex; align-items: center; gap: 0.4rem; }
  .tag {
    font-size: 0.65rem;
    color: #7c6af7;
    border: 1px solid #2a2a35;
    border-radius: 4px;
    padding: 0.1rem 0.3rem;
    font-family: system-ui;
  }
</style>