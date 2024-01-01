export type Subscription = {
  id: number
  data: {
    expiration_time: null | number
    endpoint: String
    keys: {
      p256dh: String
      auth: String
    }
  }
}

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

export const getSubscriptions = async () => {
  return fetchJson(`/subscriptions`).then(
    (data) => data as { subscriptions: Subscription[] },
  )
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

export const send = (
  subscription: PushSubscription,
  notification: { title: string; body: string },
) => {
  return fetchApi(`/send`, {
    method: 'POST',
    headers: new Headers({
      'Content-Type': 'application/json',
    }),
    body: JSON.stringify({
      subscription,
      notification,
    }),
  })
}

export const sendToAll = (notification: { title: string; body: string }) => {
  return fetchApi(`/send-to-all`, {
    method: 'POST',
    headers: new Headers({
      'Content-Type': 'application/json',
    }),
    body: JSON.stringify({
      notification,
    }),
  })
}
