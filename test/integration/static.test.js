import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { SAILBOAT_URL } from './setup.js'

describe('Static files test', async () => {

  it('shows "Hello World!"', async () => {
    const res = await fetch(`${SAILBOAT_URL}/static/common.css`)
    const body = await res.text()
    assert.equal(body, 'body {\n  max-width: 800px;\n}\n')
  })

})


