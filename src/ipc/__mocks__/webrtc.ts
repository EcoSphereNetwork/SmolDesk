export function setupWebRTCMocks() {
  globalThis.navigator.mediaDevices = {
    getUserMedia: async () => ({
      getTracks: () => [{ stop: () => {} }]
    })
  } as MediaDevices

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  globalThis.RTCPeerConnection = class {
    createOffer = async () => ({ sdp: 'mock-sdp' })
    setLocalDescription = async () => {}
    addTrack = () => {}
  } as unknown as typeof RTCPeerConnection
}
