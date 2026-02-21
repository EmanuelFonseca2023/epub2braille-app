<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { goto } from "$app/navigation";
  import Resumen from "$lib/Resumen.svelte";

  type Estado =
    | { tipo: "idle" }
    | { tipo: "cargando" }
    | { tipo: "ok"; ruta: string; epubPath: string; bytes: number }
    | { tipo: "error"; mensaje: string };

  let estado: Estado = { tipo: "idle" };
  let arrastrando = false;

  async function convertir(path: string) {
    estado = { tipo: "cargando" };
    try {
      const salida = await invoke<string>("convertir", { path });
      const bytes = await invoke<number>("obtener_tamano", { path: salida });
      estado = { tipo: "ok", ruta: salida, epubPath: path, bytes };
    } catch (e) {
      estado = { tipo: "error", mensaje: String(e) };
    }
  }

  async function abrirEditor(path: string) {
    estado = { tipo: "cargando" };
    try {
      const texto = await invoke<string>("extraer_texto", { path });
      sessionStorage.setItem("editor_texto", texto);
      sessionStorage.setItem("editor_bin_path", path.replace(".epub", "_braille.bin"));
      goto("/editor");
    } catch (e) {
      estado = { tipo: "error", mensaje: String(e) };
    }
  }

  async function abrirDialogo(modo: "convertir" | "editar") {
    const seleccionado = await open({
      multiple: false,
      filters: [{ name: "EPUB", extensions: ["epub"] }],
    });
    if (!seleccionado) return;
    const path = seleccionado as string;
    if (modo === "editar") abrirEditor(path);
    else convertir(path);
  }

  function onDragOver(e: DragEvent) { e.preventDefault(); arrastrando = true; }
  function onDragLeave() { arrastrando = false; }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    arrastrando = false;
    const file = e.dataTransfer?.files[0];
    if (!file) return;
    if (!file.name.endsWith(".epub")) {
      estado = { tipo: "error", mensaje: "Solo se aceptan archivos .epub" };
      return;
    }
    const path = (file as File & { path?: string }).path ?? file.name;
    convertir(path);
  }
</script>

<main>
  <div class="titulo">
    <h1>EPUB → Braille</h1>
    <p>Convierte libros digitales a formato binario braille</p>
  </div>

  <button
    class="zona {arrastrando ? 'sobre' : ''} {estado.tipo}"
    on:click={() => abrirDialogo("convertir")}
    on:dragover={onDragOver}
    on:dragleave={onDragLeave}
    on:drop={onDrop}
    disabled={estado.tipo === "cargando"}
  >
    {#if estado.tipo === "cargando"}
      <div class="spinner"></div>
      <span>Procesando...</span>
    {:else if estado.tipo === "ok"}
      <span class="icono">✓</span>
      <span>¡Conversión exitosa!</span>
      <small>Arrastra otro para convertir</small>
    {:else if estado.tipo === "error"}
      <span class="icono error">!</span>
      <span>Error al convertir</span>
      <small>Arrastra otro para reintentar</small>
    {:else}
      <span class="icono">↑</span>
      <span>Arrastra tu <code>.epub</code> aquí</span>
      <small>o haz clic para convertir directo</small>
    {/if}
  </button>

  {#if estado.tipo === "ok"}
    <div class="resultado ok">
      <span class="label">Archivo guardado en:</span>
      <span class="ruta">{estado.ruta}</span>
    </div>
    <Resumen bytes={estado.bytes} />
    <button class="btn-secundario" on:click={() => estado.tipo === "ok" && abrirEditor(estado.epubPath)}>
      Editar este archivo
    </button>
  {/if}

  {#if estado.tipo === "error"}
    <div class="resultado error">
      {estado.mensaje}
    </div>
  {/if}

  {#if estado.tipo === "idle" || estado.tipo === "error"}
    <button class="btn-secundario" on:click={() => abrirDialogo("editar")}>
      Editar antes de convertir
    </button>
  {/if}
</main>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) { background: #0f0f13; color: #e8e8e8; font-family: system-ui, sans-serif; }

  main {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2rem;
    padding: 2rem;
  }

  .titulo { text-align: center; }
  .titulo h1 { font-size: 1.6rem; font-weight: 600; letter-spacing: -0.5px; }
  .titulo p { color: #666; font-size: 0.9rem; margin-top: 0.3rem; }

  .zona {
    width: 320px;
    height: 220px;
    border: 2px dashed #333;
    border-radius: 20px;
    background: #16161d;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    cursor: pointer;
    transition: all 0.2s;
    color: #666;
    font-size: 0.95rem;
  }
  .zona:hover { border-color: #555; color: #999; }
  .zona.sobre { border-color: #7c6af7; background: #1a1830; color: #c0b8ff; }
  .zona.ok { border-color: #6fcf97; color: #6fcf97; }
  .zona.error { border-color: #eb5757; color: #eb5757; }

  .icono { font-size: 2.5rem; line-height: 1; }
  .icono.error { color: #eb5757; }
  small { font-size: 0.78rem; color: #444; }
  code { color: #a78bfa; }

  .spinner {
    width: 36px; height: 36px;
    border: 3px solid #333;
    border-top-color: #7c6af7;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .resultado {
    width: 320px;
    border-radius: 12px;
    padding: 1rem;
    font-size: 0.82rem;
    background: #16161d;
  }
  .resultado.ok { border: 1px solid #2d5a3d; }
  .resultado.error { border: 1px solid #5a2d2d; color: #eb5757; }
  .label { display: block; color: #555; margin-bottom: 0.3rem; }
  .ruta { color: #6fcf97; font-family: monospace; word-break: break-all; }

  .btn-secundario {
    background: none;
    border: 1px solid #2a2a35;
    border-radius: 10px;
    color: #555;
    font-size: 0.82rem;
    padding: 0.45rem 1.1rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  .btn-secundario:hover { border-color: #7c6af7; color: #a78bfa; }
</style>