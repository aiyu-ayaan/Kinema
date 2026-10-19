const { test } = require('node:test')
const assert = require('node:assert/strict')
const { parseReply } = require('../src/main')

test('parseReply parses valid Kinema discovery responses', () => {
  const reply = parseReply(
    JSON.stringify({ service: 'kinema', name: 'Den', port: 3000, tls: true, version: '0.1.0' }),
    '192.168.1.5'
  )
  assert.deepEqual(reply, {
    name: 'Den',
    url: 'https://192.168.1.5:3000',
    version: '0.1.0',
  })
})

test('parseReply respects scheme and IPv6 formatting', () => {
  const httpReply = parseReply(
    JSON.stringify({ service: 'kinema', port: 80, tls: false }),
    '192.168.1.5'
  )
  assert.equal(httpReply.url, 'http://192.168.1.5:80')

  const ipv6Reply = parseReply(
    JSON.stringify({ service: 'kinema', port: 3000, tls: true }),
    'fe80::1%wlan0'
  )
  assert.equal(ipv6Reply.url, 'https://[fe80::1]:3000')
})

test('parseReply rejects non-Kinema or malformed responses', () => {
  assert.equal(
    parseReply(JSON.stringify({ service: 'domus', port: 8443 }), '1.2.3.4'),
    null
  )
  assert.equal(
    parseReply(JSON.stringify({ service: 'kinema', port: 0 }), '1.2.3.4'),
    null
  )
  assert.equal(parseReply('garbage', '1.2.3.4'), null)
})
