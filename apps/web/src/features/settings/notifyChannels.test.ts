import assert from 'node:assert/strict'
import { test } from 'node:test'
import { channelFields, channelKinds, newChannel } from './notifyChannels.ts'

test('every kind has fields, and secrets are only tokens and passwords', () => {
  for (const kind of channelKinds) {
    const fields = channelFields[kind]
    assert.ok(fields.length > 0, kind)
    for (const f of fields.filter((f) => f.secret)) assert.ok(['token', 'smtp_password'].includes(f.field), `${kind}.${f.field}`)
  }
})

test('new channels start with the kind defaults', () => {
  assert.equal(newChannel('ntfy').url, 'https://ntfy.sh')
  assert.equal(newChannel('discord').url, '')
  assert.deepEqual([newChannel('smtp').smtp_port, newChannel('smtp').smtp_security], [587, 'starttls'])
})
