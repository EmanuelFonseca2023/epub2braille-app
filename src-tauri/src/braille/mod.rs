use std::fs::{self, File};
use std::io::Read;
use zip::ZipArchive;
use scraper::{Html, Selector};

// ─── Braille: puntos activos → byte ────────────────────────────────────────

fn puntos_a_byte(puntos: &[u8]) -> u8 {
    puntos.iter().fold(0u8, |acc, &p| acc | (1 << (p - 1)))
}

fn letra_minuscula(c: char) -> Option<Vec<u8>> {
    match c {
        'a' => Some(vec![1]),
        'b' => Some(vec![1, 2]),
        'c' => Some(vec![1, 4]),
        'd' => Some(vec![1, 4, 5]),
        'e' => Some(vec![1, 5]),
        'f' => Some(vec![1, 2, 4]),
        'g' => Some(vec![1, 2, 4, 5]),
        'h' => Some(vec![1, 2, 5]),
        'i' => Some(vec![2, 4]),
        'j' => Some(vec![2, 4, 5]),
        'k' => Some(vec![1, 3]),
        'l' => Some(vec![1, 2, 3]),
        'm' => Some(vec![1, 3, 4]),
        'n' => Some(vec![1, 3, 4, 5]),
        'o' => Some(vec![1, 3, 5]),
        'p' => Some(vec![1, 2, 3, 4]),
        'q' => Some(vec![1, 2, 3, 4, 5]),
        'r' => Some(vec![1, 2, 3, 5]),
        's' => Some(vec![2, 3, 4]),
        't' => Some(vec![2, 3, 4, 5]),
        'u' => Some(vec![1, 3, 6]),
        'v' => Some(vec![1, 2, 3, 6]),
        'w' => Some(vec![2, 4, 5, 6]),
        'x' => Some(vec![1, 3, 4, 6]),
        'y' => Some(vec![1, 3, 4, 5, 6]),
        'z' => Some(vec![1, 3, 5, 6]),
        '\u{00E1}' => Some(vec![1, 2, 3, 5, 6]), // á
        '\u{00E9}' => Some(vec![2, 3, 4, 6]),     // é
        '\u{00ED}' => Some(vec![3, 4]),            // í
        '\u{00F3}' => Some(vec![3, 4, 6]),         // ó
        '\u{00FA}' => Some(vec![2, 3, 4, 5, 6]),  // ú
        '\u{00FC}' => Some(vec![1, 2, 5, 6]),      // ü
        '\u{00F1}' => Some(vec![1, 2, 4, 5, 6]),  // ñ
        _ => None,
    }
}

fn digito_a_puntos(c: char) -> Option<Vec<u8>> {
    match c {
        '1' => Some(vec![1]),         // = a
        '2' => Some(vec![1, 2]),      // = b
        '3' => Some(vec![1, 4]),      // = c
        '4' => Some(vec![1, 4, 5]),   // = d
        '5' => Some(vec![1, 5]),      // = e
        '6' => Some(vec![1, 2, 4]),   // = f
        '7' => Some(vec![1, 2, 4, 5]),// = g
        '8' => Some(vec![1, 2, 5]),   // = h
        '9' => Some(vec![2, 4]),      // = i
        '0' => Some(vec![2, 4, 5]),   // = j
        _ => None,
    }
}

fn puntuacion_a_puntos(c: char) -> Option<Vec<u8>> {
    match c {
        '.' => Some(vec![3]),
        ',' => Some(vec![2]),
        ';' => Some(vec![2, 3]),
        ':' => Some(vec![2, 5]),
        // Guion tipográfico normal (no de división braille):
        '-' => Some(vec![3, 6]),
        '?' | '\u{00BF}' => Some(vec![2, 6]),
        '!' | '\u{00A1}' => Some(vec![2, 3, 5]),
        '"' | '\u{201C}' | '\u{201D}' => Some(vec![2, 3, 6]),
        '(' => Some(vec![1, 2, 6]),
        ')' => Some(vec![3, 4, 5]),
        _ => None,
    }
}

const PREFIJO_MAYUSCULA: &[u8] = &[4, 6];
const PREFIJO_NUMERO: &[u8] = &[3, 4, 5, 6];

// Guion de división braille (CBE/ONCE): puntos 3,6
// Se inserta al final de la línea cuando se parte una palabra por sílabas.
const GUION_DIVISION: &[u8] = &[3, 6];

fn es_mayuscula(c: char) -> bool {
    c.is_uppercase()
}

fn a_minuscula(c: char) -> char {
    match c {
        '\u{00C1}' => '\u{00E1}',
        '\u{00C9}' => '\u{00E9}',
        '\u{00CD}' => '\u{00ED}',
        '\u{00D3}' => '\u{00F3}',
        '\u{00DA}' => '\u{00FA}',
        '\u{00DC}' => '\u{00FC}',
        '\u{00D1}' => '\u{00F1}',
        _ => c.to_lowercase().next().unwrap_or(c),
    }
}

fn palabra_es_all_caps(palabra: &str) -> bool {
    let letras: Vec<char> = palabra.chars().filter(|c| c.is_alphabetic()).collect();
    !letras.is_empty() && letras.iter().all(|c| es_mayuscula(*c))
}

// ─── Celda braille codificada con metadatos de posición ──────────────────────
// Necesitamos saber qué celdas son "prefijos" para no separarlos de su carácter
// y qué celdas son inicio de sílaba para poder insertar el guion de división.

#[derive(Clone, Debug)]
struct CeldaInfo {
    byte: u8,
    /// true si esta celda es un prefijo (mayúscula o número) que NO puede
    /// quedar al final de línea separada del carácter al que precede.
    es_prefijo: bool,
    /// Índice de sílaba al que pertenece esta celda dentro de su palabra.
    /// None si no forma parte de una palabra (espacio, puntuación, etc.).
    silaba_idx: Option<u32>,
    /// true si es la primera celda de una nueva sílaba (punto de corte posible).
    inicio_silaba: bool,
    /// true si es la primera celda de la palabra (útil para el traslado entero).
    inicio_palabra: bool,
}

// ─── Silabador español ────────────────────────────────────────────────────────
// Implementación de las reglas fonéticas de la RAE / CBE (sin dependencias externas).

fn es_vocal(c: char) -> bool {
    matches!(c.to_lowercase().next().unwrap_or(c),
        'a' | 'e' | 'i' | 'o' | 'u' | '\u{00E1}' | '\u{00E9}' | '\u{00ED}' |
        '\u{00F3}' | '\u{00FA}' | '\u{00FC}'
    )
}

fn es_vocal_fuerte(c: char) -> bool {
    matches!(c.to_lowercase().next().unwrap_or(c),
        'a' | 'e' | 'o' | '\u{00E1}' | '\u{00E9}' | '\u{00F3}'
    )
}

fn es_vocal_debil_acentuada(c: char) -> bool {
    matches!(c, '\u{00ED}' | '\u{00FA}')
}

fn forman_diptongo(v1: char, v2: char) -> bool {
    // Hiato: dos vocales fuertes
    if es_vocal_fuerte(v1) && es_vocal_fuerte(v2) {
        return false;
    }
    // Hiato: vocal débil acentuada (rompe el diptongo)
    if es_vocal_debil_acentuada(v1) || es_vocal_debil_acentuada(v2) {
        return false;
    }
    true
}

// Grupos consonánticos inseparables (van juntos a la sílaba siguiente)
fn es_grupo_inseparable(c1: char, c2: char) -> bool {
    let p: &[(&str,)] = &[
        ("bl",), ("br",), ("cl",), ("cr",), ("dr",),
        ("fl",), ("fr",), ("gl",), ("gr",), ("pl",),
        ("pr",), ("tr",), ("ch",), ("ll",), ("rr",),
    ];
    let s = format!("{}{}", c1.to_lowercase().next().unwrap_or(c1),
                            c2.to_lowercase().next().unwrap_or(c2));
    p.iter().any(|(g,)| *g == s.as_str())
}

/// Retorna los índices (en bytes) donde comienza cada sílaba dentro de `palabra`.
/// El primer elemento siempre es 0.
fn silabear(palabra: &str) -> Vec<usize> {
    let chars: Vec<char> = palabra.chars().collect();
    let n = chars.len();
    if n == 0 {
        return vec![];
    }

    // Construir tabla de índices byte → char
    let char_to_byte: Vec<usize> = {
        let mut v = Vec::with_capacity(n + 1);
        let mut b = 0usize;
        for &c in &chars {
            v.push(b);
            b += c.len_utf8();
        }
        v.push(b);
        v
    };

    // Índices de inicio de sílaba (en posición de char)
    let mut cortes_char: Vec<usize> = vec![0]; // siempre empieza en 0
    let mut i = 0usize;

    while i < n {
        // Avanzar consonantes iniciales hasta vocal
        while i < n && !es_vocal(chars[i]) {
            i += 1;
        }
        if i >= n { break; }

        // Consumir la vocal (y posible diptongo / triptongo)
        i += 1;
        if i < n && es_vocal(chars[i]) && forman_diptongo(chars[i - 1], chars[i]) {
            i += 1;
            // Triptongo
            if i < n && es_vocal(chars[i]) && forman_diptongo(chars[i - 1], chars[i]) {
                i += 1;
            }
        }

        if i >= n { break; }

        // Contar consonantes hasta la siguiente vocal
        let cons_start = i;
        while i < n && !es_vocal(chars[i]) {
            i += 1;
        }
        let num_cons = i - cons_start;

        if i >= n {
            // Final de palabra: no hay más cortes
            break;
        }

        if num_cons == 0 {
            // Hiato: dos vocales que no forman diptongo
            if !forman_diptongo(chars[i - 1], chars[i]) {
                cortes_char.push(i);
            }
        } else if num_cons == 1 {
            // Una consonante → va con la sílaba siguiente
            cortes_char.push(cons_start);
            i = cons_start + 1;
        } else if num_cons == 2 {
            if es_grupo_inseparable(chars[cons_start], chars[cons_start + 1]) {
                cortes_char.push(cons_start);
            } else {
                cortes_char.push(cons_start + 1);
            }
            i = cons_start + num_cons;
        } else if num_cons == 3 {
            // ¿Las dos últimas forman grupo inseparable?
            if es_grupo_inseparable(chars[cons_start + 1], chars[cons_start + 2]) {
                cortes_char.push(cons_start + 1);
            } else {
                cortes_char.push(cons_start + 2);
            }
            i = cons_start + num_cons;
        } else {
            // 4+ consonantes (muy raro): corte a la mitad
            let mid = num_cons / 2;
            cortes_char.push(cons_start + mid);
            i = cons_start + num_cons;
        }
    }

    cortes_char.iter().map(|&ci| char_to_byte[ci]).collect()
}

// ─── Codificador de texto → Vec<CeldaInfo> ───────────────────────────────────

fn codificar_texto(texto: &str) -> Vec<CeldaInfo> {
    let mut resultado: Vec<CeldaInfo> = Vec::new();
    let mut modo_numerico = false;
    let chars: Vec<char> = texto.chars().collect();
    let n = chars.len();
    let mut i = 0;

    let push = |resultado: &mut Vec<CeldaInfo>, byte: u8, es_pref: bool,
                sil: Option<u32>, inicio_s: bool, inicio_p: bool| {
        resultado.push(CeldaInfo {
            byte,
            es_prefijo: es_pref,
            silaba_idx: sil,
            inicio_silaba: inicio_s,
            inicio_palabra: inicio_p,
        });
    };

    while i < n {
        let c = chars[i];

        // ── Espacios y saltos de línea ──
        if c == ' ' || c == '\n' || c == '\r' {
            push(&mut resultado, puntos_a_byte(&[]), false, None, false, false);
            modo_numerico = false;
            i += 1;
            continue;
        }

        // ── Dígitos ──
        if let Some(puntos) = digito_a_puntos(c) {
            if !modo_numerico {
                push(&mut resultado, puntos_a_byte(PREFIJO_NUMERO), true, None, false, false);
                modo_numerico = true;
            }
            push(&mut resultado, puntos_a_byte(&puntos), false, None, false, false);
            i += 1;
            continue;
        }

        // ── Letras ──
        // Procesamos la palabra COMPLETA de una vez al encontrar su primer carácter,
        // y avanzamos i hasta el final. Esto evita recalcular offsets letra a letra.
        if c.is_alphabetic() {
            modo_numerico = false;

            // Solo procesamos si estamos en el inicio de la palabra
            let en_inicio_palabra = i == 0 || !chars[i - 1].is_alphabetic();
            if !en_inicio_palabra {
                // No debería ocurrir: siempre saltamos al fin_pal tras procesar
                i += 1;
                continue;
            }

            // Fin de la palabra
            let fin_pal = {
                let mut j = i;
                while j < n && chars[j].is_alphabetic() { j += 1; }
                j
            };
            let num_chars = fin_pal - i;
            let palabra: String = chars[i..fin_pal].iter().collect();

            // Silabear y construir mapas por carácter
            let silaba_inicios = silabear(&palabra);

            let mut char_silaba   = vec![0u32;  num_chars];
            let mut char_inicio_s = vec![false; num_chars];
            {
                let mut sil_idx  = 0u32;
                let mut byte_off = 0usize;
                for (k, ch) in palabra.chars().enumerate() {
                    // El offset 0 es siempre inicio de primera sílaba, no marcamos
                    // inicio_silaba[0] porque no hay "corte antes del primer char".
                    if k > 0 && silaba_inicios.contains(&byte_off) {
                        sil_idx += 1;
                        char_inicio_s[k] = true;
                    }
                    char_silaba[k] = sil_idx;
                    byte_off += ch.len_utf8();
                }
            }

            // ALL CAPS → prefijo único antes de la primera letra
            if palabra_es_all_caps(&palabra) {
                push(&mut resultado,
                     puntos_a_byte(PREFIJO_MAYUSCULA),
                     true,               // es_prefijo → no separar del siguiente
                     Some(char_silaba[0]),
                     false,              // el prefijo no es "inicio_silaba" por sí solo
                     true);
                for (k, letra) in palabra.chars().enumerate() {
                    let min = a_minuscula(letra);
                    if let Some(puntos) = letra_minuscula(min) {
                        push(&mut resultado, puntos_a_byte(&puntos), false,
                             Some(char_silaba[k]),
                             char_inicio_s[k],
                             k == 0);
                    }
                }
            } else {
                // Palabra mixta o minúscula: letra a letra
                for (k, letra) in palabra.chars().enumerate() {
                    let sil      = char_silaba[k];
                    let is_s     = char_inicio_s[k];
                    let es_ini_p = k == 0;

                    if es_mayuscula(letra) {
                        // Prefijo individual de mayúscula
                        push(&mut resultado,
                             puntos_a_byte(PREFIJO_MAYUSCULA),
                             true, Some(sil), is_s, es_ini_p);
                        let min = a_minuscula(letra);
                        if let Some(puntos) = letra_minuscula(min) {
                            push(&mut resultado, puntos_a_byte(&puntos), false,
                                 Some(sil), false, false);
                        }
                    } else if let Some(puntos) = letra_minuscula(letra) {
                        push(&mut resultado, puntos_a_byte(&puntos), false,
                             Some(sil), is_s, es_ini_p);
                    }
                }
            }

            i = fin_pal; // saltar la palabra entera de una vez
            continue;
        }


        // ── Puntuación ──
        if let Some(puntos) = puntuacion_a_puntos(c) {
            // El punto decimal dentro de modo numérico reinicia el modo
            // (regla 9.2: "el punto interrumpe y reinicia").
            // Nota: el ejemplo de la doc muestra 6 celdas para "1.809" pero
            // eso contradice la regla textual. Seguimos la REGLA: nuevo prefijo
            // tras el punto → 7 celdas para "1.809".
            if c == '.' && modo_numerico {
                modo_numerico = false;
            }
            push(&mut resultado, puntos_a_byte(&puntos), false, None, false, false);
            i += 1;
            continue;
        }

        i += 1;
    }

    resultado
}

// ─── Formateador con silabación y reglas ONCE ─────────────────────────────────

const CELDAS_POR_LINEA: usize = 30;
const CTRL_SALTO_LINEA: u8 = 0xFF;

/// Byte del guion de división braille (puntos 3,6 = 0b00100100 = 36)
fn byte_guion_division() -> u8 {
    puntos_a_byte(GUION_DIVISION)
}



/// Formatea las celdas respetando:
/// 1. Máximo CELDAS_POR_LINEA por línea.
/// 2. Nunca separar un prefijo del carácter al que modifica.
/// 3. División silábica con guion braille al final de línea.
/// 4. Si no cabe ninguna sílaba completa, trasladar la palabra entera.
fn formatear_lineas(celdas: &[CeldaInfo]) -> Vec<u8> {
    let mut resultado: Vec<u8> = Vec::new();
    let n = celdas.len();

    // Construir líneas de objetos, luego serializar
    // Trabajamos con índices sobre `celdas`.

    let mut linea_actual: Vec<u8> = Vec::new(); // bytes en la línea en curso
    let mut i = 0usize;

    let flush_linea = |linea: &mut Vec<u8>, resultado: &mut Vec<u8>| {
        // Rellenar hasta CELDAS_POR_LINEA con celdas vacías
        while linea.len() < CELDAS_POR_LINEA {
            linea.push(0x00);
        }
        resultado.extend_from_slice(linea);
        resultado.push(CTRL_SALTO_LINEA);
        linea.clear();
    };

    while i < n {
        let celda = &celdas[i];

        // ── Espacio o salto de párrafo ──
        if celda.byte == 0x00 && !celda.es_prefijo {
            // El espacio siempre cabe (1 celda); si ya no cabe, flush primero
            if linea_actual.len() >= CELDAS_POR_LINEA {
                flush_linea(&mut linea_actual, &mut resultado);
            }
            linea_actual.push(0x00);
            i += 1;
            continue;
        }

        // ── Inicio de palabra alfabética ──
        if celda.inicio_palabra && celda.silaba_idx.is_some() {
            // Recopilar todas las celdas de esta palabra
            let inicio_pal = i;
            let mut fin_pal = i;
            while fin_pal < n && celdas[fin_pal].silaba_idx.is_some() {
                fin_pal += 1;
            }
            // celdas[inicio_pal..fin_pal] es la palabra completa

            colocar_palabra(
                &celdas[inicio_pal..fin_pal],
                &mut linea_actual,
                &mut resultado,
            );

            i = fin_pal;
            continue;
        }

        // ── Celdas no-palabra (puntuación, números, etc.) ──
        // Para prefijos: asegurar que el par (prefijo + carácter) no se parta.
        if celda.es_prefijo {
            let siguiente_size = if i + 1 < n { 1usize } else { 0 };
            let needed = 1 + siguiente_size;
            if linea_actual.len() + needed > CELDAS_POR_LINEA {
                flush_linea(&mut linea_actual, &mut resultado);
            }
            linea_actual.push(celda.byte);
            i += 1;
            if i < n && siguiente_size > 0 {
                linea_actual.push(celdas[i].byte);
                i += 1;
            }
        } else {
            if linea_actual.len() >= CELDAS_POR_LINEA {
                flush_linea(&mut linea_actual, &mut resultado);
            }
            linea_actual.push(celda.byte);
            i += 1;
        }
    }

    // Volcar última línea si tiene contenido
    if !linea_actual.is_empty() {
        flush_linea(&mut linea_actual, &mut resultado);
    }

    resultado
}

/// Coloca las celdas de una palabra respetando silabación y reglas ONCE.
fn colocar_palabra(
    celdas: &[CeldaInfo],
    linea: &mut Vec<u8>,
    resultado: &mut Vec<u8>,
) {
    let flush = |linea: &mut Vec<u8>, resultado: &mut Vec<u8>| {
        while linea.len() < CELDAS_POR_LINEA {
            linea.push(0x00);
        }
        resultado.extend_from_slice(linea);
        resultado.push(CTRL_SALTO_LINEA);
        linea.clear();
    };

    // Si la palabra cabe completa en el espacio restante → colocarla directa
    let total = celdas.len();
    let restante = CELDAS_POR_LINEA - linea.len();
    if total <= restante {
        for c in celdas {
            linea.push(c.byte);
        }
        return;
    }

    // La palabra no cabe completa. Intentar división silábica.
    // Identificar los puntos de corte (inicio de sílaba) dentro de `celdas`.
    // Un punto de corte es válido si: es inicio_silaba && !es_prefijo
    // (nunca cortamos dejando un prefijo al final).
    // Además, el corte debe dejar al menos una celda en la línea siguiente.

    // Recopilar posiciones de inicio de sílaba
    let mut cortes: Vec<usize> = Vec::new(); // índices dentro de `celdas`
    for (k, c) in celdas.iter().enumerate() {
        if k > 0 && c.inicio_silaba && !c.es_prefijo {
            // Verificar que la celda anterior NO es prefijo (si lo es, no podemos
            // cortar entre el prefijo y su carácter)
            if !celdas[k - 1].es_prefijo {
                cortes.push(k);
            }
        }
    }

    // Buscar el último corte que quepa en `restante - 1` (reservar celda para guion)
    // El guion de división ocupa 1 celda.
    let espacio_con_guion = if restante > 0 { restante - 1 } else { 0 };

    let mut corte_elegido: Option<usize> = None;
    for &k in cortes.iter().rev() {
        if k <= espacio_con_guion {
            // Hay al menos `celdas.len() - k` celdas para la línea siguiente
            if celdas.len() - k >= 1 {
                corte_elegido = Some(k);
                break;
            }
        }
    }

    if let Some(k) = corte_elegido {
        // Colocar la primera parte + guion de división
        for c in &celdas[..k] {
            linea.push(c.byte);
        }
        linea.push(byte_guion_division());
        flush(linea, resultado);

        // Colocar el resto de la palabra en la nueva línea (recursivo)
        colocar_palabra(&celdas[k..], linea, resultado);
    } else {
        // No hay corte silábico posible: trasladar la palabra entera a la línea siguiente
        // (regla ONCE: si no se puede dividir correctamente, pasar íntegra)
        if !linea.is_empty() {
            flush(linea, resultado);
        }
        // Ahora intentar colocar en la línea fresca
        // Si la palabra es más larga que toda una línea, dividir forzosamente
        // (caso extremo; en braille muy poco probable)
        if total > CELDAS_POR_LINEA {
            // División forzada sin guion (caso degenerado)
            let mut j = 0;
            while j < total {
                let chunk = std::cmp::min(CELDAS_POR_LINEA, total - j);
                for c in &celdas[j..j + chunk] {
                    linea.push(c.byte);
                }
                if j + chunk < total {
                    flush(linea, resultado);
                }
                j += chunk;
            }
        } else {
            for c in celdas {
                linea.push(c.byte);
            }
        }
    }
}

// ─── Lector de epub universal vía content.opf ────────────────────────────────

fn leer_opf_path(archive: &mut ZipArchive<File>) -> String {
    let mut entry = archive.by_name("META-INF/container.xml").unwrap();
    let mut xml = String::new();
    entry.read_to_string(&mut xml).unwrap();

    xml.lines()
        .find(|l| l.contains("full-path"))
        .and_then(|l| {
            let start = l.find("full-path=\"")? + 11;
            let end = l[start..].find('"')? + start;
            Some(l[start..end].to_string())
        })
        .expect("No se encontró full-path en container.xml")
}

fn leer_spine(archive: &mut ZipArchive<File>, opf_path: &str) -> Vec<String> {
    let mut entry = archive.by_name(opf_path).unwrap();
    let mut xml = String::new();
    entry.read_to_string(&mut xml).unwrap();

    let base = opf_path.rfind('/').map(|i| &opf_path[..i]).unwrap_or("");

    let mut manifest: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in xml.lines() {
        if line.contains("<item ") && line.contains("media-type=\"application/xhtml") {
            let id = extraer_atributo(line, "id").unwrap_or_default();
            let href = extraer_atributo(line, "href").unwrap_or_default();
            if !id.is_empty() && !href.is_empty() {
                manifest.insert(id, href);
            }
        }
    }

    let spine_start = xml.find("<spine").unwrap_or(0);
    let spine_end = xml.find("</spine>").unwrap_or(xml.len());
    let spine_xml = &xml[spine_start..spine_end];

    spine_xml.lines()
        .filter(|l| l.contains("<itemref "))
        .filter_map(|l| extraer_atributo(l, "idref"))
        .filter_map(|idref| manifest.get(&idref).cloned())
        .map(|href| {
            if href.starts_with('/') {
                href
            } else if base.is_empty() {
                href
            } else {
                format!("{}/{}", base, href)
            }
        })
        .collect()
}

fn extraer_atributo(line: &str, attr: &str) -> Option<String> {
    let buscar = format!("{}=\"", attr);
    let start = line.find(&buscar)? + buscar.len();
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

fn extraer_texto_xhtml(content: &str, selector: &Selector) -> String {
    let doc = Html::parse_document(content);
    doc.select(selector)
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
pub fn convertir_epub_a_bin(epub_path: &str) -> Result<String, String> {
    use scraper::Selector;

    let bin_path = format!("{}_braille.bin",
        epub_path.trim_end_matches(".epub"));

    let file = File::open(epub_path)
        .map_err(|e| format!("No se pudo abrir: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("No es un EPUB válido: {}", e))?;

    let opf_path = leer_opf_path(&mut archive);
    let archivos = leer_spine(&mut archive, &opf_path);

    let selector = Selector::parse("p").unwrap();
    let texto = archivos.iter()
        .filter_map(|path| {
            let path_limpio = path.split('#').next().unwrap_or(path);
            archive.by_name(path_limpio).ok().map(|mut entry| {
                let mut content = String::new();
                entry.read_to_string(&mut content).unwrap();
                extraer_texto_xhtml(&content, &selector) + "\n\n"
            })
        })
        .collect::<String>();

    let celdas_info = codificar_texto(&texto);
    let bytes_formateados = formatear_lineas(&celdas_info);

    fs::write(&bin_path, &bytes_formateados)
        .map_err(|e| format!("No se pudo escribir: {}", e))?;

    Ok(bin_path)
}
// ─── API pública para el editor ──────────────────────────────────────────────

pub fn extraer_texto_epub(epub_path: &str) -> Result<String, String> {
    use scraper::Selector;

    let file = File::open(epub_path)
        .map_err(|e| format!("No se pudo abrir: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("No es un EPUB válido: {}", e))?;

    let opf_path = leer_opf_path(&mut archive);
    let archivos = leer_spine(&mut archive, &opf_path);
    let selector = Selector::parse("p").unwrap();

    let texto = archivos.iter()
        .filter_map(|path| {
            let path_limpio = path.split('#').next().unwrap_or(path);
            archive.by_name(path_limpio).ok().map(|mut entry| {
                let mut content = String::new();
                entry.read_to_string(&mut content).unwrap();
                extraer_texto_xhtml(&content, &selector) + "\n\n"
            })
        })
        .collect::<String>();

    Ok(texto)
}

pub fn convertir_texto_a_bin(texto: &str, bin_path: &str) -> Result<String, String> {
    let celdas = codificar_texto(texto);
    let bytes = formatear_lineas(&celdas);
    fs::write(bin_path, &bytes)
        .map_err(|e| format!("No se pudo escribir: {}", e))?;
    Ok(bin_path.to_string())
}