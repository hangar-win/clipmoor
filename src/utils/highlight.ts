import hljs from 'highlight.js/lib/core'
import bash from 'highlight.js/lib/languages/bash'
import csharp from 'highlight.js/lib/languages/csharp'
import css from 'highlight.js/lib/languages/css'
import javascript from 'highlight.js/lib/languages/javascript'
import json from 'highlight.js/lib/languages/json'
import powershell from 'highlight.js/lib/languages/powershell'
import python from 'highlight.js/lib/languages/python'
import rust from 'highlight.js/lib/languages/rust'
import sql from 'highlight.js/lib/languages/sql'
import typescript from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml'
import yaml from 'highlight.js/lib/languages/yaml'

const LANGS = { sql, csharp, typescript, javascript, json, xml, css, python, rust, bash, powershell, yaml }
for (const [name, lang] of Object.entries(LANGS)) hljs.registerLanguage(name, lang)

export type Tone = 'keyword' | 'string' | 'comment' | 'number' | 'function' | 'type'
export type Segment = { text: string; tone: Tone | null }

const TONES: Record<string, Tone | null> = {
  keyword: 'keyword', meta: 'keyword', doctag: 'keyword', name: 'keyword', 'selector-tag': 'keyword', 'selector-pseudo': 'keyword', bullet: 'keyword', 'variable language_': 'keyword',
  string: 'string', regexp: 'string', symbol: 'string', char: 'string', link: 'string', addition: 'string',
  comment: 'comment', quote: 'comment',
  number: 'number', literal: 'number', deletion: 'number',
  title: 'function', 'title function_': 'function', section: 'function', attr: 'function', attribute: 'function', property: 'function',
  type: 'type', built_in: 'type', 'title class_': 'type', 'title class_ inherited__': 'type', variable: 'type', 'template-variable': 'type', 'selector-class': 'type', 'selector-id': 'type',
  subst: null, params: null, operator: null, punctuation: null, tag: null,
}

export function toneOf(scope: string, outer: Tone | null = null): Tone | null {
  const key = scope in TONES ? scope : scope.split(' ')[0]
  return key in TONES ? TONES[key] : outer
}

const ENTITIES: Record<string, string> = { '&amp;': '&', '&lt;': '<', '&gt;': '>', '&quot;': '"', '&#x27;': "'" }

export function highlight(code: string): { language?: string; segments: Segment[] } {
  const r = hljs.highlightAuto(code, Object.keys(LANGS))
  const stack: (Tone | null)[] = []
  const segments: Segment[] = []
  for (const [, open, close, text] of r.value.matchAll(/<span class="([^"]*)">|(<\/span>)|([^<]+)/g)) {
    const outer = stack.length ? stack[stack.length - 1] : null
    if (open !== undefined) stack.push(toneOf(open.replace(/hljs-/g, ''), outer))
    else if (close) stack.pop()
    else segments.push({ text: text.replace(/&(amp|lt|gt|quot|#x27);/g, (e) => ENTITIES[e]), tone: outer })
  }
  return { language: r.language && hljs.getLanguage(r.language)?.name, segments }
}
