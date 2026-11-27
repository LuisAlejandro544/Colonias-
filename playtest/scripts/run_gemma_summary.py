#!/usr/bin/env python3
"""
Cliente en Python estándar para consultar a Gemma 4 E2B (solo texto + razonamiento)
vía `llama-server` local dentro de GitHub Actions y generar el resumen Markdown.

Lógica:
- Lee el prompt del sistema en `playtest/prompts/gemma4_playtest_prompt.txt`
  (ignorando las líneas de comentarios `#`).
- Lee el reporte compacto `gemma_prompt_input.txt` generado por Rust.
- Envía una petición HTTP POST a `http://127.0.0.1:8080/v1/chat/completions`
  activando `"chat_template_kwargs": {"enable_thinking": true}`.
- Separa limpiamente el razonamiento (`reasoning_content` o etiquetas
  `<|channel>thought ... <channel|>`) de la respuesta final en español.
- Guarda `gemma_summary.md` y lo publica en `$GITHUB_STEP_SUMMARY`.
"""

import json
import os
import re
import sys
import urllib.error
import urllib.request


def load_system_prompt(path: str) -> str:
    with open(path, "r", encoding="utf-8") as fh:
        lines = [line for line in fh if not line.lstrip().startswith("#")]
    return "".join(lines).strip()


def split_gemma4_thought(raw_content: str, reasoning_field: str) -> tuple[str, str]:
    thought = (reasoning_field or "").strip()
    answer = (raw_content or "").strip()

    pattern = re.compile(r"<\|channel>thought\s*(.*?)\s*<channel\|>", re.DOTALL)
    match = pattern.search(answer)
    if match:
        extracted = match.group(1).strip()
        if extracted and not thought:
            thought = extracted
        answer = pattern.sub("", answer).strip()

    answer = answer.replace("<turn|>", "").replace("<|turn>", "").strip()
    thought = thought.replace("<turn|>", "").replace("<|turn>", "").strip()
    return thought, answer


def main() -> int:
    if len(sys.argv) < 4:
        print(
            "Uso: run_gemma_summary.py <system_prompt.txt> <gemma_prompt_input.txt> <out_summary.md>",
            file=sys.stderr,
        )
        return 1

    prompt_path, input_path, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
    system_prompt = load_system_prompt(prompt_path)
    with open(input_path, "r", encoding="utf-8") as fh:
        report_text = fh.read().strip()

    payload = {
        "model": "gemma-4-E2B-it",
        "messages": [
            {"role": "system", "content": system_prompt},
            {
                "role": "user",
                "content": (
                    "Analiza este reporte del playtest por neuroevolución y entrega "
                    "el diagnóstico completo siguiendo el formato pedido:\n\n"
                    + report_text
                ),
            },
        ],
        "temperature": 0.6,
        "top_p": 0.95,
        "max_tokens": 1400,
        "chat_template_kwargs": {"enable_thinking": True},
    }

    url = os.environ.get("LLAMA_SERVER_URL", "http://127.0.0.1:8080/v1/chat/completions")
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except (urllib.error.URLError, TimeoutError) as exc:
        print(f"Error llamando a llama-server: {exc}", file=sys.stderr)
        return 1

    choices = data.get("choices") or []
    if not choices:
        print("Respuesta vacía de Gemma 4 E2B.", file=sys.stderr)
        return 1

    msg = choices[0].get("message") or {}
    thought, answer = split_gemma4_thought(
        msg.get("content", ""),
        msg.get("reasoning_content", ""),
    )

    md_parts = [
        "# Resumen IA — Gemma 4 E2B (Solo Texto + Razonamiento)\n",
        answer if answer else "_Gemma 4 E2B no devolvió respuesta final fuera del canal de pensamiento._",
        "\n",
    ]
    if thought:
        md_parts.append(
            "\n<details>\n<summary>Ver razonamiento interno de Gemma 4 E2B (&lt;|channel&gt;thought)</summary>\n\n"
            + thought
            + "\n\n</details>\n"
        )

    md_parts.append(
        "\n<details>\n<summary>Datos crudos enviados desde Rust (gemma_prompt_input.txt)</summary>\n\n```text\n"
        + report_text
        + "\n```\n\n</details>\n"
    )

    final_md = "\n".join(md_parts)
    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write(final_md)

    print(final_md)

    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_path:
        with open(summary_path, "a", encoding="utf-8") as fh:
            fh.write(final_md + "\n")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
