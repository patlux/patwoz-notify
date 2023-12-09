/// <reference lib="WebWorker" />

const sw: ServiceWorkerGlobalScope = self as any

sw.addEventListener('push', async (event) => {
  const payload = event.data?.text()
  console.log(`Got push: "${payload}".`)
  try {
    const data = JSON.parse(payload ?? '{}')
    const { title, body } = data
    sw.registration.showNotification(title, {
      body,
    })
  } catch (error: unknown) {
    if (`${error}`.includes('is not valid JSON')) {
      sw.registration.showNotification('New message', {
        body: payload ?? 'No message',
      })
      return
    }
    console.error(`ERR in sw push handler: "${error}".`)
  }
})
