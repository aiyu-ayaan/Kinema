const { test } = require('node:test')
const assert = require('node:assert/strict')
const { t, setLanguage, getLanguage, resolveLanguage } = require('../src/i18n')

test('i18n resolves language and falls back to en', () => {
  assert.equal(resolveLanguage(['en-US']), 'en')
  assert.equal(resolveLanguage(['fr-FR']), 'en') // only en catalog exists currently
})

test('i18n translates and substitutes parameters', () => {
  setLanguage('en')
  assert.equal(getLanguage(), 'en')

  const title = t('connect.title')
  assert.equal(title, 'Connect to Kinema')

  const formatted = t('cert.detail', { host: '192.168.1.100', fingerprint: 'AA:BB:CC' })
  assert.ok(formatted.includes('192.168.1.100'))
  assert.ok(formatted.includes('AA:BB:CC'))

  const fallback = t('non_existent_key')
  assert.equal(fallback, 'non_existent_key')
})
