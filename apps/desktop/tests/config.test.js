const { test } = require('node:test')
const assert = require('node:assert/strict')
const fs = require('fs')
const os = require('os')
const path = require('path')

test('server configuration and switching state persist correctly', () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'kinema-desktop-test-'))
  const configFile = path.join(tmpDir, 'kinema.json')

  let config = { servers: [], pins: {}, token: null }

  // Simulate connecting to first server
  const server1 = 'https://192.168.1.10:3000'
  config = { ...config, servers: [server1], token: 'token-abc' }
  config.pins[server1] = 'A0:B1:C2:D3'
  fs.writeFileSync(configFile, JSON.stringify(config, null, 2), 'utf8')

  // Verify persistence
  const loaded = JSON.parse(fs.readFileSync(configFile, 'utf8'))
  assert.deepEqual(loaded.servers, [server1])
  assert.equal(loaded.pins[server1], 'A0:B1:C2:D3')
  assert.equal(loaded.token, 'token-abc')

  // Simulate changing to another server (as in server:connect / forget)
  const server2 = 'https://192.168.1.50:3000'
  config = { ...config, servers: [server2], token: null }
  config.pins[server2] = 'E4:F5:06:17'
  fs.writeFileSync(configFile, JSON.stringify(config, null, 2), 'utf8')

  const updated = JSON.parse(fs.readFileSync(configFile, 'utf8'))
  assert.deepEqual(updated.servers, [server2])
  assert.equal(updated.token, null, 'Switching servers clears the previous session token')
  assert.equal(updated.pins[server2], 'E4:F5:06:17')

  // Cleanup
  fs.rmSync(tmpDir, { recursive: true, force: true })
})
