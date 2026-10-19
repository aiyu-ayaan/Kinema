const { test } = require('node:test')
const assert = require('node:assert/strict')
const { removeDefaultMenu } = require('../src/main')

test('default desktop application menu items are removed via removeDefaultMenu', () => {
  let calledWith = undefined
  const fakeMenu = {
    setApplicationMenu: (menu) => {
      calledWith = menu
    },
  }

  removeDefaultMenu(fakeMenu)
  assert.equal(
    calledWith,
    null,
    'removeDefaultMenu must set application menu to null so default File/Edit/View menu items are removed'
  )
})
