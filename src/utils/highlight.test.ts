import { expect, test } from 'vitest'
import { highlight, toneOf } from './highlight'

const text = (code: string) => highlight(code).segments.map((s) => s.text).join('')
const tones = (code: string) => highlight(code).segments.filter((s) => s.tone).map((s) => [s.tone, s.text])

test('detects the language among the registered subset', () => {
  expect(highlight('SELECT id, name FROM users WHERE id = 1 ORDER BY name;').language).toBe('SQL')
  expect(highlight('public class Foo\n{\n    public async Task<int> BarAsync(string s) => await Task.FromResult(s.Length);\n}').language).toBe('C#')
  expect(highlight('fn main() {\n    let mut v: Vec<i32> = Vec::new();\n    println!("{}", v.len());\n}').language).toBe('Rust')
})

test('segments rebuild the exact source, entities decoded', () => {
  const code = 'if (a < b && c > "d") { return \'e\' }\n\t  x = 1'
  expect(text(code)).toBe(code)
  expect(text('<div class="a">&amp;</div>')).toBe('<div class="a">&amp;</div>')
})

test('scopes map to tones', () => {
  expect(tones('SELECT 1 -- note')).toEqual(expect.arrayContaining([['keyword', 'SELECT'], ['number', '1'], ['comment', '-- note']]))
  expect(toneOf('title class_')).toBe('type')
  expect(toneOf('title function_')).toBe('function')
  expect(toneOf('subst', 'string')).toBeNull()
  expect(toneOf('unknown', 'string')).toBe('string')
})
