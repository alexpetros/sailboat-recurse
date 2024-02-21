import { describe, it } from 'node:test'
import assert from 'node:assert/strict'

describe('echo tests', async () => {

  it('says hello world', async () => {
    const res = await fetch('http://localhost:3000')
    const body = await res.text()
    assert.equal(body, 'Hello, World!\n')
  })

  it('echoes a post body', async () => {
    const body = JSON.stringify({ test: 'hi' })
    const res = await fetch('http://localhost:3000/echo', { method: 'POST', body })
    const text = await res.text()
    assert.equal(text, '{"test":"hi"}')
  })

  it('echoes an uppercased post body', async () => {
    const body = JSON.stringify({ test: 'hi' })
    const res = await fetch('http://localhost:3000/echo/uppercase', { method: 'POST', body })
    const text = await res.text()
    assert.equal(text, '{"TEST":"HI"}')
  })

  it('echoes a reversed body', async () => {
    const body = JSON.stringify({ test: 'hi' })
    const res = await fetch('http://localhost:3000/echo/reversed', { method: 'POST', body })
    const text = await res.text()
    assert.equal(text, '}"ih":"tset"{')
  })

})
