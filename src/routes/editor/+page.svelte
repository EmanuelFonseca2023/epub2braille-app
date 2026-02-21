<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import Resumen from "$lib/Resumen.svelte";

  let texto = "";
  let binPath = "";
  let estado: "idle" | "guardando" | "ok" | "error" = "idle";
  let errorMsg = "";
  let maxCeldas = 30;
  let textareaEl: HTMLTextAreaElement;
  let numerosEl: HTMLDivElement;
  let bytesGenerados = 0;

  onMount(() => {
    texto = sessionStorage.getItem("editor_texto") ?? "";
    binPath = sessionStorage.getItem("editor_bin_path") ?? "";
  });

  $: lineas = texto.split("\n").map(l => ({
    celdas: [...l].length,
    excede: [...l].length > maxCeldas,
  }));

  function sincronizarScroll() {
    if (numerosEl) numerosEl.scrollTop = textareaEl.scrollTop;
  }

  async function guardar() {
    estado = "guardando";
    try {
      await invoke("convertir_texto", { texto, binPath });
      bytesGenerados = await invoke<number>("obtener_tamano", { path: binPath });
      estado = "ok";
    } catch (e) {
      errorMsg = String(e);
      estado = "error";
    }
  }
</script>

<main>
  <div class="cabecera">
    <button class="btn-volver" on:click={() => goto("/")}>← Volver</button>
    <div class="titulo">
      <h1>Editor Braille</h1>
      <p>Máximo <input class="input-celdas" type="number" min="1" max="60" bind:value={maxCeldas} /> celdas por línea</p>
    </div>
    <button class="btn-guardar" on:click={guardar} disabled={estado === "guardando"}>
      {estado === "guardando" ? "Generando..." : estado === "ok" ? "✓ Guardado" : "Generar .bin"}
    </button>
  </div>

  {#if estado === "error"}
    <div class="banner error">{errorMsg}</div>
  {/if}

  {#if estado === "ok"}
    <Resumen bytes={bytesGenerados} />
  {/if}

  <div class="editor">
    <div class="numeros" bind:this={numerosEl} aria-hidden="true">
      {#each lineas as linea}
        <div class="num {linea.excede ? 'excede' : ''}">{linea.celdas}/{maxCeldas}</div>
      {/each}
    </div>
    <textarea
      bind:this={textareaEl}
      on:scroll={sincronizarScroll}
      spellcheck="false"
      bind:value={texto}
    ></textarea>
  </div>
</main>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) { background: #0f0f13; color: #e8e8e8; font-family: system-ui, sans-serif; }

  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
    gap: 1rem;
  }

  .cabecera {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-shrink: 0;
  }

  .titulo { text-align: center; }
  .titulo h1 { font-size: 1.2rem; font-weight: 600; }
  .titulo p { color: #555; font-size: 0.82rem; margin-top: 0.2rem; display: flex; align-items: center; gap: 0.4rem; justify-content: center; }

  .input-celdas {
    width: 3rem;
    background: #16161d;
    border: 1px solid #2a2a35;
    border-radius: 6px;
    color: #a78bfa;
    font-size: 0.82rem;
    padding: 0.1rem 0.3rem;
    text-align: center;
  }

  .btn-volver {
    background: none;
    border: 1px solid #2a2a35;
    border-radius: 10px;
    color: #555;
    font-size: 0.82rem;
    padding: 0.45rem 1rem;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }
  .btn-volver:hover { border-color: #555; color: #999; }

  .btn-guardar {
    background: #16161d;
    border: 1px solid #333;
    border-radius: 10px;
    color: #a78bfa;
    font-size: 0.82rem;
    padding: 0.45rem 1.1rem;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }
  .btn-guardar:hover:not(:disabled) { border-color: #7c6af7; }
  .btn-guardar:disabled { opacity: 0.5; cursor: default; }

  .banner {
    border-radius: 10px;
    padding: 0.6rem 1rem;
    font-size: 0.82rem;
    flex-shrink: 0;
  }
  .banner.error { background: #1a0a0a; border: 1px solid #5a2d2d; color: #eb5757; }

  .editor {
    display: flex;
    flex: 1;
    border: 1px solid #2a2a35;
    border-radius: 12px;
    overflow: hidden;
    background: #16161d;
    min-height: 0;
  }

  .numeros {
    padding: 0.75rem 0;
    background: #111118;
    border-right: 1px solid #2a2a35;
    min-width: 4.5rem;
    text-align: right;
    overflow: hidden;
  }

  .num {
    font-size: 0.72rem;
    font-family: monospace;
    color: #333;
    padding: 0 0.5rem;
    line-height: 1.6rem;
    height: 1.6rem;
  }
  .num.excede { color: #eb5757; }

  textarea {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #e8e8e8;
    font-family: monospace;
    font-size: 0.9rem;
    line-height: 1.6rem;
    padding: 0.75rem;
    resize: none;
    overflow-y: scroll;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>