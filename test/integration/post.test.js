import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { SAILBOAT_URL } from './setup.js'

describe('/post', async () => {

  it('posts a new post', async () => {
    const body = 'feed_id=1&content=posting%20new%20content'
    const res = await fetch(`${SAILBOAT_URL}/post`, { body, method: 'POST' })

    const text = await res.text()
    assert.equal(res.status, 200)
    assert.match(text, /posting new content/)
  })

  it('rejects posts without a feed_id', async () => {
    const body = 'content=posting%20new%20content'
    const res = await fetch(`${SAILBOAT_URL}/post`, { body, method: 'POST' })
    // TODO change to 400
    assert.equal(res.status, 500)
  })

  it('rejects posts without content', async () => {
    const body = 'feed_id=1'
    const res = await fetch(`${SAILBOAT_URL}/post`, { body, method: 'POST' })
    // TODO change to 400
    assert.equal(res.status, 500)
  })

})

