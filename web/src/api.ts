export const fetchApi = (pathname: string, options?: RequestInit) => {
  return fetch(`/api${pathname}`, options)
}

export const fetchJson = async (pathname: string, options?: RequestInit) => {
  const response = await fetchApi(pathname, options)
  return response.json()
}

export const getVapidPublicKey = async () => {
  const { vapidPublicKey } = await fetchJson('/public-key')
  return vapidPublicKey
}

export const subscribe = async (subscription: PushSubscription) => {
  const response = await fetchApi(`/subscribe`, {
    method: 'POST',
    headers: new Headers({
      'Content-Type': 'application/json',
    }),
    body: JSON.stringify(subscription),
  })
  if (response.status !== 304 && response.status !== 200) {
    throw new Error(`Server response not ok.`)
  }
  return response
}

export const send = (subscription: PushSubscription) => {
  return fetchApi(`/send`, {
    method: 'POST',
    headers: new Headers({
      'Content-Type': 'application/json',
    }),
    body: JSON.stringify(subscription),
  })
}
