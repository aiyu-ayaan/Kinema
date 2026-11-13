import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'
import { formatMessage, isRtl, resolveLanguage, RTL_LANGUAGES, SUPPORTED_LANGUAGES } from './languages.ts'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const localesDir = path.resolve(__dirname, '../locales')

function extractPlaceholders(text: string): string[] {
  return [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort()
}

test('SUPPORTED_LANGUAGES has all 27 canonical languages and RTL flags', () => {
  assert.equal(SUPPORTED_LANGUAGES.length, 27)
  assert.equal(SUPPORTED_LANGUAGES[0].code, 'en')
  assert.deepEqual([...RTL_LANGUAGES].sort(), ['ar', 'ur'])
  assert.equal(isRtl('ar'), true)
  assert.equal(isRtl('ur'), true)
  assert.equal(isRtl('en'), false)
  assert.equal(isRtl('hi'), false)
})

test('resolveLanguage normalizes tags and falls back to en', () => {
  assert.equal(resolveLanguage('pt-BR'), 'pt-BR')
  assert.equal(resolveLanguage('pt_br'), 'pt-BR')
  assert.equal(resolveLanguage('pt-PT'), 'pt-BR')
  assert.equal(resolveLanguage('pt'), 'pt-BR')
  assert.equal(resolveLanguage('zh-CN'), 'zh-CN')
  assert.equal(resolveLanguage('zh_TW'), 'zh-TW')
  assert.equal(resolveLanguage('zh-Hant'), 'zh-TW')
  assert.equal(resolveLanguage('zh-HK'), 'zh-TW')
  assert.equal(resolveLanguage('zh-Hans'), 'zh-CN')
  assert.equal(resolveLanguage('zh'), 'zh-CN')
  assert.equal(resolveLanguage('in'), 'id')
  assert.equal(resolveLanguage('in-ID'), 'id')
  assert.equal(resolveLanguage('es-419'), 'es')
  assert.equal(resolveLanguage('ar-EG'), 'ar')
  assert.equal(resolveLanguage('xx-YY'), 'en')
  assert.equal(resolveLanguage(''), 'en')
  assert.equal(resolveLanguage(null), 'en')
})

test('formatMessage interpolates variables and leaves unknown placeholders untouched', () => {
  assert.equal(formatMessage('Hello {name}, {n} items', { name: 'Aiyu', n: 3 }), 'Hello Aiyu, 3 items')
  assert.equal(formatMessage('Keep {unknown}', {}), 'Keep {unknown}')
})

test('all 27 locale files exist, parse, and match en.json keys and placeholders', () => {
  const en = JSON.parse(fs.readFileSync(path.join(localesDir, 'en.json'), 'utf8')) as Record<string, string>
  const enKeys = Object.keys(en).sort()
  assert.ok(enKeys.length > 600)

  for (const { code } of SUPPORTED_LANGUAGES) {
    const file = path.join(localesDir, `${code}.json`)
    assert.ok(fs.existsSync(file), `Missing locale file: ${code}.json`)
    const data = JSON.parse(fs.readFileSync(file, 'utf8')) as Record<string, unknown>
    const keys = Object.keys(data).sort()
    assert.deepEqual(keys, enKeys, `Key mismatch in ${code}.json`)

    for (const k of enKeys) {
      const val = data[k]
      assert.equal(typeof val, 'string', `Non-string value for ${code}.${k}`)
      assert.ok((val as string).trim().length > 0, `Empty translation for ${code}.${k}`)
      assert.deepEqual(
        extractPlaceholders(val as string),
        extractPlaceholders(en[k]),
        `Placeholder mismatch for ${code}.${k}`,
      )
    }
  }
})
