const { test } = require('node:test')
const assert = require('node:assert/strict')
const { normalize, isLanHost } = require('../src/main')

test('isLanHost identifies private network addresses', () => {
  assert.equal(isLanHost('localhost'), true)
  assert.equal(isLanHost('127.0.0.1'), true)
  assert.equal(isLanHost('10.0.2.2'), true)
  assert.equal(isLanHost('myhost.local'), true)
  assert.equal(isLanHost('192.168.1.10'), true)
  assert.equal(isLanHost('10.1.2.3'), true)
  assert.equal(isLanHost('172.16.0.1'), true)
  assert.equal(isLanHost('172.31.255.254'), true)

  assert.equal(isLanHost('172.32.0.1'), false)
  assert.equal(isLanHost('8.8.8.8'), false)
  assert.equal(isLanHost('media.example.com'), false)
  assert.equal(isLanHost(''), false)
})

test('normalize parses and normalizes server URLs', () => {
  // Matching Android test suite in CoreTest.kt
  assert.equal(normalize('192.168.1.10'), 'https://192.168.1.10:3000')
  assert.equal(normalize(' 192.168.1.10:3001/ '), 'https://192.168.1.10:3001')
  assert.equal(normalize('media.example.com'), 'https://media.example.com')
  assert.equal(normalize('http://nas.local:3000/api'), 'http://nas.local:3000')
  assert.equal(normalize('https://10.0.2.2:3000'), 'https://10.0.2.2:3000')
  assert.equal(normalize('localhost'), 'https://localhost:3000')

  // Invalid addresses
  assert.equal(normalize(''), null)
  assert.equal(normalize('not a url'), null)
  assert.equal(normalize('   '), null)
})

test('home address goes first, public ones after it', () => {
  const { preferHome, isLanUrl } = require('../src/main')
  assert.deepEqual(preferHome(['https://media.example.com'], 'https://192.168.1.10:3000'), ['https://192.168.1.10:3000', 'https://media.example.com'])
  assert.deepEqual(preferHome(['https://media.example.com', 'https://192.168.1.10:3000'], 'https://192.168.1.10:3000'), ['https://192.168.1.10:3000', 'https://media.example.com'])
  assert.equal(isLanUrl('https://192.168.1.10:3000'), true)
  assert.equal(isLanUrl('https://media.example.com'), false)
  assert.equal(isLanUrl('not a url'), false)
})
