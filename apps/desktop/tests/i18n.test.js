const { test } = require('node:test')
const assert = require('node:assert/strict')
const fs = require('fs')
const path = require('path')
const {
  t,
  setLanguage,
  getLanguage,
  isRtl,
  getDirection,
  resolveLanguage,
  catalogs,
  SUPPORTED_LANGUAGES,
  RTL_LANGUAGES,
} = require('../src/i18n')

const EXPECTED_TAGS = [
  'en', 'hi', 'es', 'pt-BR', 'id', 'ar', 'fr', 'ru', 'de',
  'tr', 'ja', 'ko', 'it', 'vi', 'th', 'zh-CN', 'zh-TW', 'pl',
  'bn', 'ta', 'te', 'mr', 'gu', 'kn', 'ml', 'pa', 'ur',
]

function placeholders(str) {
  return [...str.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort()
}

test('all 27 canonical languages are registered and have valid locale files with matching keys', () => {
  assert.equal(SUPPORTED_LANGUAGES.length, 27)
  assert.deepEqual(
    SUPPORTED_LANGUAGES.map((l) => l.tag),
    EXPECTED_TAGS,
  )

  const localesDir = path.join(__dirname, '../src/locales')
  const files = fs.readdirSync(localesDir).filter((f) => f.endsWith('.json')).sort()
  assert.deepEqual(
    files,
    EXPECTED_TAGS.map((tag) => `${tag}.json`).sort(),
  )

  const en = catalogs.en
  const enKeys = Object.keys(en).sort()
  assert.ok(enKeys.length > 0)

  for (const tag of EXPECTED_TAGS) {
    const dict = catalogs[tag]
    assert.ok(dict, `Missing catalog for ${tag}`)
    assert.deepEqual(Object.keys(dict).sort(), enKeys, `Key mismatch in ${tag}.json`)
    for (const key of enKeys) {
      assert.equal(typeof dict[key], 'string', `${tag}.json[${key}] must be a string`)
      assert.ok(dict[key].trim().length > 0, `${tag}.json[${key}] must not be empty`)
      assert.deepEqual(
        placeholders(dict[key]),
        placeholders(en[key]),
        `Placeholder mismatch in ${tag}.json[${key}]`,
      )
    }
  }
})

test('i18n resolves exact, region, script, legacy, and fallback language tags', () => {
  assert.equal(resolveLanguage(['en-US']), 'en')
  assert.equal(resolveLanguage(['fr-FR']), 'fr')
  assert.equal(resolveLanguage(['pt-BR']), 'pt-BR')
  assert.equal(resolveLanguage(['pt-pt']), 'pt-BR')
  assert.equal(resolveLanguage(['zh-CN']), 'zh-CN')
  assert.equal(resolveLanguage(['zh_Hans_CN']), 'zh-CN')
  assert.equal(resolveLanguage(['zh-TW']), 'zh-TW')
  assert.equal(resolveLanguage(['zh-Hant-TW']), 'zh-TW')
  assert.equal(resolveLanguage(['zh-HK']), 'zh-TW')
  assert.equal(resolveLanguage(['in-ID']), 'id')
  assert.equal(resolveLanguage(['ar-SA']), 'ar')
  assert.equal(resolveLanguage(['ur-PK']), 'ur')
  assert.equal(resolveLanguage(['system', null, 'ja-JP']), 'ja')
  assert.equal(resolveLanguage(['xx-YY', 'zz']), 'en')
})

test('RTL detection only flags ar and ur', () => {
  assert.deepEqual([...RTL_LANGUAGES].sort(), ['ar', 'ur'])
  for (const tag of EXPECTED_TAGS) {
    const expectedRtl = tag === 'ar' || tag === 'ur'
    assert.equal(isRtl(tag), expectedRtl, `isRtl(${tag})`)
    assert.equal(getDirection(tag), expectedRtl ? 'rtl' : 'ltr', `getDirection(${tag})`)
  }
})

test('i18n translates and substitutes parameters across locales', () => {
  setLanguage('en')
  assert.equal(getLanguage(), 'en')
  assert.equal(t('connect.title'), 'Connect to Kinema')

  setLanguage('fr-FR')
  assert.equal(getLanguage(), 'fr')
  assert.equal(t('connect.title'), 'Se connecter à Kinema')

  setLanguage('ar')
  assert.equal(getLanguage(), 'ar')
  assert.equal(isRtl(), true)
  const formatted = t('cert.detail', { host: '192.168.1.100', fingerprint: 'AA:BB:CC' })
  assert.ok(formatted.includes('192.168.1.100'))
  assert.ok(formatted.includes('AA:BB:CC'))

  assert.equal(t('non_existent_key'), 'non_existent_key')
  setLanguage('en')
})
